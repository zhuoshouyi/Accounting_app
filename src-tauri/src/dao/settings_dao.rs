// 应用设置 DAO

use std::collections::HashMap;
use rusqlite::{params, Connection};

/// 获取所有设置（返回 HashMap<key, value>）
pub fn get_all_settings(conn: &Connection) -> Result<HashMap<String, String>, String> {
    let mut stmt = conn
        .prepare("SELECT key, value FROM app_settings")
        .map_err(|e| format!("查询设置失败: {}", e))?;

    let map: HashMap<String, String> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1).unwrap_or_default(),
            ))
        })
        .map_err(|e| format!("映射设置失败: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(map)
}

/// 保存设置（UPSERT）
pub fn save_setting(conn: &Connection, key: &str, value: &str) -> Result<(), String> {
    let now = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string();

    let existing: Option<String> = conn
        .query_row(
            "SELECT key FROM app_settings WHERE key = ?",
            params![key],
            |row| row.get(0),
        )
        .ok();

    match existing {
        Some(_) => {
            conn.execute(
                "UPDATE app_settings SET value = ?, updated_at = ? WHERE key = ?",
                params![value, now, key],
            )
            .map_err(|e| format!("更新设置失败: {}", e))?;
        }
        None => {
            conn.execute(
                "INSERT INTO app_settings (key, value, description, updated_at) VALUES (?, ?, '', ?)",
                params![key, value, now],
            )
            .map_err(|e| format!("插入设置失败: {}", e))?;
        }
    }

    Ok(())
}
