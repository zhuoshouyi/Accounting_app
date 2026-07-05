// 分类规则 DAO
// 提供对 category_rules 和 ai_learning_rules 表的查询操作

use rusqlite::{Connection, Row};

use crate::models::ai_learning_rule::AiLearningRule;
use crate::models::category_rule::CategoryRule;

/// 将数据库行映射为 CategoryRule 结构体
fn row_to_category_rule(row: &Row) -> rusqlite::Result<CategoryRule> {
    Ok(CategoryRule {
        id: row.get("id")?,
        match_field: row.get("match_field")?,
        match_type: row.get("match_type")?,
        match_value: row.get("match_value")?,
        target_tag_id: row.get("target_tag_id")?,
        priority: row.get("priority")?,
        enabled: row.get("enabled")?,
        source: row.get("source")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

/// 将数据库行映射为 AiLearningRule 结构体
fn row_to_ai_learning_rule(row: &Row) -> rusqlite::Result<AiLearningRule> {
    Ok(AiLearningRule {
        id: row.get("id")?,
        match_field: row.get("match_field")?,
        match_value: row.get("match_value")?,
        match_type: row.get("match_type")?,
        target_tag_id: row.get("target_tag_id")?,
        confidence: row.get("confidence")?,
        confirm_count: row.get("confirm_count")?,
        correct_count: row.get("correct_count")?,
        source: row.get("source")?,
        enabled: row.get("enabled")?,
        counterparty: row.get("counterparty").ok(),
        product: row.get("product").ok(),
        transaction_type: row.get("transaction_type").ok(),
        amount: row.get("amount").ok(),
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

/// 查询已启用的分类规则（enabled=1）
///
/// 排序规则：priority ASC（数值越小优先级越高），
/// 同 priority 按 match_field 排序（counterparty > product > transaction_type），
/// 最后按 id 排序保证稳定性。
///
/// # 参数
/// - `conn`: 数据库连接
///
/// # 返回
/// 规则列表（已排序）
pub fn list_enabled_rules(conn: &Connection) -> Result<Vec<CategoryRule>, String> {
    // match_field 排序优先级：counterparty(1) > product(2) > transaction_type(3)
    let mut stmt = conn
        .prepare(
            r#"SELECT * FROM category_rules
               WHERE enabled = 1
               ORDER BY priority ASC,
                        CASE match_field
                            WHEN 'counterparty' THEN 1
                            WHEN 'product' THEN 2
                            WHEN 'transaction_type' THEN 3
                            ELSE 4
                        END,
                        id ASC"#,
        )
        .map_err(|e| format!("准备查询失败: {}", e))?;
    let rules = stmt
        .query_map([], row_to_category_rule)
        .map_err(|e| format!("查询规则失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("映射规则失败: {}", e))?;
    Ok(rules)
}

/// 查询已启用的 AI 学习规则（enabled=1）
///
/// 排序规则：confidence DESC（置信度越高优先级越高），
/// 同 confidence 按 match_field 排序（counterparty > product > transaction_type）。
///
/// # 参数
/// - `conn`: 数据库连接
///
/// # 返回
/// AI 学习规则列表（已排序）
pub fn list_enabled_ai_rules(conn: &Connection) -> Result<Vec<AiLearningRule>, String> {
    let mut stmt = conn
        .prepare(
            r#"SELECT * FROM ai_learning_rules
               WHERE enabled = 1
               ORDER BY confidence DESC,
                        CASE match_field
                            WHEN 'counterparty' THEN 1
                            WHEN 'product' THEN 2
                            WHEN 'transaction_type' THEN 3
                            ELSE 4
                        END"#,
        )
        .map_err(|e| format!("准备查询失败: {}", e))?;
    let rules = stmt
        .query_map([], row_to_ai_learning_rule)
        .map_err(|e| format!("查询 AI 规则失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("映射 AI 规则失败: {}", e))?;
    Ok(rules)
}

// ====================================================================
// Step 9 新增：规则 CRUD
// ====================================================================

pub fn list_all_rules(conn: &Connection) -> Result<Vec<CategoryRule>, String> {
    let mut stmt = conn
        .prepare("SELECT * FROM category_rules ORDER BY source DESC, priority ASC, id ASC")
        .map_err(|e| format!("查询规则失败: {}", e))?;
    let rules = stmt.query_map([], row_to_category_rule)
        .map_err(|e| format!("映射规则失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("读取规则失败: {}", e))?;
    Ok(rules)
}

pub fn create_rule(conn: &Connection, rule: &CategoryRule) -> Result<CategoryRule, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    conn.execute(
        "INSERT INTO category_rules (id, match_field, match_type, match_value, target_tag_id, priority, enabled, source, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![id, rule.match_field, rule.match_type, rule.match_value, rule.target_tag_id, rule.priority, rule.enabled, rule.source, now, now],
    ).map_err(|e| format!("创建规则失败: {}", e))?;
    let mut r = rule.clone();
    r.id = id; r.created_at = now.clone(); r.updated_at = now;
    Ok(r)
}

pub fn update_rule(conn: &Connection, rule: &CategoryRule) -> Result<(), String> {
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    conn.execute(
        "UPDATE category_rules SET match_field=?, match_type=?, match_value=?, target_tag_id=?, priority=?, enabled=?, updated_at=? WHERE id=?",
        rusqlite::params![rule.match_field, rule.match_type, rule.match_value, rule.target_tag_id, rule.priority, rule.enabled, now, rule.id],
    ).map_err(|e| format!("更新规则失败: {}", e))?;
    Ok(())
}

pub fn delete_rule(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM category_rules WHERE id = ?", rusqlite::params![id])
        .map_err(|e| format!("删除规则失败: {}", e))?;
    Ok(())
}

pub fn toggle_rule(conn: &Connection, id: &str, enabled: i64) -> Result<(), String> {
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    conn.execute("UPDATE category_rules SET enabled = ?, updated_at = ? WHERE id = ?", rusqlite::params![enabled, now, id])
        .map_err(|e| format!("切换规则状态失败: {}", e))?;
    Ok(())
}

pub fn list_all_ai_rules(conn: &Connection) -> Result<Vec<AiLearningRule>, String> {
    let mut stmt = conn
        .prepare("SELECT * FROM ai_learning_rules ORDER BY confidence DESC")
        .map_err(|e| format!("查询 AI 规则失败: {}", e))?;
    let rules = stmt.query_map([], row_to_ai_learning_rule)
        .map_err(|e| format!("映射 AI 规则失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("读取 AI 规则失败: {}", e))?;
    Ok(rules)
}

pub fn delete_ai_rule(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM ai_learning_rules WHERE id = ?", rusqlite::params![id])
        .map_err(|e| format!("删除 AI 规则失败: {}", e))?;
    Ok(())
}
