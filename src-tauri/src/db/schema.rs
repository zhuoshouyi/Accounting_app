/// 数据库建表 SQL 常量
/// 通过 include_str! 嵌入 001_init.sql 迁移文件
/// CREATE TABLE IF NOT EXISTS 保证幂等执行

/// 完整建表 SQL（10 张表 + 索引）
pub const SCHEMA_SQL: &str = include_str!("../../migrations/001_init.sql");
