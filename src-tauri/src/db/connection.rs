/// SQLite 连接管理模块
/// 负责数据库文件的创建、连接初始化、迁移执行

use std::path::PathBuf;
use std::sync::Mutex;
use rusqlite::Connection;

use super::migrations;
use super::schema;

/// 全局数据库状态（通过 Tauri State 管理）
pub struct DatabaseState {
    pub conn: Mutex<Connection>,
}

/// 获取数据库文件路径
/// macOS: ~/Library/Application Support/family-accounting-app/database.db
pub fn get_db_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home)
        .join("Library")
        .join("Application Support")
        .join("family-accounting-app")
        .join("database.db")
}

/// 初始化数据库
/// 1. 创建数据库目录（如果不存在）
/// 2. 打开/创建数据库文件
/// 3. 执行建表 SQL（幂等）
/// 4. 插入初始数据（仅首次创建时）
/// 5. 返回 DatabaseState 供 Tauri 管理
pub fn init_database() -> Result<DatabaseState, Box<dyn std::error::Error>> {
    let db_path = get_db_path();

    // 创建父目录
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // 检查是否为新建数据库
    let is_new_db = !db_path.exists();

    // 打开/创建数据库连接
    let conn = Connection::open(&db_path)?;

    // 启用外键约束
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    // 执行建表 SQL（CREATE TABLE IF NOT EXISTS 幂等）
    conn.execute_batch(schema::SCHEMA_SQL)?;

    // 迁移：添加 details_json 列（如不存在）
    conn.execute_batch("ALTER TABLE monthly_manual_data ADD COLUMN details_json TEXT;").ok();
    // 迁移：AI 学习规则添加完整交易信息列
    conn.execute_batch("ALTER TABLE ai_learning_rules ADD COLUMN counterparty TEXT;").ok();
    conn.execute_batch("ALTER TABLE ai_learning_rules ADD COLUMN product TEXT;").ok();
    conn.execute_batch("ALTER TABLE ai_learning_rules ADD COLUMN transaction_type TEXT;").ok();
    conn.execute_batch("ALTER TABLE ai_learning_rules ADD COLUMN amount REAL;").ok();

    // 仅在首次创建时插入初始数据
    if is_new_db {
        migrations::insert_initial_data(&conn)?;
        println!("[DB] 初始数据插入完成（新建数据库）");
    } else {
        // 已存在的数据库，检查并补充缺失的初始数据
        migrations::ensure_initial_data(&conn)?;
    }

    Ok(DatabaseState {
        conn: Mutex::new(conn),
    })
}
