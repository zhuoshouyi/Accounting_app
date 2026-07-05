use reqwest::Client;
use serde_json::json;

use super::provider::{AiClassifyResult, AiGenerateResult, AiProvider, ClassifyInput};
use super::prompts;

/// DeepSeek API 适配器
pub struct DeepSeekProvider {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
}

impl DeepSeekProvider {
    pub fn new(api_key: String, base_url: String, model: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url,
            model,
        }
    }

    async fn call(&self, system_prompt: &str, user_message: &str) -> Result<String, String> {
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let body = json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_message}
            ],
            "temperature": 0.3,
            "max_tokens": 4096,
        });

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("DeepSeek 请求失败: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("DeepSeek API 错误 ({}): {}", status, text));
        }

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("解析响应失败: {}", e))?;

        data["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "响应格式异常: 无 content".to_string())
    }
}

#[async_trait::async_trait]
impl AiProvider for DeepSeekProvider {
    async fn classify(
        &self,
        counterparty: &str,
        product: &str,
        transaction_type: &str,
        amount: f64,
        tag_list: &[String],
    ) -> Result<AiClassifyResult, String> {
        let user_msg = prompts::classify_single(
            counterparty,
            product,
            transaction_type,
            amount,
            tag_list,
        );
        let content = self.call(prompts::CLASSIFY_SYSTEM, &user_msg).await?;
        parse_classify_result(&content)
    }

    async fn classify_batch(
        &self,
        transactions: &[ClassifyInput],
        tag_list: &[String],
    ) -> Result<Vec<AiClassifyResult>, String> {
        let user_msg = prompts::classify_batch(transactions, tag_list);
        let content = self.call(prompts::CLASSIFY_SYSTEM, &user_msg).await?;

        // 解析 JSON 数组
        let parsed: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| format!("解析 AI 响应失败: {}", e))?;

        let arr = parsed
            .as_array()
            .ok_or_else(|| "AI 响应不是数组格式".to_string())?;

        arr.iter()
            .map(|v| {
                Ok(AiClassifyResult {
                    tag_name: v["tag"].as_str().unwrap_or("").to_string(),
                    confidence: v["confidence"].as_f64().unwrap_or(0.0),
                    reason: v["reason"].as_str().unwrap_or("").to_string(),
                })
            })
            .collect()
    }

    async fn analyze_monthly(
        &self,
        month: &str,
        total_expense: f64,
        category_breakdown: &str,
    ) -> Result<AiGenerateResult, String> {
        let user_msg = prompts::analyze_monthly(month, total_expense, category_breakdown);
        let content = self.call(prompts::ANALYSIS_SYSTEM, &user_msg).await?;
        Ok(AiGenerateResult {
            content,
            model_name: self.model.clone(),
        })
    }

    async fn generate_report_html(
        &self,
        month: &str,
        summary_json: &str,
    ) -> Result<AiGenerateResult, String> {
        let user_msg = prompts::generate_report(month, summary_json);
        let content = self.call(prompts::REPORT_SYSTEM, &user_msg).await?;

        // 提取 HTML（去掉可能的 markdown 包裹）
        let html = if content.contains("```html") {
            content
                .split("```html")
                .nth(1)
                .and_then(|s| s.split("```").next())
                .unwrap_or(&content)
                .to_string()
        } else if content.contains("```") {
            content
                .split("```")
                .nth(1)
                .unwrap_or(&content)
                .to_string()
        } else {
            content
        };

        // 安全过滤
        let safe_html = sanitize_html(&html);

        Ok(AiGenerateResult {
            content: safe_html,
            model_name: self.model.clone(),
        })
    }

    async fn test_connection(&self) -> Result<bool, String> {
        let user_msg = "请回复 OK（只回复这两个字母）。";
        let content = self.call("你是一个连接测试。", user_msg).await?;
        Ok(content.contains("OK"))
    }

    async fn generate_chart(&self, system_prompt: &str, user_message: &str) -> Result<String, String> {
        let content = self.call(system_prompt, user_message).await?;
        // 提取 JSON
        let json = if content.contains("```json") {
            content.split("```json").nth(1).and_then(|s| s.split("```").next()).unwrap_or(&content).to_string()
        } else if content.contains("```") {
            content.split("```").nth(1).unwrap_or(&content).to_string()
        } else {
            content
        };
        Ok(json.trim().to_string())
    }
}

// ====================================================================
// 辅助函数
// ====================================================================

/// 解析 AI 分类结果 JSON
fn parse_classify_result(content: &str) -> Result<AiClassifyResult, String> {
    // 提取 JSON（可能包裹在 ```json ``` 中）
    let json_str = if content.contains("```json") {
        content
            .split("```json")
            .nth(1)
            .and_then(|s| s.split("```").next())
            .unwrap_or(content)
    } else if content.contains("```") {
        content
            .split("```")
            .nth(1)
            .unwrap_or(content)
    } else {
        content
    };

    let v: serde_json::Value =
        serde_json::from_str(json_str).map_err(|e| format!("解析 AI 分类结果失败: {}", e))?;

    Ok(AiClassifyResult {
        tag_name: v["tag"].as_str().unwrap_or("").to_string(),
        confidence: v["confidence"].as_f64().unwrap_or(0.0),
        reason: v["reason"].as_str().unwrap_or("").to_string(),
    })
}

/// 安全过滤 HTML（移除 script/iframe/on* 事件）
fn sanitize_html(html: &str) -> String {
    html.replace("<script", "<!-- script")
        .replace("</script>", "</script-->")
        .replace("<iframe", "<!-- iframe")
        .replace("</iframe>", "</iframe-->")
        .replace("onerror=", "data-onerror=")
        .replace("onload=", "data-onload=")
        .replace("onclick=", "data-onclick=")
}
