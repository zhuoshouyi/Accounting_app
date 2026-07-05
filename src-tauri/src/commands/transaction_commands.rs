use tauri::State;
use crate::db::DatabaseState;
use crate::dao::manual_data_dao;
use crate::dao::transaction_dao;
use crate::models::monthly_manual_data::MonthlyManualData;

#[tauri::command]
pub fn save_manual_data(state: State<'_, DatabaseState>, month: String, total_assets: Option<f64>, joey_income: Option<f64>, vila_income: Option<f64>, mortgage_savings: Option<f64>, investment: Option<f64>, insurance: Option<f64>, analysis_text: Option<String>, details_json: Option<String>) -> Result<MonthlyManualData, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let data = MonthlyManualData { id: String::new(), month, total_assets, joey_income, vila_income, mortgage_savings, investment, insurance, analysis_text, details_json, created_at: String::new(), updated_at: String::new() };
    manual_data_dao::upsert_manual_data(&conn, &data)
}

#[tauri::command]
pub fn get_manual_data(state: State<'_, DatabaseState>, month: String) -> Result<Option<MonthlyManualData>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    manual_data_dao::get_manual_data(&conn, &month)
}

#[tauri::command]
pub fn update_transaction_payer(state: State<'_, DatabaseState>, id: String, payer: Option<String>) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    transaction_dao::update_transaction_payer(&conn, &id, payer.as_deref())
}

#[tauri::command]
pub fn update_transaction_rigid(state: State<'_, DatabaseState>, id: String, is_rigid: bool) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    transaction_dao::update_transaction_rigid(&conn, &id, is_rigid)
}

#[tauri::command]
pub fn batch_update_payer(state: State<'_, DatabaseState>, ids: Vec<String>, payer: String) -> Result<usize, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    transaction_dao::batch_update_payer(&conn, &ids, &payer)
}

#[tauri::command]
pub fn batch_update_rigid(state: State<'_, DatabaseState>, ids: Vec<String>, is_rigid: bool) -> Result<usize, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    transaction_dao::batch_update_rigid(&conn, &ids, is_rigid)
}

#[tauri::command]
pub fn create_manual_transaction(state: State<'_, DatabaseState>, transaction_time: String, amount: f64, counterparty: String, product: String, direction: String, tag_id: Option<String>, payer: Option<String>, payment_method: Option<String>) -> Result<crate::models::transaction::Transaction, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let month = if transaction_time.len() >= 7 { transaction_time[..7].to_string() } else { now[..7].to_string() };
    conn.execute("INSERT INTO transactions (id, transaction_time, tag_id, tag_source, source, transaction_type, counterparty, product, direction, amount, payment_method, status, payer, is_excluded_from_summary, month, created_at, updated_at) VALUES (?, ?, ?, 'manual', 'manual', NULL, ?, ?, ?, ?, ?, '支付成功', ?, 0, ?, ?, ?)", rusqlite::params![id, transaction_time, tag_id, counterparty, product, direction, amount, payment_method, payer, month, now, now]).map_err(|e| format!("新增交易失败: {}", e))?;
    Ok(crate::models::transaction::Transaction { id, transaction_time, tag_id, tag_source: Some("manual".into()), source: "manual".into(), transaction_type: None, counterparty: Some(counterparty), product: Some(product), direction, amount, payment_method, status: Some("支付成功".into()), payer, payer_source: None, is_rigid: None, is_excluded_from_summary: 0, ai_suggested_tag: None, ai_confidence: None, payment_purpose: None, month, raw_data: None, created_at: now.clone(), updated_at: now })
}

#[tauri::command]
pub fn batch_create_transactions(state: State<'_, DatabaseState>, items: Vec<BatchTxInput>) -> Result<usize, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let mut count = 0;
    conn.execute_batch("BEGIN").map_err(|e| format!("开启事务失败: {}", e))?;
    for item in &items {
        let id = uuid::Uuid::new_v4().to_string();
        let month = if item.transaction_time.len() >= 7 { item.transaction_time[..7].to_string() } else { now[..7].to_string() };
        conn.execute(
            "INSERT INTO transactions (id, transaction_time, tag_id, tag_source, source, counterparty, product, direction, amount, payment_method, status, payer, is_excluded_from_summary, month, created_at, updated_at) VALUES (?, ?, ?, 'manual', 'manual', ?, ?, ?, ?, ?, '支付成功', ?, 0, ?, ?, ?)",
            rusqlite::params![id, item.transaction_time, item.tag_id, item.counterparty, item.product, item.direction, item.amount, item.payment_method, item.payer, month, now, now],
        ).map_err(|e| { conn.execute_batch("ROLLBACK").ok(); format!("批量新增失败: {}", e) })?;
        count += 1;
    }
    conn.execute_batch("COMMIT").map_err(|e| format!("提交事务失败: {}", e))?;
    Ok(count)
}

#[derive(Debug, serde::Deserialize)]
pub struct BatchTxInput {
    pub transaction_time: String,
    pub amount: f64,
    pub counterparty: String,
    pub product: String,
    pub direction: String,
    pub tag_id: Option<String>,
    pub payer: Option<String>,
    pub payment_method: Option<String>,
}

#[tauri::command]
pub fn batch_delete_transactions(state: State<'_, DatabaseState>, ids: Vec<String>) -> Result<usize, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    transaction_dao::batch_delete_transactions(&conn, &ids)
}

#[tauri::command]
pub fn update_transaction_field(state: State<'_, DatabaseState>, id: String, field: String, value: String) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let sql = format!("UPDATE transactions SET {} = ?, updated_at = ? WHERE id = ?", field);
    conn.execute(&sql, rusqlite::params![value, now, id])
        .map_err(|e| format!("更新字段失败: {}", e))?;
    Ok(())
}
