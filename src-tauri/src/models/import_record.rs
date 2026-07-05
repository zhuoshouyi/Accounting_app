use serde::{Deserialize, Serialize};

/// 导入记录模型
/// 对应数据库 import_records 表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportRecord {
    /// 主键 UUID v4
    pub id: String,
    /// 月份（多月用逗号分隔，如 "2026-06,2026-07"）
    pub month: Option<String>,
    /// 数据来源（wechat / alipay）
    pub source: String,
    /// 归属人
    pub payer: Option<String>,
    /// 文件名
    pub file_name: String,
    /// 文件 SHA-256 哈希（用于去重）
    pub file_hash: Option<String>,
    /// 账户信息（从文件提取）
    pub account_info: Option<String>,
    /// 总交易数
    pub total_count: Option<i64>,
    /// 有效交易数（实际写入数据库）
    pub valid_count: Option<i64>,
    /// 过滤掉的交易数
    pub filtered_count: Option<i64>,
    /// 导入时间 ISO 8601
    pub imported_at: String,
}
