// 导入服务 — 编排解析器和 DAO，完成账单导入流程
//
// 流程：
// 1. 读取文件并计算 SHA-256 哈希
// 2. 根据 source 调用对应解析器
// 3. 将 ParsedTransaction 转换为 Transaction
// 4. 批量插入交易记录
// 5. 插入导入记录
// 6. 返回 ImportResult

use std::collections::HashSet;
use std::path::Path;

use rusqlite::Connection;
use sha2::{Digest, Sha256};

use crate::dao::import_dao;
use crate::dao::transaction_dao;
use crate::models::import_record::ImportRecord;
use crate::models::transaction::Transaction;
use crate::parsers::alipay_parser;
use crate::parsers::wechat_parser;
use crate::parsers::ParsedTransaction;

use super::ImportResult;

/// 导入账单文件
///
/// # 参数
/// - `file_path`: 文件路径
/// - `source`: 数据来源（"wechat" 或 "alipay"）
/// - `payer`: 归属人名称（可选）
/// - `conn`: 数据库连接
///
/// # 返回
/// ImportResult 包含导入统计信息
pub fn import_bill(
    file_path: &str,
    source: &str,
    payer: Option<String>,
    conn: &Connection,
) -> Result<ImportResult, String> {
    // 1. 读取文件并计算哈希
    let file_bytes = std::fs::read(file_path)
        .map_err(|e| format!("读取文件失败: {}", e))?;

    let mut hasher = Sha256::new();
    hasher.update(&file_bytes);
    let hash_result = hasher.finalize();
    let file_hash: String = hash_result.iter().map(|b| format!("{:02x}", b)).collect();

    // 2. 提取文件名
    let file_name = Path::new(file_path)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| file_path.to_string());

    // 3. 根据 source 调用对应解析器
    let (account_info, parsed_transactions) = match source {
        "wechat" => {
            let result = wechat_parser::parse_wechat_xlsx(file_path)?;
            (result.account_info, result.transactions)
        }
        "alipay" => {
            let result = alipay_parser::parse_alipay_csv(file_path)?;
            (result.account_info, result.transactions)
        }
        _ => return Err(format!("不支持的数据来源: '{}'", source)),
    };

    let total_count = parsed_transactions.len();

    // 4. 转换 ParsedTransaction → Transaction
    let mut transactions = parsed_transactions
        .iter()
        .map(|pt| parsed_to_transaction(pt, source))
        .collect::<Vec<Transaction>>();

    // 5. 收集涉及的月份
    let months_set: HashSet<String> = parsed_transactions
        .iter()
        .map(|pt| pt.month.clone())
        .collect();
    let mut months: Vec<String> = months_set.into_iter().collect();
    months.sort();

    // 6. 批量插入交易记录
    let imported_count =
        transaction_dao::insert_transactions_batch(conn, &mut transactions, payer.clone())?;

    // 7. 插入导入记录
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let import_record = ImportRecord {
        id: uuid::Uuid::new_v4().to_string(),
        month: Some(months.join(",")),
        source: source.to_string(),
        payer: payer.clone().filter(|s| !s.is_empty()),
        file_name: file_name.clone(),
        file_hash: Some(file_hash),
        account_info: Some(account_info.clone()),
        total_count: Some(total_count as i64),
        valid_count: Some(imported_count as i64),
        filtered_count: Some(0),
        imported_at: now,
    };

    import_dao::create_import_record(conn, &import_record)?;

    // 8. 返回结果
    Ok(ImportResult {
        source: source.to_string(),
        account_info,
        total_count,
        imported_count,
        skipped_count: total_count - imported_count,
        months,
        payer: payer.filter(|s| !s.is_empty()),
        file_name,
    })
}

/// 计算文件 SHA-256 哈希
pub fn compute_file_hash(file_path: &str) -> Result<String, String> {
    let bytes = std::fs::read(file_path)
        .map_err(|e| format!("读取文件失败: {}", e))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let result = hasher.finalize();
    let hex: String = result.iter().map(|b| format!("{:02x}", b)).collect();
    Ok(hex)
}

/// 将 ParsedTransaction 转换为 Transaction
fn parsed_to_transaction(pt: &ParsedTransaction, source: &str) -> Transaction {
    Transaction {
        id: String::new(), // 由 insert_transactions_batch 填充
        transaction_time: pt.transaction_time.clone(),
        tag_id: None,
        tag_source: None,
        source: source.to_string(),
        transaction_type: Some(pt.transaction_type.clone()),
        counterparty: Some(pt.counterparty.clone()),
        product: Some(pt.product.clone()),
        direction: pt.direction.clone(),
        amount: pt.amount,
        payment_method: Some(pt.payment_method.clone()),
        status: Some(pt.status.clone()),
        payer: None,           // 由 insert_transactions_batch 填充
        payer_source: None,    // 由 insert_transactions_batch 填充
        is_rigid: None,
        is_excluded_from_summary: 0, // 由 insert_transactions_batch 填充
        ai_suggested_tag: None,
        ai_confidence: None,
        payment_purpose: None,
        month: pt.month.clone(),
        raw_data: Some(pt.raw_data.clone()),
        created_at: String::new(), // 由 insert_transactions_batch 填充
        updated_at: String::new(), // 由 insert_transactions_batch 填充
    }
}
