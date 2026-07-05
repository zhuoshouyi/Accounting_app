// 数据清洗服务
// 实现交易数据的自动清洗规则：
//   A. 待过滤（软删除）：特定状态 + 小额支出
//   B. 待修改（部分退款）：从备注提取退款金额并调整

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::dao::transaction_dao;
use crate::models::transaction::Transaction;

// ====================================================================
// 数据结构定义
// ====================================================================

/// 待过滤的交易项（展示用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionItem {
    /// 交易 ID
    pub id: String,
    /// 交易时间
    pub transaction_time: String,
    /// 数据来源（wechat / alipay）
    pub source: String,
    /// 交易类型
    pub transaction_type: Option<String>,
    /// 交易对方
    pub counterparty: Option<String>,
    /// 商品说明
    pub product: Option<String>,
    /// 收支方向（income / expense / neutral）
    pub direction: String,
    /// 金额（元）
    pub amount: f64,
    /// 交易状态
    pub status: Option<String>,
    /// 过滤原因（如 "状态:退款成功" 或 "金额≤3元（2.50元）"）
    pub reason: String,
}

/// 待修改的交易项（部分退款处理，展示用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifyItem {
    /// 交易 ID
    pub id: String,
    /// 交易时间
    pub transaction_time: String,
    /// 数据来源
    pub source: String,
    /// 交易对方
    pub counterparty: Option<String>,
    /// 商品说明
    pub product: Option<String>,
    /// 原始状态
    pub original_status: Option<String>,
    /// 原始金额
    pub original_amount: f64,
    /// 新状态
    pub new_status: String,
    /// 新金额（None 表示不变）
    pub new_amount: Option<f64>,
    /// 提取到的退款金额（None 表示未提取到）
    pub refund_amount: Option<f64>,
    /// 处理说明
    pub note: String,
}

/// 清洗预览结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleaningPreviewResult {
    /// 待过滤列表
    pub to_exclude: Vec<TransactionItem>,
    /// 待修改列表
    pub to_modify: Vec<ModifyItem>,
    /// 待过滤数量
    pub exclude_count: usize,
    /// 待修改数量
    pub modify_count: usize,
}

/// 清洗执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleaningExecuteResult {
    /// 已排除数量
    pub excluded_count: usize,
    /// 已修改数量
    pub modified_count: usize,
    /// 剩余有效交易数
    pub remaining_count: usize,
}

// ====================================================================
// 常量
// ====================================================================

/// 需要过滤的交易状态列表
const EXCLUDE_STATUSES: &[&str] = &["还款成功", "交易关闭", "退款成功", "不计收入", "已全额退款"];

/// 小额过滤阈值（元）
const SMALL_AMOUNT_THRESHOLD: f64 = 3.0;

// ====================================================================
// 辅助函数
// ====================================================================

/// 从 raw_data JSON 字符串中提取 remark 字段
///
/// 微信 raw_data 格式: {"transaction_id":"...","merchant_order_id":"...","remark":"..."}
/// 支付宝 raw_data 格式: {"counterparty_account":"...","transaction_order_id":"...","merchant_order_id":"...","remark":"..."}
fn extract_remark(raw_data: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(raw_data).ok()?;
    value.get("remark")?.as_str().map(|s| s.to_string())
}

/// 从 remark 字符串中提取退款金额
///
/// 策略：
/// 1. 先尝试匹配 "退款" 关键词后面的数字
/// 2. 回退：提取字符串中的第一个正数
fn extract_refund_amount(remark: &str) -> Option<f64> {
    // 策略 1：匹配 "退款" 后面的数字（允许中间有非数字字符）
    let re_refund = regex::Regex::new(r"退款[^\d]*([\d.]+)").ok()?;
    if let Some(caps) = re_refund.captures(remark) {
        if let Some(m) = caps.get(1) {
            if let Ok(amount) = m.as_str().parse::<f64>() {
                if amount > 0.0 {
                    return Some(amount);
                }
            }
        }
    }

    // 策略 2：提取字符串中的第一个正数
    let re_any = regex::Regex::new(r"([\d.]+)").ok()?;
    if let Some(caps) = re_any.captures(remark) {
        if let Some(m) = caps.get(1) {
            if let Ok(amount) = m.as_str().parse::<f64>() {
                if amount > 0.0 {
                    return Some(amount);
                }
            }
        }
    }

    None
}

/// 计算部分退款交易的修改方案
///
/// 返回 (new_status, new_amount, refund_amount, note)
fn compute_modification(tx: &Transaction) -> ModifyItem {
    let raw_data = tx.raw_data.as_deref().unwrap_or("");
    let remark = extract_remark(raw_data);

    let (new_status, new_amount, refund_amount, note) = match &remark {
        Some(remark_str) => {
            match extract_refund_amount(remark_str) {
                // 退款金额 ≤ 3 → 状态改为"支付成功"，金额不变
                Some(refund) if refund <= SMALL_AMOUNT_THRESHOLD => {
                    (
                        "支付成功".to_string(),
                        None,
                        Some(refund),
                        format!(
                            "退款金额{:.2}元（≤{:.0}元），状态改为支付成功，金额不变",
                            refund, SMALL_AMOUNT_THRESHOLD
                        ),
                    )
                }
                // 退款金额 > 3 → 状态改为"部分退款"，金额 = 原amount - 退款金额
                Some(refund) => {
                    let new_amt = tx.amount - refund;
                    (
                        "部分退款".to_string(),
                        Some(new_amt),
                        Some(refund),
                        format!(
                            "退款金额{:.2}元，金额调整为{:.2}（原{:.2} - 退{:.2}）",
                            refund, new_amt, tx.amount, refund
                        ),
                    )
                }
                // 无法提取退款金额 → 只修改状态，金额不变
                None => {
                    (
                        "部分退款".to_string(),
                        None,
                        None,
                        "未提取到退款金额".to_string(),
                    )
                }
            }
        }
        // 没有 remark 字段 → 只修改状态，金额不变
        None => {
            (
                "部分退款".to_string(),
                None,
                None,
                "未提取到退款金额".to_string(),
            )
        }
    };

    ModifyItem {
        id: tx.id.clone(),
        transaction_time: tx.transaction_time.clone(),
        source: tx.source.clone(),
        counterparty: tx.counterparty.clone(),
        product: tx.product.clone(),
        original_status: tx.status.clone(),
        original_amount: tx.amount,
        new_status,
        new_amount,
        refund_amount,
        note,
    }
}

