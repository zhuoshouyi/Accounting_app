// 分类规则引擎
// 实现三层匹配优先级的自动分类逻辑：
//   第 1 层：AI 学习规则（ai_learning_rules，按 confidence DESC）
//   第 2 层：内置/用户规则（category_rules，按 priority ASC）
//   第 3 层：留空（进入人工复核队列）

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::dao::rule_dao;
use crate::dao::transaction_dao;
use crate::models::ai_learning_rule::AiLearningRule;
use crate::models::category_rule::CategoryRule;
use crate::models::transaction::Transaction;

// ====================================================================
// 数据结构定义
// ====================================================================

/// 分类结果统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifyResult {
    /// 待分类交易总数
    pub total: usize,
    /// 成功匹配的数量
    pub classified: usize,
    /// 未匹配（留空）的数量
    pub unclassified: usize,
}

// ====================================================================
// 匹配核心逻辑
// ====================================================================

/// 根据匹配字段获取交易中对应的字段值
///
/// # 参数
/// - `tx`: 交易记录
/// - `match_field`: 匹配字段名（counterparty / product / transaction_type）
///
/// # 返回
/// 字段值（如果字段为 None 或空字符串则返回 None）
fn get_field_value<'a>(tx: &'a Transaction, match_field: &str) -> Option<&'a str> {
    let value = match match_field {
        "counterparty" => tx.counterparty.as_deref(),
        "product" => tx.product.as_deref(),
        "transaction_type" => tx.transaction_type.as_deref(),
        _ => return None,
    };

    // 字段值为 None 或空字符串时不匹配任何规则
    match value {
        Some(v) if !v.is_empty() => Some(v),
        _ => None,
    }
}

/// 单条规则匹配判断
///
/// # 参数
/// - `field_value`: 交易字段值（保证非空）
/// - `match_type`: 匹配方式（exact / like / in）
/// - `match_value`: 规则匹配值
///
/// # 返回
/// true 表示匹配成功
///
/// # 匹配规则
/// - `exact`：精确等于（大小写敏感）
/// - `like`：包含子串（大小写不敏感）
/// - `in`：值在逗号分隔的列表中（大小写不敏感）
fn try_match(field_value: &str, match_type: &str, match_value: &str) -> bool {
    match match_type {
        // 精确匹配：大小写敏感
        "exact" => field_value == match_value,
        // 包含匹配：大小写不敏感
        "like" => field_value
            .to_lowercase()
            .contains(&match_value.to_lowercase()),
        // 列表匹配：大小写不敏感，match_value 为逗号分隔列表
        "in" => match_value.split(',').any(|v| {
            v.trim().eq_ignore_ascii_case(field_value)
        }),
        // 未知匹配方式，不匹配
        _ => false,
    }
}

/// 遍历 AI 学习规则列表，返回首个匹配的标签
///
/// # 参数
/// - `tx`: 交易记录
/// - `rules`: AI 学习规则列表（已按 confidence DESC 排序）
///
/// # 返回
/// Some((tag_id, tag_source)) 表示匹配成功，None 表示未匹配
fn match_ai_rules(tx: &Transaction, rules: &[AiLearningRule]) -> Option<(String, String)> {
    for rule in rules {
        if let Some(field_value) = get_field_value(tx, &rule.match_field) {
            if try_match(field_value, &rule.match_type, &rule.match_value) {
                return Some((rule.target_tag_id.clone(), "ai_learned".to_string()));
            }
        }
    }
    None
}

/// 遍历内置/用户规则列表，返回首个匹配的标签
///
/// # 参数
/// - `tx`: 交易记录
/// - `rules`: 分类规则列表（已按 priority ASC, match_field 排序）
///
/// # 返回
/// Some((tag_id, tag_source)) 表示匹配成功，None 表示未匹配
fn match_builtin_rules(tx: &Transaction, rules: &[CategoryRule]) -> Option<(String, String)> {
    for rule in rules {
        if let Some(field_value) = get_field_value(tx, &rule.match_field) {
            if try_match(field_value, &rule.match_type, &rule.match_value) {
                return Some((rule.target_tag_id.clone(), "rule".to_string()));
            }
        }
    }
    None
}

// ====================================================================
// 公共 API
// ====================================================================

/// 执行自动分类
///
/// 流程：
/// 1. 加载 AI 学习规则（按 confidence DESC）
/// 2. 加载内置规则（按 priority ASC, match_field 排序）
/// 3. 查询待分类交易（tag_source IS NULL AND is_excluded_from_summary=0）
/// 4. 对每条交易依次尝试 AI 规则 → 内置规则 → 留空
/// 5. 匹配成功则更新 tag_id 和 tag_source
///
/// # 参数
/// - `conn`: 数据库连接
///
/// # 返回
/// 分类结果统计
pub fn classify_transactions(conn: &Connection) -> Result<ClassifyResult, String> {
    // 步骤 1：加载 AI 学习规则（可能为空，Step 7 实现 AI 功能后才有数据）
    let ai_rules = rule_dao::list_enabled_ai_rules(conn)?;

    // 步骤 2：加载内置/用户规则
    let builtin_rules = rule_dao::list_enabled_rules(conn)?;

    // 步骤 3：查询待分类交易
    let transactions = transaction_dao::list_unclassified_transactions(conn)?;

    let total = transactions.len();
    let mut classified = 0usize;

    // 步骤 4：逐条匹配
    let now = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string();

    conn.execute_batch("BEGIN")
        .map_err(|e| format!("开启事务失败: {}", e))?;

    for tx in &transactions {
        // 第 1 层：AI 学习规则匹配
        let matched = match_ai_rules(tx, &ai_rules)
            .or_else(|| match_builtin_rules(tx, &builtin_rules));

        match matched {
            Some((tag_id, tag_source)) => {
                // 匹配成功，更新标签
                let result = conn.execute(
                    "UPDATE transactions SET tag_id = ?, tag_source = ?, updated_at = ? WHERE id = ?",
                    rusqlite::params![tag_id, tag_source, now, tx.id],
                );
                match result {
                    Ok(_) => classified += 1,
                    Err(e) => {
                        conn.execute_batch("ROLLBACK").ok();
                        return Err(format!(
                            "更新交易标签失败（id: {}, 交易时间: {}）: {}",
                            tx.id, tx.transaction_time, e
                        ));
                    }
                }
            }
            None => {
                // 第 3 层：留空，不更新（tag_source 保持 NULL）
            }
        }
    }

    conn.execute_batch("COMMIT")
        .map_err(|e| format!("提交事务失败: {}", e))?;

    let unclassified = total - classified;

    Ok(ClassifyResult {
        total,
        classified,
        unclassified,
    })
}
