// 账单解析器模块
// 支持微信 XLSX 和支付宝 CSV(GBK) 两种格式

use serde::{Deserialize, Serialize};

pub mod wechat_parser;
pub mod alipay_parser;

/// 解析后的交易记录（中间结构，不直接对应数据库表）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedTransaction {
    /// 交易时间 ISO 8601（如 "2026-06-30T16:51:40"）
    pub transaction_time: String,
    /// 交易类型
    pub transaction_type: String,
    /// 交易对方
    pub counterparty: String,
    /// 商品说明
    pub product: String,
    /// 收支方向（expense / income / neutral）
    pub direction: String,
    /// 金额（元）
    pub amount: f64,
    /// 支付方式
    pub payment_method: String,
    /// 交易状态
    pub status: String,
    /// 月份（如 "2026-06"）
    pub month: String,
    /// 原始数据 JSON 字符串
    pub raw_data: String,
}

/// 微信账单解析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WechatParseResult {
    /// 账户信息（微信昵称）
    pub account_info: String,
    /// 交易记录列表
    pub transactions: Vec<ParsedTransaction>,
}

/// 支付宝账单解析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlipayParseResult {
    /// 账户信息（姓名 + 账号）
    pub account_info: String,
    /// 交易记录列表
    pub transactions: Vec<ParsedTransaction>,
}

/// 从 ISO 8601 时间字符串中提取月份
/// "2026-06-30T16:51:40" → "2026-06"
pub fn extract_month(transaction_time: &str) -> String {
    if transaction_time.len() >= 7 {
        transaction_time[..7].to_string()
    } else {
        "unknown".to_string()
    }
}
