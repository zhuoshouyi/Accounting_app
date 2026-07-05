// 归属人 DAO — CRUD 操作

use rusqlite::{params, Connection};

use crate::models::account_owner::AccountOwner;

/// 查询所有归属人，按 sort_order 排序
pub fn list_owners(conn: &Connection) -> Vec<AccountOwner> {
    let mut stmt = match conn.prepare(
        "SELECT id, name, sort_order, created_at, updated_at FROM account_owners ORDER BY sort_order ASC, created_at ASC",
    ) {
        Ok(stmt) => stmt,
        Err(e) => {
            eprintln!("[DAO] 查询归属人列表失败: {}", e);
            return Vec::new();
        }
    };

    let rows = stmt.query_map([], |row| {
        Ok(AccountOwner {
            id: row.get(0)?,
            name: row.get(1)?,
            sort_order: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        })
    });

    match rows {
        Ok(iter) => iter.filter_map(|r| r.ok()).collect(),
        Err(e) => {
            eprintln!("[DAO] 读取归属人行失败: {}", e);
            Vec::new()
        }
    }
}

/// 新增归属人
/// id 使用 UUID v4，sort_order 取当前最大值 + 1
pub fn create_owner(conn: &Connection, name: &str) -> Result<AccountOwner, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();

    // 获取当前最大 sort_order
    let max_sort: i64 = conn
        .query_row("SELECT COALESCE(MAX(sort_order), 0) FROM account_owners", [], |row| {
            row.get(0)
        })
        .map_err(|e| format!("查询 sort_order 失败: {}", e))?;

    let sort_order = max_sort + 1;

    conn.execute(
        "INSERT INTO account_owners (id, name, sort_order, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
        params![id, name, sort_order, now, now],
    )
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            format!("归属人 '{}' 已存在", name)
        } else {
            format!("新增归属人失败: {}", e)
        }
    })?;

    Ok(AccountOwner {
        id,
        name: name.to_string(),
        sort_order,
        created_at: now.clone(),
        updated_at: now,
    })
}

/// 修改归属人名称
pub fn update_owner(conn: &Connection, id: &str, name: &str) -> Result<AccountOwner, String> {
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();

    let affected = conn
        .execute(
            "UPDATE account_owners SET name = ?, updated_at = ? WHERE id = ?",
            params![name, now, id],
        )
        .map_err(|e| {
            if e.to_string().contains("UNIQUE") {
                format!("归属人 '{}' 已存在", name)
            } else {
                format!("修改归属人失败: {}", e)
            }
        })?;

    if affected == 0 {
        return Err(format!("归属人 ID '{}' 不存在", id));
    }

    // 查询更新后的记录
    conn.query_row(
        "SELECT id, name, sort_order, created_at, updated_at FROM account_owners WHERE id = ?",
        params![id],
        |row| {
            Ok(AccountOwner {
                id: row.get(0)?,
                name: row.get(1)?,
                sort_order: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        },
    )
    .map_err(|e| format!("查询更新后的归属人失败: {}", e))
}

/// 删除归属人
/// 注意：如果 transactions 表有引用该归属人的记录，需要先处理
pub fn delete_owner(conn: &Connection, id: &str) -> Result<(), String> {
    // 先将引用该归属人名称的交易记录的 payer 设为 NULL
    // 查询归属人名称
    let name: String = conn
        .query_row(
            "SELECT name FROM account_owners WHERE id = ?",
            params![id],
            |row| row.get(0),
        )
        .map_err(|e| format!("查询归属人失败: {}", e))?;

    // 将引用该名称的交易记录的 payer 和 payer_source 清空
    conn.execute(
        "UPDATE transactions SET payer = NULL, payer_source = NULL WHERE payer = ?",
        params![name],
    )
    .map_err(|e| format!("清除交易引用失败: {}", e))?;

    // 删除归属人
    let affected = conn
        .execute("DELETE FROM account_owners WHERE id = ?", params![id])
        .map_err(|e| format!("删除归属人失败: {}", e))?;

    if affected == 0 {
        return Err(format!("归属人 ID '{}' 不存在", id));
    }

    Ok(())
}

/// 按名字查询归属人
#[allow(dead_code)]
pub fn get_owner_by_name(conn: &Connection, name: &str) -> Option<AccountOwner> {
    conn.query_row(
        "SELECT id, name, sort_order, created_at, updated_at FROM account_owners WHERE name = ?",
        params![name],
        |row| {
            Ok(AccountOwner {
                id: row.get(0)?,
                name: row.get(1)?,
                sort_order: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        },
    )
    .ok()
}
