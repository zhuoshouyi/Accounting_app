// 设置相关 Tauri 命令

use std::collections::HashMap;
use tauri::State;

use crate::dao::settings_dao;
use crate::db::DatabaseState;

/// 获取所有设置
#[tauri::command]
pub fn get_all_settings(
    state: State<'_, DatabaseState>,
) -> Result<HashMap<String, String>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    settings_dao::get_all_settings(&conn)
}

/// 保存单条设置
#[tauri::command]
pub fn save_setting(
    state: State<'_, DatabaseState>,
    key: String,
    value: String,
) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    settings_dao::save_setting(&conn, &key, &value)
}
