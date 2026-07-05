use serde::{Deserialize, Serialize};

/// AI 分类建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiClassifyResult {
    pub tag_name: String,
    pub confidence: f64,
    pub reason: String,
}

/// AI 分析/报表生成结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiGenerateResult {
    pub content: String,
    pub model_name: String,
}

/// AI 服务商抽象 trait
#[async_trait::async_trait]
pub trait AiProvider: Send + Sync {
    /// 分类单条交易，返回建议标签 + 置信度
    async fn classify(
        &self,
        counterparty: &str,
        product: &str,
        transaction_type: &str,
        amount: f64,
        tag_list: &[String],
    ) -> Result<AiClassifyResult, String>;

    /// 批量分类（一次请求多条）
    async fn classify_batch(
        &self,
        transactions: &[ClassifyInput],
        tag_list: &[String],
    ) -> Result<Vec<AiClassifyResult>, String>;

    /// 月度分析
    async fn analyze_monthly(
        &self,
        month: &str,
        total_expense: f64,
        category_breakdown: &str,
    ) -> Result<AiGenerateResult, String>;

    /// 生成 HTML 报表
    async fn generate_report_html(
        &self,
        month: &str,
        summary_json: &str,
    ) -> Result<AiGenerateResult, String>;

    /// 测试连接是否可用
    async fn test_connection(&self) -> Result<bool, String>;
    /// 生成图表配置 JSON
    async fn generate_chart(&self, system_prompt: &str, user_message: &str) -> Result<String, String>;
}

/// 单条交易分类输入
#[derive(Debug, Clone)]
pub struct ClassifyInput {
    pub counterparty: String,
    pub product: String,
    pub transaction_type: String,
    pub amount: f64,
}
