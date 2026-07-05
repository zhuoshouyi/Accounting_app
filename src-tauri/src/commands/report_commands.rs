// 报表相关 Tauri 命令

use tauri::State;

use crate::db::DatabaseState;
use crate::models::ai_report::AiReport;
use crate::services::report_service::{self, TrendData};

/// 保存 AI 报表到历史
#[tauri::command]
pub fn save_report(
    state: State<'_, DatabaseState>,
    month: String,
    report_type: String,
    title: String,
    content: String,
    summary_json: String,
    model_name: String,
) -> Result<AiReport, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    report_service::save_report(&conn, &month, &report_type, &title, &content, &summary_json, &model_name)
}

/// 获取报表历史
#[tauri::command]
pub fn get_report_history(
    state: State<'_, DatabaseState>,
    month: Option<String>,
) -> Result<Vec<AiReport>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    report_service::get_report_history(&conn, month.as_deref())
}

/// 获取单个报表
#[tauri::command]
pub fn get_report_by_id(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<Option<AiReport>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    report_service::get_report_by_id(&conn, &id)
}

/// 获取多月趋势数据
#[tauri::command]
pub fn get_trend_data(
    state: State<'_, DatabaseState>,
    months: Vec<String>,
) -> Result<TrendData, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    report_service::get_trend_data(&conn, &months)
}
