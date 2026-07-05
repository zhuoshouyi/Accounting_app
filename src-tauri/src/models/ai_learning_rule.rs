use serde::{Deserialize, Serialize};

/// AI 学习规则模型 — 基于完整交易数据的概率学习
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiLearningRule {
    pub id: String,
    /// 匹配字段（counterparty / product / transaction_type）
    pub match_field: String,
    /// 匹配值
    pub match_value: String,
    /// 匹配方式（exact / like / in）
    pub match_type: String,
    /// 目标标签 ID
    pub target_tag_id: String,
    /// 置信度（0.0 ~ 1.0）
    pub confidence: f64,
    /// 确认次数
    pub confirm_count: i64,
    /// 正确次数
    pub correct_count: i64,
    /// 来源
    pub source: Option<String>,
    /// 是否启用
    pub enabled: i64,
    /// 原始交易对方（Step 9 新增，保留完整上下文）
    pub counterparty: Option<String>,
    /// 原始商品说明
    pub product: Option<String>,
    /// 原始交易类型
    pub transaction_type: Option<String>,
    /// 金额
    pub amount: Option<f64>,
    pub created_at: String,
    pub updated_at: String,
}
