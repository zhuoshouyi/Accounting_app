/// 命令模块
/// 注册 Tauri 命令处理器，供前端通过 invoke 调用

pub mod account_owner_commands;
pub mod import_commands;
pub mod cleaning_commands;
pub mod classification_commands;
pub mod summary_commands;
pub mod transaction_commands;
pub mod ai_commands;
pub mod settings_commands;
pub mod report_commands;
pub mod rule_commands;

use serde::Serialize;
use crate::db;

/// 应用信息（返回给前端）
#[derive(Debug, Serialize)]
pub struct AppInfo {
    pub app_name: String,
    pub app_version: String,
    pub db_path: String,
    pub db_exists: bool,
    pub table_count: i64,
}

/// 获取应用信息
/// 用于前端验证 Step 1 是否成功（数据库路径、表数量等）
#[tauri::command]
pub fn get_app_info() -> AppInfo {
    let db_path = db::get_db_path();
    let db_path_str = db_path.to_string_lossy().to_string();
    let db_exists = db_path.exists();

    let table_count = if db_exists {
        match rusqlite::Connection::open(&db_path) {
            Ok(conn) => {
                let count: i64 = conn
                    .query_row(
                        "SELECT COUNT(*) FROM sqlite_master WHERE type='table'",
                        [],
                        |row| row.get(0),
                    )
                    .unwrap_or(0);
                count
            }
            Err(_) => 0,
        }
    } else {
        0
    };

    AppInfo {
        app_name: env!("CARGO_PKG_NAME").to_string(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        db_path: db_path_str,
        db_exists,
        table_count,
    }
}
