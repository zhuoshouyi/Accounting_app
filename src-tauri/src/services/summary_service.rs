// 月度汇总服务
// 实现月度消费汇总计算 + 汇总映射合并 + 下钻数据查询
//
// 流程：
// 1. 查询当月有效支出交易，按 tag_id GROUP BY + SUM
// 2. 加载 summary_mappings 表，将子标签合并到汇总类
// 3. 加载 category_tags 表，获取标签名称
// 4. 返回结构化的月度汇总数据

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::dao::transaction_dao;
use crate::models::transaction::Transaction;

// ====================================================================
// 数据结构定义
// ====================================================================

/// 标签级汇总（单个标签的金额统计）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagSummary {
    /// 标签 ID
    pub tag_id: String,
    /// 标签名称
    pub tag_name: String,
    /// 该标签总金额
    pub amount: f64,
    /// 交易笔数
    pub count: i64,
}

/// 汇总类（合并后的汇总条目，包含子标签明细）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryCategory {
    /// 汇总类名称（如"买菜"、"家用"）
    pub summary_category: String,
    /// 汇总类总金额
    pub total_amount: f64,
    /// 交易笔数
    pub total_count: i64,
    /// 包含的子标签明细
    pub tags: Vec<TagSummary>,
    /// 排序序号
    pub sort_order: i32,
}

/// 月度汇总完整结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlySummary {
    /// 月份（YYYY-MM）
    pub month: String,
    /// 各汇总类明细
    pub categories: Vec<SummaryCategory>,
    /// 总支出
    pub total_expense: f64,
    /// 总交易笔数
    pub transaction_count: i64,
}

/// 汇总映射行（从 summary_mappings 表读取）
struct MappingRow {
    summary_category: String,
    tag_id: String,
    sort_order: i32,
}

// ====================================================================
// 辅助：查询汇总映射 + 标签名称
// ====================================================================

