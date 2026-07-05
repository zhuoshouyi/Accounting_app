// 汇总映射 DAO — CRUD 操作

use rusqlite::{params, Connection};

use crate::models::summary_mapping::SummaryMapping;

/// 查询所有映射（按 sort_order 排序）
pub fn list_all(conn: &Connection) -> Result<Vec<SummaryMapping>, String> {
    let mut stmt = conn
        .prepare("SELECT * FROM summary_mappings ORDER BY sort_order ASC, summary_category ASC")
        .map_err(|e| format!("查询映射失败: {}", e))?;
    let items = stmt
        .query_map([], |row| {
            Ok(SummaryMapping {
                id: row.get("id")?,
                summary_category: row.get("summary_category")?,
                tag_id: row.get("tag_id")?,
                sort_order: row.get("sort_order")?,
                created_at: row.get("created_at")?,
            })
        })
        .map_err(|e| format!("映射映射失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("读取映射失败: {}", e))?;
    Ok(items)
}

/// 创建映射
pub fn create_mapping(conn: &Connection, summary_category: &str, tag_id: &str, sort_order: i64) -> Result<SummaryMapping, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    conn.execute(
        "INSERT INTO summary_mappings (id, summary_category, tag_id, sort_order, created_at) VALUES (?, ?, ?, ?, ?)",
        params![id, summary_category, tag_id, sort_order, now],
    )
    .map_err(|e| format!("创建映射失败: {}", e))?;
    Ok(SummaryMapping { id, summary_category: summary_category.to_string(), tag_id: tag_id.to_string(), sort_order, created_at: now })
}

/// 删除映射
pub fn delete_mapping(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM summary_mappings WHERE id = ?", params![id])
        .map_err(|e| format!("删除映射失败: {}", e))?;
    Ok(())
}
