// 消费标签 DAO
// 提供对 category_tags 表的查询操作

use rusqlite::{params, Connection, Row};

use crate::models::category_tag::CategoryTag;

/// 将数据库行映射为 CategoryTag 结构体
fn row_to_category_tag(row: &Row) -> rusqlite::Result<CategoryTag> {
    Ok(CategoryTag {
        id: row.get("id")?,
        name: row.get("name")?,
        is_system: row.get("is_system")?,
        sort_order: row.get("sort_order")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

/// 查询所有消费标签，按 sort_order 升序排列
///
/// # 参数
/// - `conn`: 数据库连接
///
/// # 返回
/// 标签列表（按 sort_order 排序）
pub fn list_all_tags(conn: &Connection) -> Result<Vec<CategoryTag>, String> {
    let mut stmt = conn
        .prepare("SELECT * FROM category_tags ORDER BY sort_order ASC")
        .map_err(|e| format!("准备查询失败: {}", e))?;
    let tags = stmt
        .query_map([], row_to_category_tag)
        .map_err(|e| format!("查询标签失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("映射标签失败: {}", e))?;
    Ok(tags)
}

/// 根据 ID 查询单个标签
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 标签 ID
///
/// # 返回
/// 标签（不存在时返回 None）
pub fn get_tag_by_id(conn: &Connection, id: &str) -> Result<Option<CategoryTag>, String> {
    let mut stmt = conn
        .prepare("SELECT * FROM category_tags WHERE id = ?")
        .map_err(|e| format!("准备查询失败: {}", e))?;
    let mut rows = stmt
        .query(params![id])
        .map_err(|e| format!("查询标签失败: {}", e))?;

    match rows.next() {
        Ok(Some(row)) => {
            let tag = row_to_category_tag(row).map_err(|e| format!("映射标签失败: {}", e))?;
            Ok(Some(tag))
        }
        Ok(None) => Ok(None),
        Err(e) => Err(format!("查询标签失败: {}", e)),
    }
}

// ====================================================================
// Step 9 新增：标签 CRUD
// ====================================================================

pub fn create_tag(conn: &Connection, name: &str, sort_order: i64) -> Result<CategoryTag, String> {
    let id = format!("tag_{}", uuid::Uuid::new_v4().to_string().replace("-", "").chars().take(8).collect::<String>());
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    conn.execute(
        "INSERT INTO category_tags (id, name, is_system, sort_order, created_at, updated_at) VALUES (?, ?, 0, ?, ?, ?)",
        params![id, name, sort_order, now, now],
    ).map_err(|e| format!("创建标签失败: {}", e))?;
    Ok(CategoryTag { id, name: name.to_string(), is_system: 0, sort_order, created_at: now.clone(), updated_at: now })
}

pub fn update_tag(conn: &Connection, id: &str, name: &str, sort_order: i64) -> Result<(), String> {
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    conn.execute("UPDATE category_tags SET name = ?, sort_order = ?, updated_at = ? WHERE id = ?", params![name, sort_order, now, id])
        .map_err(|e| format!("更新标签失败: {}", e))?;
    Ok(())
}

pub fn delete_tag(conn: &Connection, id: &str) -> Result<(), String> {
    let is_system: i64 = conn.query_row("SELECT is_system FROM category_tags WHERE id = ?", params![id], |row| row.get(0))
        .map_err(|e| format!("查询标签失败: {}", e))?;
    if is_system == 1 {
        return Err("系统内置标签不可删除".to_string());
    }
    conn.execute("DELETE FROM category_tags WHERE id = ?", params![id])
        .map_err(|e| format!("删除标签失败: {}", e))?;
    Ok(())
}