/// 加载所有汇总映射，按 sort_order 排序
fn load_mappings(conn: &Connection) -> Result<Vec<MappingRow>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT summary_category, tag_id, sort_order
             FROM summary_mappings
             ORDER BY sort_order ASC, summary_category ASC",
        )
        .map_err(|e| format!("查询汇总映射失败: {}", e))?;

    let mappings = stmt
        .query_map([], |row| {
            Ok(MappingRow {
                summary_category: row.get(0)?,
                tag_id: row.get(1)?,
                sort_order: row.get(2)?,
            })
        })
        .map_err(|e| format!("映射汇总映射失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("读取汇总映射失败: {}", e))?;

    Ok(mappings)
}

/// 加载所有标签名称映射（tag_id → tag_name）
fn load_tag_names(conn: &Connection) -> Result<std::collections::HashMap<String, String>, String> {
    let mut stmt = conn
        .prepare("SELECT id, name FROM category_tags")
        .map_err(|e| format!("查询标签失败: {}", e))?;

    let map: std::collections::HashMap<String, String> = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| format!("映射标签失败: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(map)
}

// ====================================================================
// 公共 API
// ====================================================================

/// 获取月度汇总
///
/// # 参数
/// - `conn`: 数据库连接
/// - `month`: 月份（YYYY-MM 格式）
///
/// # 返回
/// MonthlySummary — 含各汇总类金额、子标签明细、总支出
pub fn get_monthly_summary(conn: &Connection, month: &str) -> Result<MonthlySummary, String> {
    // 1. 查询当月有效支出，按 tag_id 分组聚合
    let mut stmt = conn
        .prepare(
            "SELECT COALESCE(tag_id, '__unclassified__') AS tag_id,
                    SUM(amount) AS total_amount,
                    COUNT(*) AS cnt
             FROM transactions
             WHERE month = ?
               AND direction = 'expense'
               AND is_excluded_from_summary = 0
             GROUP BY tag_id",
        )
        .map_err(|e| format!("查询月度汇总失败: {}", e))?;

    let tag_amounts: Vec<(String, f64, i64)> = stmt
        .query_map(params![month], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, f64>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })
        .map_err(|e| format!("映射汇总数据失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("读取汇总数据失败: {}", e))?;

    // 2. 加载汇总映射和标签名称
    let mappings = load_mappings(conn)?;
    let tag_names = load_tag_names(conn)?;

    // 3. 构建 tag_id → (amount, count) 查找表
    let amount_map: std::collections::HashMap<&str, (f64, i64)> = tag_amounts
        .iter()
        .map(|(id, amt, cnt)| (id.as_str(), (*amt, *cnt)))
        .collect();

    // 4. 按汇总类合并且保持 sort_order 顺序
    let mut cat_map: std::collections::BTreeMap<String, (Vec<TagSummary>, i32)> =
        std::collections::BTreeMap::new();

    // 记录汇总类出现顺序（保持 sort_order 排序）
    let mut cat_order: Vec<(String, i32)> = Vec::new();
    let mut seen_cats = std::collections::HashSet::new();

    for mapping in &mappings {
        if !seen_cats.contains(&mapping.summary_category) {
            seen_cats.insert(mapping.summary_category.clone());
            cat_order.push((mapping.summary_category.clone(), mapping.sort_order));
        }

        let tag_id = mapping.tag_id.as_str();
        let tag_name = tag_names
            .get(mapping.tag_id.as_str())
            .cloned()
            .unwrap_or_else(|| mapping.tag_id.clone());

        if let Some(&(amount, count)) = amount_map.get(tag_id) {
            let entry = cat_map
                .entry(mapping.summary_category.clone())
                .or_insert_with(|| (Vec::new(), mapping.sort_order));

            entry.0.push(TagSummary {
                tag_id: mapping.tag_id.clone(),
                tag_name,
                amount: (amount * 100.0).round() / 100.0,
                count,
            });
        }
    }

    // 5. 处理未分类的交易（tag_id IS NULL）
    if let Some(&(amount, count)) = amount_map.get("__unclassified__") {
        let entry = cat_map
            .entry("未分类".to_string())
            .or_insert_with(|| (Vec::new(), 999));
        entry.0.push(TagSummary {
            tag_id: "__unclassified__".to_string(),
            tag_name: "未分类".to_string(),
            amount: (amount * 100.0).round() / 100.0,
            count,
        });
        if !seen_cats.contains("未分类") {
            cat_order.push(("未分类".to_string(), 999));
        }
    }

    // 6. 检查是否有标签不在任何汇总映射中（孤立标签）
    let mapped_tag_ids: std::collections::HashSet<&str> =
        mappings.iter().map(|m| m.tag_id.as_str()).collect();
    for (tag_id, amount, count) in &tag_amounts {
        if *tag_id == "__unclassified__" {
            continue;
        }
        if !mapped_tag_ids.contains(tag_id.as_str()) {
            let tag_name = tag_names
                .get(tag_id.as_str())
                .cloned()
                .unwrap_or_else(|| tag_id.clone());
            let entry = cat_map
                .entry("其他".to_string())
                .or_insert_with(|| (Vec::new(), 998));
            entry.0.push(TagSummary {
                tag_id: tag_id.clone(),
                tag_name,
                amount: (*amount * 100.0).round() / 100.0,
                count: *count,
            });
            if !seen_cats.contains("其他") {
                cat_order.push(("其他".to_string(), 998));
            }
        }
    }

    // 7. 按 sort_order 排序，构建最终结果
    cat_order.sort_by_key(|(_, order)| *order);

    let mut categories = Vec::new();
    let mut total_expense = 0.0;
    let mut transaction_count = 0i64;

    for (cat_name, sort_order) in &cat_order {
        if let Some((tags, _)) = cat_map.get(cat_name) {
            let total_amount: f64 = tags.iter().map(|t| t.amount).sum();
            let total_cnt: i64 = tags.iter().map(|t| t.count).sum();
            let rounded_total = (total_amount * 100.0).round() / 100.0;

            categories.push(SummaryCategory {
                summary_category: cat_name.clone(),
                total_amount: rounded_total,
                total_count: total_cnt,
                tags: tags.clone(),
                sort_order: *sort_order,
            });

            total_expense += rounded_total;
            transaction_count += total_cnt;
        }
    }

    total_expense = (total_expense * 100.0).round() / 100.0;

    Ok(MonthlySummary {
        month: month.to_string(),
        categories,
        total_expense,
        transaction_count,
    })
}

/// 按月份和标签列表查询交易明细（供下钻跳转使用，委托 DAO 层）
pub fn get_transactions_by_tags(
    conn: &Connection,
    month: &str,
    tag_ids: &[String],
) -> Result<Vec<Transaction>, String> {
    transaction_dao::list_transactions_by_tags(conn, month, tag_ids)
}

/// 多月份汇总行（对应 Excel【家庭记账本】汇总的每一行）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlySummaryRow {
    pub month: String,
    pub total_assets: Option<f64>,
    pub joey_income: Option<f64>,
    pub vila_income: Option<f64>,
    pub total_expense: f64,
    pub mortgage_savings: Option<f64>,
    pub categories: Vec<SummaryCategory>,
    pub investment: Option<f64>,
    pub insurance: Option<f64>,
    pub analysis_text: Option<String>,
    pub details_json: Option<String>,
}

/// 获取全部月份汇总透视表
pub fn get_all_months_summary(conn: &Connection) -> Result<Vec<MonthlySummaryRow>, String> {
    let months = crate::dao::transaction_dao::get_distinct_months(conn)?;
    let manual_dao = |m: &str| -> Option<crate::models::monthly_manual_data::MonthlyManualData> {
        crate::dao::manual_data_dao::get_manual_data(conn, m).ok().flatten()
    };

    let mut rows = Vec::new();
    for month in &months {
        let summary = get_monthly_summary(conn, month)?;
        let manual = manual_dao(month);
        rows.push(MonthlySummaryRow {
            month: month.clone(),
            total_assets: manual.as_ref().and_then(|d| d.total_assets),
            joey_income: manual.as_ref().and_then(|d| d.joey_income),
            vila_income: manual.as_ref().and_then(|d| d.vila_income),
            total_expense: summary.total_expense,
            mortgage_savings: manual.as_ref().and_then(|d| d.mortgage_savings),
            categories: summary.categories,
            investment: manual.as_ref().and_then(|d| d.investment),
            insurance: manual.as_ref().and_then(|d| d.insurance),
            analysis_text: manual.as_ref().and_then(|d| d.analysis_text.clone()),
            details_json: manual.as_ref().and_then(|d| d.details_json.clone()),
        });
    }
    Ok(rows)
}
