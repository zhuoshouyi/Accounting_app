use tauri::State;
use crate::db::DatabaseState;
use crate::dao;
use crate::models::category_rule::CategoryRule;
use crate::models::category_tag::CategoryTag;
use crate::models::ai_learning_rule::AiLearningRule;
use crate::models::summary_mapping::SummaryMapping;

// ---- 分类规则 ----

#[tauri::command]
pub fn list_all_rules(state: State<'_, DatabaseState>) -> Result<Vec<CategoryRule>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    dao::rule_dao::list_all_rules(&conn)
}

#[tauri::command]
pub fn create_rule(state: State<'_, DatabaseState>, rule: CategoryRule) -> Result<CategoryRule, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    dao::rule_dao::create_rule(&conn, &rule)
}

#[tauri::command]
pub fn update_rule(state: State<'_, DatabaseState>, rule: CategoryRule) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    dao::rule_dao::update_rule(&conn, &rule)
}

#[tauri::command]
pub fn delete_rule(state: State<'_, DatabaseState>, id: String) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    dao::rule_dao::delete_rule(&conn, &id)
}

#[tauri::command]
pub fn toggle_rule(state: State<'_, DatabaseState>, id: String, enabled: bool) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    dao::rule_dao::toggle_rule(&conn, &id, if enabled { 1 } else { 0 })
}

// ---- AI 学习规则 ----

#[tauri::command]
pub fn list_ai_rules(state: State<'_, DatabaseState>) -> Result<Vec<AiLearningRule>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    dao::rule_dao::list_all_ai_rules(&conn)
}

#[tauri::command]
pub fn delete_ai_rule(state: State<'_, DatabaseState>, id: String) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    dao::rule_dao::delete_ai_rule(&conn, &id)
}

// ---- 标签管理 ----

#[tauri::command]
pub fn create_tag(state: State<'_, DatabaseState>, name: String, sort_order: i64) -> Result<CategoryTag, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    dao::tag_dao::create_tag(&conn, &name, sort_order)
}

#[tauri::command]
pub fn update_tag(state: State<'_, DatabaseState>, id: String, name: String, sort_order: i64) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    dao::tag_dao::update_tag(&conn, &id, &name, sort_order)
}

#[tauri::command]
pub fn delete_tag(state: State<'_, DatabaseState>, id: String) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    dao::tag_dao::delete_tag(&conn, &id)
}

// ---- 汇总映射 ----

#[tauri::command]
pub fn list_mappings(state: State<'_, DatabaseState>) -> Result<Vec<SummaryMapping>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    dao::mapping_dao::list_all(&conn)
}

#[tauri::command]
pub fn create_mapping(state: State<'_, DatabaseState>, summary_category: String, tag_id: String, sort_order: i64) -> Result<SummaryMapping, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    dao::mapping_dao::create_mapping(&conn, &summary_category, &tag_id, sort_order)
}

#[tauri::command]
pub fn delete_mapping(state: State<'_, DatabaseState>, id: String) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    dao::mapping_dao::delete_mapping(&conn, &id)
}

// ---- 规则测试 ----

/// 测试规则匹配：输入字段值，返回匹配到的标签信息
#[tauri::command]
pub fn test_rule_match(
    state: State<'_, DatabaseState>,
    counterparty: String,
    product: String,
    transaction_type: String,
) -> Result<Option<String>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let rules = dao::rule_dao::list_enabled_rules(&conn)?;
    let tags = dao::tag_dao::list_all_tags(&conn)?;
    let tag_map: std::collections::HashMap<&str, &str> = tags.iter().map(|t| (t.id.as_str(), t.name.as_str())).collect();

    for rule in &rules {
        let field_value = match rule.match_field.as_str() {
            "counterparty" => &counterparty,
            "product" => &product,
            "transaction_type" => &transaction_type,
            _ => continue,
        };
        if field_value.is_empty() { continue; }
        let matched = match rule.match_type.as_str() {
            "exact" => field_value == &rule.match_value,
            "like" => field_value.to_lowercase().contains(&rule.match_value.to_lowercase()),
            "in" => rule.match_value.split(',').any(|v| v.trim().eq_ignore_ascii_case(&field_value)),
            _ => false,
        };
        if matched {
            let tag_name = tag_map.get(rule.target_tag_id.as_str()).copied().unwrap_or(&rule.target_tag_id);
            return Ok(Some(format!("{} (via {} {})", tag_name, rule.match_type, rule.match_field)));
        }
    }
    Ok(None)
}
