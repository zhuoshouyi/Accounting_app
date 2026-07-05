// AI 相关 Tauri 命令

use std::collections::HashMap;
use tauri::State;

use crate::ai::deepseek_provider::DeepSeekProvider;
use crate::ai::provider::{AiClassifyResult, AiGenerateResult, AiProvider, ClassifyInput};
use crate::dao;
use crate::db::DatabaseState;

/// 从数据库读取配置，构建 DeepSeek provider
fn build_provider_from_settings(settings: &HashMap<String, String>) -> Option<DeepSeekProvider> {
    let api_key = settings.get("ai_api_key")?.clone();
    let enabled = settings.get("ai_enabled")?.clone();
    if enabled != "true" || api_key.is_empty() {
        return None;
    }
    let base_url = settings
        .get("ai_base_url")
        .cloned()
        .unwrap_or_else(|| "https://api.deepseek.com".to_string());
    let model = settings
        .get("ai_model")
        .cloned()
        .unwrap_or_else(|| "deepseek-chat".to_string());
    Some(DeepSeekProvider::new(api_key, base_url, model))
}

/// AI 辅助分类
#[tauri::command]
pub async fn ai_classify(
    state: State<'_, DatabaseState>,
) -> Result<Vec<(String, AiClassifyResult)>, String> {
    // 在锁内完成所有同步操作
    let (provider, unclassified, tag_names) = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        let settings = dao::settings_dao::get_all_settings(&conn)?;
        let provider = build_provider_from_settings(&settings)
            .ok_or_else(|| "AI 未启用或未配置 API Key".to_string())?;
        let unclassified = dao::transaction_dao::list_unclassified_transactions(&conn)?;
        let tags = dao::tag_dao::list_all_tags(&conn)?;
        let tag_names: Vec<String> = tags.iter().map(|t| t.name.clone()).collect();
        (provider, unclassified, tag_names)
    };
    // 锁已释放

    if unclassified.is_empty() {
        return Ok(Vec::new());
    }

    let mut results = Vec::new();
    for chunk in unclassified.chunks(20) {
        let inputs: Vec<ClassifyInput> = chunk
            .iter()
            .map(|tx| ClassifyInput {
                counterparty: tx.counterparty.clone().unwrap_or_default(),
                product: tx.product.clone().unwrap_or_default(),
                transaction_type: tx.transaction_type.clone().unwrap_or_default(),
                amount: tx.amount,
            })
            .collect();
        let batch_results = provider.classify_batch(&inputs, &tag_names).await?;
        results.push((chunk.to_vec(), batch_results));
    }

    // 写回数据库需要在锁内
    let mut output = Vec::new();
    {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        let tags = dao::tag_dao::list_all_tags(&conn)?;
        for (chunk, batch_results) in results {
            for (tx, result) in chunk.iter().zip(batch_results.into_iter()) {
                if !result.tag_name.is_empty() {
                    if let Some(tag) = tags.iter().find(|t| t.name == result.tag_name) {
                        dao::transaction_dao::update_ai_suggestion(
                            &conn, &tx.id, &tag.id, result.confidence,
                        )?;
                    }
                }
                output.push((tx.id.clone(), result));
            }
        }
    }

    Ok(output)
}

/// AI 月度分析
#[tauri::command]
pub async fn ai_analyze(
    state: State<'_, DatabaseState>,
    month: String,
) -> Result<AiGenerateResult, String> {
    let (provider, category_breakdown, total_expense) = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        let settings = dao::settings_dao::get_all_settings(&conn)?;
        let provider = build_provider_from_settings(&settings)
            .ok_or_else(|| "AI 未启用或未配置 API Key".to_string())?;
        let summary = crate::services::summary_service::get_monthly_summary(&conn, &month)?;
        let breakdown: Vec<String> = summary
            .categories
            .iter()
            .map(|c| format!("{}: ¥{:.2}", c.summary_category, c.total_amount))
            .collect();
        (provider, breakdown.join("\n"), summary.total_expense)
    };

    provider
        .analyze_monthly(&month, total_expense, &category_breakdown)
        .await
}

