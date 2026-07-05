use serde::{Deserialize, Serialize};

/// 分类规则模型
/// 对应数据库 category_rules 表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryRule {
    /// 主键（如 "rule_001"）
    pub id: String,
    /// 匹配字段（counterparty / product / transaction_type）
    pub match_field: String,
    /// 匹配方式（exact / like / in）
    pub match_type: String,
    /// 匹配值
    pub match_value: String,
    /// 目标标签 ID（关联 category_tags）
    pub target_tag_id: String,
    /// 优先级（数值越小优先级越高）
    pub priority: i64,
    /// 是否启用（1=启用，0=禁用）
    pub enabled: i64,
    /// 来源（builtin / user / ai_learned）
    pub source: String,
    /// 创建时间 ISO 8601
    pub created_at: String,
    /// 更新时间 ISO 8601
    pub updated_at: String,
}
