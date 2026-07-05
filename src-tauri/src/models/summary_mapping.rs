use serde::{Deserialize, Serialize};

/// 汇总映射模型
/// 对应数据库 summary_mappings 表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryMapping {
    pub id: String,
    pub summary_category: String,
    pub tag_id: String,
    pub sort_order: i64,
    pub created_at: String,
}
