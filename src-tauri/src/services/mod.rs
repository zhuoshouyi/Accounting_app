// 服务层模块
// 编排 DAO 和解析器，实现业务逻辑

pub mod import_service;
pub mod cleaning_service;
pub mod classification_engine;
pub mod summary_service;
pub mod ai_service;
pub mod report_service;

use serde::{Deserialize, Serialize};

/// 导入结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    /// 数据来源（wechat / alipay）
    pub source: String,
    /// 账户信息（从文件提取）
    pub account_info: String,
    /// 解析出的总交易数
    pub total_count: usize,
    /// 实际写入数据库的数
    pub imported_count: usize,
    /// 跳过的数量（如重复导入）
    pub skipped_count: usize,
    /// 涉及的月份列表（支持多月）
    pub months: Vec<String>,
    /// 归属人
    pub payer: Option<String>,
    /// 文件名
    pub file_name: String,
}
