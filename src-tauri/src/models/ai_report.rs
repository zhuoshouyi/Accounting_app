use serde::{Deserialize, Serialize};

/// AI 报表历史模型
/// 对应数据库 ai_reports 表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiReport {
    pub id: String,
    pub month: String,
    pub report_type: String,
    pub title: Option<String>,
    pub content: Option<String>,
    pub summary_data: Option<String>,
    pub model_name: Option<String>,
    pub created_at: String,
}