/// 检查交易是否应该被排除（过滤）
///
/// 返回 Some(reason) 表示应该排除，None 表示不需要排除
fn check_exclude(tx: &Transaction) -> Option<String> {
    // 规则 1：状态在排除列表中
    if let Some(ref status) = tx.status {
        if EXCLUDE_STATUSES.contains(&status.as_str()) {
            return Some(format!("状态:{}", status));
        }
    }

    // 规则 2：支出方向且金额 ≤ 3 元
    if tx.direction == "expense" && tx.amount <= SMALL_AMOUNT_THRESHOLD {
        return Some(format!("金额≤{:.0}元（{:.2}元）", SMALL_AMOUNT_THRESHOLD, tx.amount));
    }

    None
}

// ====================================================================
// 公共 API
// ====================================================================

/// 清洗预览
///
/// 扫描所有 is_excluded_from_summary = 0 的交易，分类为：
/// - to_exclude: 需要过滤（软删除）的交易
/// - to_modify: 需要修改（部分退款处理）的交易
///
/// 分类优先级：部分退款 > 状态排除 > 金额排除
pub fn preview_cleaning(conn: &Connection) -> Result<CleaningPreviewResult, String> {
    let transactions = transaction_dao::list_transactions_for_cleaning(conn)?;

    let mut to_exclude = Vec::new();
    let mut to_modify = Vec::new();

    for tx in transactions {
        // 优先检查部分退款（部分退款需要修改金额，不直接排除）
        let is_partial_refund = tx
            .status
            .as_ref()
            .map(|s| s.contains("部分退款"))
            .unwrap_or(false);

        if is_partial_refund {
            // 部分退款 → 待修改
            to_modify.push(compute_modification(&tx));
        } else if let Some(reason) = check_exclude(&tx) {
            // 满足排除条件 → 待过滤
            to_exclude.push(TransactionItem {
                id: tx.id.clone(),
                transaction_time: tx.transaction_time.clone(),
                source: tx.source.clone(),
                transaction_type: tx.transaction_type.clone(),
                counterparty: tx.counterparty.clone(),
                product: tx.product.clone(),
                direction: tx.direction.clone(),
                amount: tx.amount,
                status: tx.status.clone(),
                reason,
            });
        }
        // 其余交易不需要处理
    }

    let exclude_count = to_exclude.len();
    let modify_count = to_modify.len();

    Ok(CleaningPreviewResult {
        to_exclude,
        to_modify,
        exclude_count,
        modify_count,
    })
}

/// 执行清洗
///
/// # 参数
/// - `conn`: 数据库连接
/// - `exclude_ids`: 确认要排除的交易 ID 列表
/// - `modify_ids`: 确认要修改的交易 ID 列表
///
/// # 执行顺序
/// 1. 先执行修改（需要在交易未被排除前查询原始数据）
/// 2. 再执行排除
/// 3. 统计剩余有效交易数
pub fn execute_cleaning(
    conn: &Connection,
    exclude_ids: &[String],
    modify_ids: &[String],
) -> Result<CleaningExecuteResult, String> {
    // ----------------------------------------------------------------
    // 步骤 1：执行修改（部分退款处理）
    // ----------------------------------------------------------------
    let mut modified_count = 0usize;
    if !modify_ids.is_empty() {
        // 查询当前未排除的交易（修改需要在排除前进行）
        let all_transactions = transaction_dao::list_transactions_for_cleaning(conn)?;
        let id_set: std::collections::HashSet<&String> = modify_ids.iter().collect();

        for tx in &all_transactions {
            if id_set.contains(&tx.id) {
                let modification = compute_modification(tx);
                transaction_dao::batch_update_transaction(
                    conn,
                    &tx.id,
                    &modification.new_status,
                    modification.new_amount,
                )?;
                modified_count += 1;
            }
        }
    }

    // ----------------------------------------------------------------
    // 步骤 2：执行排除（软删除）
    // ----------------------------------------------------------------
    let excluded_count = if exclude_ids.is_empty() {
        0
    } else {
        transaction_dao::batch_set_excluded(conn, exclude_ids)?
    };

    // ----------------------------------------------------------------
    // 步骤 3：统计剩余有效交易数
    // ----------------------------------------------------------------
    let remaining_transactions = transaction_dao::list_transactions_for_cleaning(conn)?;
    let remaining_count = remaining_transactions.len();

    Ok(CleaningExecuteResult {
        excluded_count,
        modified_count,
        remaining_count,
    })
}
