// 导入记录 DAO — 创建和查询

use rusqlite::{params, Connection};

use crate::models::import_record::ImportRecord;

/// 插入导入记录
pub fn create_import_record(conn: &Connection, record: &ImportRecord) -> Result<(), String> {
    conn.execute(
        r#"INSERT INTO import_records (
            id, month, source, payer, file_name, file_hash,
            account_info, total_count, valid_count, filtered_count, imported_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        params![
            record.id,
            record.month,
            record.source,
            record.payer,
            record.file_name,
            record.file_hash,
            record.account_info,
            record.total_count,
            record.valid_count,
            record.filtered_count,
            record.imported_at,
        ],
    )
    .map_err(|e| format!("插入导入记录失败: {}", e))?;

    Ok(())
}

/// 查询所有导入记录，按导入时间倒序
pub fn list_import_records(conn: &Connection) -> Vec<ImportRecord> {
    let mut stmt = match conn.prepare(
        r#"SELECT id, month, source, payer, file_name, file_hash,
                  account_info, total_count, valid_count, filtered_count, imported_at
           FROM import_records
           ORDER BY imported_at DESC"#,
    ) {
        Ok(stmt) => stmt,
        Err(e) => {
            eprintln!("[DAO] 查询导入记录失败: {}", e);
            return Vec::new();
        }
    };

    let rows = stmt.query_map([], |row| {
        Ok(ImportRecord {
            id: row.get(0)?,
            month: row.get(1)?,
            source: row.get(2)?,
            payer: row.get(3)?,
            file_name: row.get(4)?,
            file_hash: row.get(5)?,
            account_info: row.get(6)?,
            total_count: row.get(7)?,
            valid_count: row.get(8)?,
            filtered_count: row.get(9)?,
            imported_at: row.get(10)?,
        })
    });

    match rows {
        Ok(iter) => iter.filter_map(|r| r.ok()).collect(),
        Err(e) => {
            eprintln!("[DAO] 读取导入记录行失败: {}", e);
            Vec::new()
        }
    }
}

/// 检查文件哈希是否已存在（用于去重）
pub fn check_file_hash_exists(conn: &Connection, file_hash: &str) -> bool {
    conn.query_row(
        "SELECT COUNT(*) > 0 FROM import_records WHERE file_hash = ?",
        params![file_hash],
        |row| row.get(0),
    )
    .unwrap_or(false)
}
