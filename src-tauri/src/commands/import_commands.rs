// 账单导入 Tauri 命令
// 供前端通过 invoke 调用

use tauri::State;

use crate::db::DatabaseState;
use crate::dao::import_dao;
use crate::models::import_record::ImportRecord;
use crate::services::import_service;
use crate::services::ImportResult;

/// 导入账单文件
///
/// # 参数
/// - `filePath`: 文件路径
/// - `source`: 数据来源（"wechat" 或 "alipay"）
/// - `payer`: 归属人名称（可选，为 null 时不设置归属人）
#[tauri::command]
pub fn import_bill_file(
    state: State<'_, DatabaseState>,
    file_path: String,
    source: String,
    payer: Option<String>,
) -> Result<ImportResult, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    import_service::import_bill(&file_path, &source, payer, &conn)
}

/// 查询所有导入记录（按导入时间倒序）
#[tauri::command]
pub fn list_import_records(state: State<'_, DatabaseState>) -> Result<Vec<ImportRecord>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    Ok(import_dao::list_import_records(&conn))
}

/// 检查文件是否已导入过（通过 SHA-256 哈希去重）
///
/// # 参数
/// - `filePath`: 文件路径
///
/// # 返回
/// true 表示已导入过，false 表示未导入过
#[tauri::command]
pub fn check_duplicate_import(
    state: State<'_, DatabaseState>,
    file_path: String,
) -> Result<bool, String> {
    // 计算文件哈希
    let file_hash = import_service::compute_file_hash(&file_path)?;

    // 查询数据库
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    Ok(import_dao::check_file_hash_exists(&conn, &file_hash))
}
