use serde::{Deserialize, Serialize};

/// 交易记录模型
/// 对应数据库 transactions 表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// 主键 UUID v4
    pub id: String,
    /// 交易时间 ISO 8601（如 "2026-06-30T16:51:40"）
    pub transaction_time: String,
    /// 消费标签 ID（关联 category_tags）
    pub tag_id: Option<String>,
    /// 标签来源（rule / ai / manual）
    pub tag_source: Option<String>,
    /// 数据来源（wechat / alipay / manual）
    pub source: String,
    /// 交易类型（如 "扫二维码付款"）
    pub transaction_type: Option<String>,
    /// 交易对方
    pub counterparty: Option<String>,
    /// 商品说明
    pub product: Option<String>,
    /// 收支方向（income / expense / neutral）
    pub direction: String,
    /// 金额（元）
    pub amount: f64,
    /// 支付方式（如 "东莞银行储蓄卡(6318)"）
    pub payment_method: Option<String>,
    /// 交易状态（如 "已转账"）
    pub status: Option<String>,
    /// 归属人（关联 account_owners.name）
    pub payer: Option<String>,
    /// 归属人来源（manual / rule）
    pub payer_source: Option<String>,
    /// 是否为刚性支出（0/1/NULL）
    pub is_rigid: Option<i64>,
    /// 是否排除出汇总（0/1）
    pub is_excluded_from_summary: i64,
    /// AI 建议标签 ID
    pub ai_suggested_tag: Option<String>,
    /// AI 置信度
    pub ai_confidence: Option<f64>,
    /// 支付用途
    pub payment_purpose: Option<String>,
    /// 月份（如 "2026-06"）
    pub month: String,
    /// 原始数据 JSON 字符串
    pub raw_data: Option<String>,
    /// 创建时间 ISO 8601
    pub created_at: String,
    /// 更新时间 ISO 8601
    pub updated_at: String,
}
