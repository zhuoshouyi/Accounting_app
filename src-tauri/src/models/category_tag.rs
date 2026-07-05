use serde::{Deserialize, Serialize};

/// 消费标签模型
/// 对应数据库 category_tags 表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryTag {
    /// 主键（如 "tag_canyin"）
    pub id: String,
    /// 标签名称（如 "餐饮"）
    pub name: String,
    /// 是否系统内置（1=系统，0=用户自定义）
    pub is_system: i64,
    /// 排序序号
    pub sort_order: i64,
    /// 创建时间 ISO 8601
    pub created_at: String,
    /// 更新时间 ISO 8601
    pub updated_at: String,
}
