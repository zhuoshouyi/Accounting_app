// AI 服务 — 编排 DeepSeek 调用 + 学习规则管理

use rusqlite::Connection;

use crate::ai::deepseek_provider::DeepSeekProvider;
use crate::ai::provider::{AiClassifyResult, AiGenerateResult, AiProvider, ClassifyInput};
use crate::dao;
use crate::models::ai_learning_rule::AiLearningRule;
use crate::models::transaction::Transaction;

/// 构建 DeepSeek provider（从 app_settings 读取配置）
fn build_provider(conn: &Connection) -> Result<DeepSeekProvider, String> {
    let settings = dao::settings_dao::get_all_settings(conn)?;
    let api_key = settings.get("ai_api_key").cloned().unwrap_or_default();
    let base_url = settings
        .get("ai_base_url")
        .cloned()
        .unwrap_or_else(|| "https://api.deepseek.com".to_string());
    let model = settings
        .get("ai_model")
        .cloned()
        .unwrap_or_else(|| "deepseek-chat".to_string());
    let enabled = settings
        .get("ai_enabled")
        .cloned()
        .unwrap_or_else(|| "false".to_string());

    if enabled != "true" || api_key.is_empty() {
        return Err("AI 未启用或未配置 API Key".to_string());
    }

    Ok(DeepSeekProvider::new(api_key, base_url, model))
}

/// AI 辅助分类（对未分类交易调用 DeepSeek）
///
/// 返回每条交易的 AI 建议（tag_name + confidence + reason）
pub async fn ai_classify_transactions(
    conn: &Connection,
) -> Result<Vec<(String, AiClassifyResult)>, String> {
    let provider = build_provider(conn)?;

    // 查询未分类且未排除的交易
    let unclassified = dao::transaction_dao::list_unclassified_transactions(conn)?;
    if unclassified.is_empty() {
        return Ok(Vec::new());
    }

    // 获取标签列表
    let tags = dao::tag_dao::list_all_tags(conn)?;
    let tag_names: Vec<String> = tags.iter().map(|t| t.name.clone()).collect();

    // 批量分类（每次最多 20 条）
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

        for (tx, result) in chunk.iter().zip(batch_results.into_iter()) {
            if !result.tag_name.is_empty() {
                // 查找 tag_id
                let tag_id = tags.iter().find(|t| t.name == result.tag_name).map(|t| t.id.clone());
                if let Some(ref tid) = tag_id {
                    // 写入 ai_suggested_tag
                    dao::transaction_dao::update_ai_suggestion(
                        conn,
                        &tx.id,
                        tid,
                        result.confidence,
                    )?;
                }
            }
            results.push((tx.id.clone(), result));
        }
    }

    Ok(results)
}

/// AI 月度分析
pub async fn ai_analyze_monthly(
    conn: &Connection,
    month: &str,
) -> Result<AiGenerateResult, String> {
    let provider = build_provider(conn)?;

    // 构建分类明细文本
    let summary = crate::services::summary_service::get_monthly_summary(conn, month)?;
    let breakdown: Vec<String> = summary
        .categories
        .iter()
        .map(|c| format!("{}: ¥{:.2}", c.summary_category, c.total_amount))
        .collect();
    let category_breakdown = breakdown.join("\n");

    provider
        .analyze_monthly(month, summary.total_expense, &category_breakdown)
        .await
}

/// AI 生成 HTML 报表
pub async fn ai_generate_report(
    conn: &Connection,
    month: &str,
) -> Result<AiGenerateResult, String> {
    let provider = build_provider(conn)?;

    let summary = crate::services::summary_service::get_monthly_summary(conn, month)?;
    let summary_json = serde_json::to_string(&summary).map_err(|e| format!("序列化失败: {}", e))?;

    provider.generate_report_html(month, &summary_json).await
}

/// 测试 AI 连接
pub async fn test_ai_connection(conn: &Connection) -> Result<bool, String> {
    let provider = build_provider(conn)?;
    provider.test_connection().await
}

