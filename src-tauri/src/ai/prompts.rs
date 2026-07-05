use super::provider::ClassifyInput;

// ====================================================================
// System Prompts
// ====================================================================

pub const CLASSIFY_SYSTEM: &str = "你是一个家庭记账分类助手。请根据交易信息从给定标签列表中选择最合适的分类。只返回 JSON，不要其他内容。";

pub const ANALYSIS_SYSTEM: &str = "你是一个家庭财务分析助手。请根据提供的月度消费数据生成简洁的分析报告。用中文输出，300字以内。";

pub const REPORT_SYSTEM: &str = "你是一个家庭财务报表生成助手。请根据数据生成美观的 HTML 报表页面。CSS 内联，不依赖外部资源，不包含 JS 代码，使用中文。";

// ====================================================================
// 分类 Prompt
// ====================================================================

pub fn classify_single(
    counterparty: &str,
    product: &str,
    transaction_type: &str,
    amount: f64,
    tag_list: &[String],
) -> String {
    format!(
        r#"交易信息：
- 交易对方：{}
- 商品：{}
- 交易类型：{}
- 金额：{:.2}元

可选标签：{}

请返回 JSON：
{{"tag": "标签名称", "confidence": 0.0-1.0, "reason": "简短理由"}}

无法确定时返回：{{"tag": "", "confidence": 0.0, "reason": "无法分类"}}"#,
        counterparty,
        product,
        transaction_type,
        amount,
        tag_list.join("、")
    )
}

pub fn classify_batch(transactions: &[ClassifyInput], tag_list: &[String]) -> String {
    let items: Vec<String> = transactions
        .iter()
        .enumerate()
        .map(|(i, tx)| {
            format!(
                "{}. 对方={} 商品={} 类型={} 金额={:.2}",
                i + 1,
                tx.counterparty,
                tx.product,
                tx.transaction_type,
                tx.amount
            )
        })
        .collect();

    format!(
        r#"请为以下{}条交易分别选择标签（可选: {}）。

{}

返回 JSON 数组：
[{{"tag": "标签名", "confidence": 0.0-1.0, "reason": "理由"}}, ...]"#,
        transactions.len(),
        tag_list.join("、"),
        items.join("\n")
    )
}

// ====================================================================
// 月度分析 Prompt
// ====================================================================

pub fn analyze_monthly(month: &str, total_expense: f64, category_breakdown: &str) -> String {
    format!(
        r#"{} 月度消费数据：

总支出：{:.2}元

各分类支出：
{}

请分析：
1. 总体消费水平
2. 异常支出提醒
3. 改善建议"#,
        month, total_expense, category_breakdown
    )
}

// ====================================================================
// HTML 报表 Prompt
// ====================================================================

pub fn generate_report(month: &str, summary_json: &str) -> String {
    format!(
        r#"{} 月度家庭财务报表数据：
{}

要求生成完整 HTML 页面：
1. CSS 内联，不依赖外部资源
2. 包含月度收支汇总表、分类占比（CSS 实现横向条形图）
3. 风格简洁美观，适合家庭使用
4. 不包含任何 JavaScript
5. 使用中文
6. 标题为"{} 家庭收支报表"

直接返回 HTML 代码。"#,
        month, summary_json, month
    )
}
