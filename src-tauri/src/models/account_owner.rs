use serde::{Deserialize, Serialize};

/// 归属人模型
/// 对应数据库 account_owners 表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountOwner {
    /// 主键 UUID v4
    pub id: String,
    /// 归属人名称（唯一）
    pub name: String,
    /// 排序序号
    pub sort_order: i64,
    /// 创建时间 ISO 8601
    pub created_at: String,
    /// 更新时间 ISO 8601
    pub updated_at: String,
}