/// 生成 AI 学习规则（用户确认或修正分类后调用）
pub fn learn_from_correction(
    conn: &Connection,
    transaction: &Transaction,
    tag_id: &str,
    source: &str, // "ai_confirmed" | "ai_corrected" | "manual"
) -> Result<(), String> {
    // 从交易中提取最有区分度的字段（优先 counterparty）
    let (match_field, match_value) = if let Some(ref cp) = transaction.counterparty {
        if !cp.is_empty() {
            ("counterparty".to_string(), cp.clone())
        } else if let Some(ref p) = transaction.product {
            ("product".to_string(), p.clone())
        } else {
            return Ok(()); // 无有效字段，不生成规则
        }
    } else if let Some(ref p) = transaction.product {
        ("product".to_string(), p.clone())
    } else {
        return Ok(());
    };

    // 检查是否已有相同规则
    let existing_rules = dao::rule_dao::list_enabled_ai_rules(conn)?;
    let existing = existing_rules.iter().find(|r| {
        r.match_field == match_field
            && r.match_value == match_value
            && r.target_tag_id == tag_id
    });

    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();

    match existing {
        Some(rule) => {
            // 更新已有规则
            let confirm_count = if source == "ai_corrected" {
                rule.confirm_count
            } else {
                rule.confirm_count + 1
            };
            let correct_count = if source == "ai_corrected" {
                rule.correct_count + 1
            } else {
                rule.correct_count
            };
            let total = confirm_count + correct_count;
            let confidence = if total > 0 {
                confirm_count as f64 / total as f64
            } else {
                0.5
            };

            conn.execute(
                "UPDATE ai_learning_rules SET confidence = ?, confirm_count = ?, correct_count = ?, updated_at = ? WHERE id = ?",
                rusqlite::params![confidence, confirm_count, correct_count, now, rule.id],
            )
            .map_err(|e| format!("更新学习规则失败: {}", e))?;
        }
        None => {
            // 创建新规则
            let id = uuid::Uuid::new_v4().to_string();
            let initial_confidence = if source == "ai_corrected" { 0.3 } else { 0.6 };
            let confirm_count: i64 = if source == "ai_corrected" { 0 } else { 1 };
            let correct_count: i64 = if source == "ai_corrected" { 1 } else { 0 };

            conn.execute(
                r#"INSERT INTO ai_learning_rules (
                    id, match_field, match_value, match_type, target_tag_id,
                    confidence, confirm_count, correct_count, source, enabled,
                    created_at, updated_at
                ) VALUES (?, ?, ?, 'exact', ?, ?, ?, ?, ?, 1, ?, ?)"#,
                rusqlite::params![
                    id,
                    match_field,
                    match_value,
                    tag_id,
                    initial_confidence,
                    confirm_count,
                    correct_count,
                    source,
                    now,
                    now
                ],
            )
            .map_err(|e| format!("创建学习规则失败: {}", e))?;
        }
    }

    Ok(())
}

/// 简化版学习函数（直接传字段，不需要完整 Transaction）
pub fn learn_from_correction_simple(
    conn: &Connection,
    counterparty: &str,
    product: &str,
    transaction_type: &str,
    amount: f64,
    tag_id: &str,
    source: &str,
) -> Result<(), String> {
    let (match_field, match_value) = if !counterparty.is_empty() {
        ("counterparty".to_string(), counterparty.to_string())
    } else if !product.is_empty() {
        ("product".to_string(), product.to_string())
    } else {
        return Ok(());
    };

    let existing_rules = crate::dao::rule_dao::list_enabled_ai_rules(conn)?;
    let existing = existing_rules.iter().find(|r| {
        r.match_field == match_field
            && r.match_value == match_value
            && r.target_tag_id == tag_id
    });

    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();

    if let Some(rule) = existing {
        let confirm_count = if source == "ai_corrected" { rule.confirm_count } else { rule.confirm_count + 1 };
        let correct_count = if source == "ai_corrected" { rule.correct_count + 1 } else { rule.correct_count };
        let total = confirm_count + correct_count;
        let confidence = if total > 0 { confirm_count as f64 / total as f64 } else { 0.5 };
        conn.execute(
            "UPDATE ai_learning_rules SET confidence = ?, confirm_count = ?, correct_count = ?, updated_at = ? WHERE id = ?",
            rusqlite::params![confidence, confirm_count, correct_count, now, rule.id],
        ).map_err(|e| format!("更新学习规则失败: {}", e))?;
    } else {
        let id = uuid::Uuid::new_v4().to_string();
        let initial_confidence = if source == "ai_corrected" { 0.3 } else { 0.6 };
        let confirm_count: i64 = if source == "ai_corrected" { 0 } else { 1 };
        let correct_count: i64 = if source == "ai_corrected" { 1 } else { 0 };
        conn.execute(
            "INSERT INTO ai_learning_rules (id, match_field, match_value, match_type, target_tag_id, confidence, confirm_count, correct_count, source, enabled, counterparty, product, transaction_type, amount, created_at, updated_at) VALUES (?, ?, ?, 'exact', ?, ?, ?, ?, ?, 1, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![id, match_field, match_value, tag_id, initial_confidence, confirm_count, correct_count, source, counterparty, product, transaction_type, amount, now, now],
        ).map_err(|e| format!("创建学习规则失败: {}", e))?;
    }
    Ok(())
}
