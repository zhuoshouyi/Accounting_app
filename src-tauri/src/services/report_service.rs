// 报表服务 — 报表生成 + 历史管理

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::models::ai_report::AiReport;

/// 月度报表数据结构（供前端图表使用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyChartData {
    pub month: String,
    pub categories: Vec<CategoryAmount>,
    pub total_expense: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryAmount {
    pub name: String,
    pub amount: f64,
}

/// 多月趋势数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendData {
    pub months: Vec<String>,
    pub series: Vec<CategorySeries>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorySeries {
    pub name: String,
    pub data: Vec<f64>,
}

/// 获取多月趋势数据（供 Recharts 图表使用）
pub fn get_trend_data(conn: &Connection, months: &[String]) -> Result<TrendData, String> {
    use std::collections::BTreeMap;

    let mappings = load_summary_mappings(conn)?;

    // category_name → [month → amount]
    let mut cat_data: BTreeMap<String, Vec<Option<f64>>> = BTreeMap::new();
    for mapping in &mappings {
        cat_data.entry(mapping.clone()).or_insert_with(|| vec![None; months.len()]);
    }

    for (i, month) in months.iter().enumerate() {
        let mut stmt = conn
            .prepare(
                "SELECT COALESCE(tag_id, '__unclassified__'), SUM(amount)
                 FROM transactions
                 WHERE month = ? AND direction = 'expense' AND is_excluded_from_summary = 0
                 GROUP BY tag_id",
            )
            .map_err(|e| format!("查询趋势数据失败: {}", e))?;

        let tag_amounts: Vec<(String, f64)> = stmt
            .query_map(rusqlite::params![month], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
            })
            .map_err(|e| format!("映射趋势数据失败: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("读取趋势数据失败: {}", e))?;

        // 将 tag 合并到汇总类
        let tag_map: std::collections::HashMap<&str, f64> =
            tag_amounts.iter().map(|(id, amt)| (id.as_str(), *amt)).collect();

        let mut cat_totals: BTreeMap<String, f64> = BTreeMap::new();

        // 直接按映射表合并
        let mappings_detail = load_mappings_detail(conn)?;
        for (cat, tag_ids) in &mappings_detail {
            let total: f64 = tag_ids.iter().filter_map(|tid| tag_map.get(tid.as_str())).sum();
            *cat_totals.entry(cat.clone()).or_insert(0.0) += total;
        }

        for (cat, amount) in &cat_totals {
            if let Some(data) = cat_data.get_mut(cat) {
                data[i] = Some(*amount);
            }
        }
    }

    let series: Vec<CategorySeries> = cat_data
        .into_iter()
        .map(|(name, data)| CategorySeries {
            name,
            data: data.into_iter().map(|v| v.unwrap_or(0.0)).collect(),
        })
        .collect();

    Ok(TrendData {
        months: months.to_vec(),
        series,
    })
}

fn load_summary_mappings(conn: &Connection) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare("SELECT DISTINCT summary_category FROM summary_mappings ORDER BY sort_order")
        .map_err(|e| format!("查询映射失败: {}", e))?;
    let cats = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| format!("映射失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("读取失败: {}", e))?;
    Ok(cats)
}

fn load_mappings_detail(
    conn: &Connection,
) -> Result<std::collections::BTreeMap<String, Vec<String>>, String> {
    let mut stmt = conn
        .prepare("SELECT summary_category, tag_id FROM summary_mappings ORDER BY sort_order")
        .map_err(|e| format!("查询映射失败: {}", e))?;
    let rows: Vec<(String, String)> = stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
        .map_err(|e| format!("映射失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("读取失败: {}", e))?;

    let mut map: std::collections::BTreeMap<String, Vec<String>> = std::collections::BTreeMap::new();
    for (cat, tag_id) in rows {
        map.entry(cat).or_default().push(tag_id);
    }
    Ok(map)
}

/// 保存 AI 报表到历史
pub fn save_report(
    conn: &Connection,
    month: &str,
    report_type: &str,
    title: &str,
    content: &str,
    summary_json: &str,
    model_name: &str,
) -> Result<AiReport, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();

    conn.execute(
        r#"INSERT INTO ai_reports (
            id, month, report_type, title, content, summary_data, model_name, created_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
        rusqlite::params![id, month, report_type, title, content, summary_json, model_name, now],
    )
    .map_err(|e| format!("保存报表失败: {}", e))?;

    Ok(AiReport {
        id,
        month: month.to_string(),
        report_type: report_type.to_string(),
        title: Some(title.to_string()),
        content: Some(content.to_string()),
        summary_data: Some(summary_json.to_string()),
        model_name: Some(model_name.to_string()),
        created_at: now,
    })
}

/// 获取报表历史（按月份筛选，month=None 查全部）
pub fn get_report_history(
    conn: &Connection,
    month: Option<&str>,
) -> Result<Vec<AiReport>, String> {
    let reports = match month {
        Some(m) => {
            let mut stmt = conn
                .prepare("SELECT * FROM ai_reports WHERE month = ? ORDER BY created_at DESC")
                .map_err(|e| format!("查询报表历史失败: {}", e))?;
            let rows: Vec<AiReport> = stmt
                .query_map(rusqlite::params![m], row_to_ai_report)
                .map_err(|e| format!("映射报表失败: {}", e))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("读取报表失败: {}", e))?;
            rows
        }
        None => {
            let mut stmt = conn
                .prepare("SELECT * FROM ai_reports ORDER BY created_at DESC")
                .map_err(|e| format!("查询报表历史失败: {}", e))?;
            let rows: Vec<AiReport> = stmt
                .query_map([], row_to_ai_report)
                .map_err(|e| format!("映射报表失败: {}", e))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("读取报表失败: {}", e))?;
            rows
        }
    };
    Ok(reports)
}

/// 根据 ID 获取单个报表
pub fn get_report_by_id(conn: &Connection, id: &str) -> Result<Option<AiReport>, String> {
    let mut stmt = conn
        .prepare("SELECT * FROM ai_reports WHERE id = ?")
        .map_err(|e| format!("查询报表失败: {}", e))?;
    let result = stmt
        .query_row(rusqlite::params![id], row_to_ai_report)
        .ok();
    Ok(result)
}

fn row_to_ai_report(row: &rusqlite::Row) -> rusqlite::Result<AiReport> {
    Ok(AiReport {
        id: row.get("id")?,
        month: row.get("month")?,
        report_type: row.get("report_type")?,
        title: row.get("title")?,
        content: row.get("content")?,
        summary_data: row.get("summary_data")?,
        model_name: row.get("model_name")?,
        created_at: row.get("created_at")?,
    })
}
