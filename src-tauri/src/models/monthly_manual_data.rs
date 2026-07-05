use serde::{Deserialize, Serialize};

/// 月度手动数据模型
/// 对应数据库 monthly_manual_data 表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyManualData {
    /// 主键 UUID v4
    pub id: String,
    /// 月份（YYYY-MM，唯一）
    pub month: String,
    /// 总资产
    pub total_assets: Option<f64>,
    /// Joey 收入
    pub joey_income: Option<f64>,
    /// Vila 收入
    pub vila_income: Option<f64>,
    /// 房贷/存钱
    pub mortgage_savings: Option<f64>,
    /// 理财
    pub investment: Option<f64>,
    /// 保险
    pub insurance: Option<f64>,
    /// 分析（文字描述）
    pub analysis_text: Option<String>,
    /// 明细拆分 JSON
    pub details_json: Option<String>,
    /// 创建时间
    pub created_at: String,
    /// 更新时间
    pub updated_at: String,
}