/// AI 生成 HTML 报表
#[tauri::command]
pub async fn ai_generate_report(
    state: State<'_, DatabaseState>,
    month: String,
) -> Result<AiGenerateResult, String> {
    let (provider, summary_json) = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        let settings = dao::settings_dao::get_all_settings(&conn)?;
        let provider = build_provider_from_settings(&settings)
            .ok_or_else(|| "AI 未启用或未配置 API Key".to_string())?;
        let summary = crate::services::summary_service::get_monthly_summary(&conn, &month)?;
        let json = serde_json::to_string(&summary).map_err(|e| format!("序列化失败: {}", e))?;
        (provider, json)
    };

    provider.generate_report_html(&month, &summary_json).await
}

/// AI 生成图表（根据用户描述返回结构化 JSON 配置）
#[tauri::command]
pub async fn ai_generate_chart(
    state: State<'_, DatabaseState>,
    prompt: String,
) -> Result<String, String> {
    let (provider, data_json) = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        let settings = dao::settings_dao::get_all_settings(&conn)?;
        let provider = build_provider_from_settings(&settings)
            .ok_or_else(|| "AI 未启用或未配置 API Key".to_string())?;
        // 获取全部月份的汇总数据供 AI 参考
        let rows = crate::services::summary_service::get_all_months_summary(&conn)?;
        let data = rows.iter().map(|r| {
            serde_json::json!({
                "month": r.month,
                "total_expense": r.total_expense,
                "categories": r.categories.iter().map(|c| {
                    serde_json::json!({ "name": c.summary_category, "amount": c.total_amount })
                }).collect::<Vec<_>>(),
                "total_assets": r.total_assets,
                "joey_income": r.joey_income,
                "vila_income": r.vila_income,
            })
        }).collect::<Vec<_>>();
        (provider, serde_json::to_string(&data).map_err(|e| format!("序列化失败: {}", e))?)
    };

    let system = r#"你是数据可视化助手。根据用户需求和提供的月度汇总数据，生成 Recharts 图表配置 JSON。
支持的图表类型: bar（柱状图）, line（折线图）, pie（饼图）, area（面积图）。
返回格式:
{
  "type": "bar",
  "title": "图表标题",
  "data": [{"name": "项目名", "value": 数值, "category": "分类", "month": "2026-06"}, ...],
  "xKey": "month",
  "yKey": "value",
  "groupKey": "category",
  "width": 800,
  "height": 400
}
仅返回 JSON，不要其他内容。"#;

    let user_msg = format!("用户需求：{}\n\n数据：{}", prompt, data_json);
    provider.generate_chart(system, &user_msg).await
}

/// 测试 AI 连接
#[tauri::command]
pub async fn test_ai_connection(
    state: State<'_, DatabaseState>,
) -> Result<bool, String> {
    let provider = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        let settings = dao::settings_dao::get_all_settings(&conn)?;
        build_provider_from_settings(&settings)
            .ok_or_else(|| "AI 未启用或未配置 API Key".to_string())?
    };

    provider.test_connection().await
}

/// 采纳 AI 建议：将 ai_suggested_tag 设为正式标签，生成学习规则
#[tauri::command]
pub fn confirm_ai_tag(
    state: State<'_, DatabaseState>,
    id: String,
    tag_id: String,
    counterparty: String,
    product: String,
    transaction_type: String,
    amount: f64,
) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    dao::transaction_dao::update_transaction_tag(&conn, &id, Some(&tag_id), "ai")?;
    crate::services::ai_service::learn_from_correction_simple(
        &conn, &counterparty, &product, &transaction_type, amount, &tag_id, "ai_confirmed",
    )?;
    Ok(())
}

/// 修正 AI 建议：用户选择不同标签，生成学习规则
#[tauri::command]
pub fn correct_ai_tag(
    state: State<'_, DatabaseState>,
    id: String,
    tag_id: String,
    counterparty: String,
    product: String,
    transaction_type: String,
    amount: f64,
) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    dao::transaction_dao::update_transaction_tag(&conn, &id, Some(&tag_id), "manual")?;
    crate::services::ai_service::learn_from_correction_simple(
        &conn, &counterparty, &product, &transaction_type, amount, &tag_id, "ai_corrected",
    )?;
    Ok(())
}
