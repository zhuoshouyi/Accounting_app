// 交易记录 DAO — 批量插入 + 查询 + 清洗操作 + 汇总下钻

use rusqlite::{params, Connection, Row};

use crate::models::transaction::Transaction;

/// 批量插入交易记录
///
/// # 参数
/// - `conn`: 数据库连接
/// - `transactions`: 交易记录列表（id/created_at/updated_at 由本函数填充）
/// - `payer`: 归属人名称（如果不为空，会验证 account_owners 表中存在该名称）
///
/// # 返回
/// 实际插入的记录数
pub fn insert_transactions_batch(
    conn: &Connection,
    transactions: &mut Vec<Transaction>,
    payer: Option<String>,
) -> Result<usize, String> {
    // 验证归属人存在
    if let Some(ref payer_name) = payer {
        if !payer_name.is_empty() {
            let exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM account_owners WHERE name = ?",
                    params![payer_name],
                    |row| row.get(0),
                )
                .map_err(|e| format!("验证归属人失败: {}", e))?;

            if !exists {
                return Err(format!("归属人 '{}' 不存在，请先在归属人管理中添加", payer_name));
            }
        }
    }

    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();

    // 填充系统字段
    for tx in transactions.iter_mut() {
        tx.id = uuid::Uuid::new_v4().to_string();
        tx.payer = payer.clone().filter(|s| !s.is_empty());
        tx.payer_source = if tx.payer.is_some() {
            Some("manual".to_string())
        } else {
            None
        };
        tx.is_excluded_from_summary = if tx.direction == "neutral" { 1 } else { 0 };
        tx.tag_id = None;
        tx.tag_source = None;
        tx.is_rigid = None;
        tx.ai_suggested_tag = None;
        tx.ai_confidence = None;
        tx.payment_purpose = None;
        tx.created_at = now.clone();
        tx.updated_at = now.clone();
    }

    // 开启事务批量插入
    conn.execute_batch("BEGIN")
        .map_err(|e| format!("开启事务失败: {}", e))?;

    let mut count = 0usize;
    for tx in transactions.iter() {
        let result = conn.execute(
            r#"INSERT INTO transactions (
                id, transaction_time, tag_id, tag_source, source,
                transaction_type, counterparty, product, direction,
                amount, payment_method, status, payer, payer_source,
                is_rigid, is_excluded_from_summary, ai_suggested_tag,
                ai_confidence, payment_purpose, month, raw_data,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            params![
                tx.id,
                tx.transaction_time,
                tx.tag_id,
                tx.tag_source,
                tx.source,
                tx.transaction_type,
                tx.counterparty,
                tx.product,
                tx.direction,
                tx.amount,
                tx.payment_method,
                tx.status,
                tx.payer,
                tx.payer_source,
                tx.is_rigid,
                tx.is_excluded_from_summary,
                tx.ai_suggested_tag,
                tx.ai_confidence,
                tx.payment_purpose,
                tx.month,
                tx.raw_data,
                tx.created_at,
                tx.updated_at,
            ],
        );

        match result {
            Ok(_) => count += 1,
            Err(e) => {
                conn.execute_batch("ROLLBACK").ok();
                return Err(format!(
                    "插入第 {} 条交易失败（交易时间: {}）: {}",
                    count + 1,
                    tx.transaction_time,
                    e
                ));
            }
        }
    }

    conn.execute_batch("COMMIT")
        .map_err(|e| format!("提交事务失败: {}", e))?;

    Ok(count)
}

// ====================================================================
// 以下为 Step 3 新增方法
// ====================================================================

/// 将数据库行映射为 Transaction 结构体
fn row_to_transaction(row: &Row) -> rusqlite::Result<Transaction> {
    Ok(Transaction {
        id: row.get("id")?,
        transaction_time: row.get("transaction_time")?,
        tag_id: row.get("tag_id")?,
        tag_source: row.get("tag_source")?,
        source: row.get("source")?,
        transaction_type: row.get("transaction_type")?,
        counterparty: row.get("counterparty")?,
        product: row.get("product")?,
        direction: row.get("direction")?,
        amount: row.get("amount")?,
        payment_method: row.get("payment_method")?,
        status: row.get("status")?,
        payer: row.get("payer")?,
        payer_source: row.get("payer_source")?,
        is_rigid: row.get("is_rigid")?,
        is_excluded_from_summary: row.get("is_excluded_from_summary")?,
        ai_suggested_tag: row.get("ai_suggested_tag")?,
        ai_confidence: row.get("ai_confidence")?,
        payment_purpose: row.get("payment_purpose")?,
        month: row.get("month")?,
        raw_data: row.get("raw_data")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

/// 查询交易列表
/// month = None 查全部，按 transaction_time DESC 排序
pub fn list_transactions_by_month(
    conn: &Connection,
    month: Option<String>,
) -> Result<Vec<Transaction>, String> {
    let transactions = match &month {
        Some(m) => {
            let mut stmt = conn
                .prepare(
                    "SELECT * FROM transactions WHERE month = ? ORDER BY transaction_time DESC",
                )
                .map_err(|e| format!("准备查询失败: {}", e))?;
            let rows = stmt
                .query_map(params![m], row_to_transaction)
                .map_err(|e| format!("查询交易失败: {}", e))?;
            let result: Vec<Transaction> = rows
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("映射交易失败: {}", e))?;
            result
        }
        None => {
            let mut stmt = conn
                .prepare("SELECT * FROM transactions ORDER BY transaction_time DESC")
                .map_err(|e| format!("准备查询失败: {}", e))?;
            let rows = stmt
                .query_map([], row_to_transaction)
                .map_err(|e| format!("查询交易失败: {}", e))?;
            let result: Vec<Transaction> = rows
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("映射交易失败: {}", e))?;
            result
        }
    };
    Ok(transactions)
}

/// 获取所有交易的月份（去重，降序）
pub fn get_distinct_months(conn: &Connection) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare("SELECT DISTINCT month FROM transactions ORDER BY month DESC")
        .map_err(|e| format!("准备查询失败: {}", e))?;
    let months = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| format!("查询月份失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("映射月份失败: {}", e))?;
    Ok(months)
}

/// 查询 is_excluded_from_summary = 0 的交易，用于清洗预览
pub fn list_transactions_for_cleaning(conn: &Connection) -> Result<Vec<Transaction>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT * FROM transactions WHERE is_excluded_from_summary = 0 ORDER BY transaction_time DESC",
        )
        .map_err(|e| format!("准备查询失败: {}", e))?;
    let transactions = stmt
        .query_map([], row_to_transaction)
        .map_err(|e| format!("查询交易失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("映射交易失败: {}", e))?;
    Ok(transactions)
}

/// 批量设置 is_excluded_from_summary = 1（软删除）
///
/// # 参数
/// - `conn`: 数据库连接
/// - `ids`: 要排除的交易 ID 列表
///
/// # 返回
/// 实际更新的记录数
pub fn batch_set_excluded(conn: &Connection, ids: &[String]) -> Result<usize, String> {
    if ids.is_empty() {
        return Ok(0);
    }
    let now = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string();

    conn.execute_batch("BEGIN")
        .map_err(|e| format!("开启事务失败: {}", e))?;

    let mut count = 0usize;
    for id in ids {
        let result = conn.execute(
            "UPDATE transactions SET is_excluded_from_summary = 1, updated_at = ? WHERE id = ?",
            params![now, id],
        );
        match result {
            Ok(_) => count += 1,
            Err(e) => {
                conn.execute_batch("ROLLBACK").ok();
                return Err(format!("更新交易失败（id: {}）: {}", id, e));
            }
        }
    }

    conn.execute_batch("COMMIT")
        .map_err(|e| format!("提交事务失败: {}", e))?;

    Ok(count)
}

/// 更新单条交易的 status 和 amount
/// amount = None 时只更新 status
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 交易 ID
/// - `status`: 新状态
/// - `amount`: 新金额（None 表示不修改金额）
pub fn batch_update_transaction(
    conn: &Connection,
    id: &str,
    status: &str,
    amount: Option<f64>,
) -> Result<(), String> {
    let now = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string();

    match amount {
        Some(a) => {
            conn.execute(
                "UPDATE transactions SET status = ?, amount = ?, updated_at = ? WHERE id = ?",
                params![status, a, now, id],
            )
            .map_err(|e| format!("更新交易失败（id: {}）: {}", id, e))?;
        }
        None => {
            conn.execute(
                "UPDATE transactions SET status = ?, updated_at = ? WHERE id = ?",
                params![status, now, id],
            )
            .map_err(|e| format!("更新交易失败（id: {}）: {}", id, e))?;
        }
    }

    Ok(())
}

// ====================================================================
// 以下为 Step 4 新增方法（分类标签相关）
// ====================================================================

/// 查询未分类的交易
///
/// 条件：tag_source IS NULL AND is_excluded_from_summary = 0
/// 排序：transaction_time DESC
///
/// # 参数
/// - `conn`: 数据库连接
///
/// # 返回
/// 未分类的有效交易列表
pub fn list_unclassified_transactions(conn: &Connection) -> Result<Vec<Transaction>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT * FROM transactions WHERE tag_source IS NULL AND is_excluded_from_summary = 0 ORDER BY transaction_time DESC",
        )
        .map_err(|e| format!("准备查询失败: {}", e))?;
    let transactions = stmt
        .query_map([], row_to_transaction)
        .map_err(|e| format!("查询未分类交易失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("映射交易失败: {}", e))?;
    Ok(transactions)
}

/// 更新单条交易的标签
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 交易 ID
/// - `tag_id`: 标签 ID（None 表示置空标签）
/// - `tag_source`: 标签来源（manual / rule / ai_learned / ai）
pub fn update_transaction_tag(
    conn: &Connection,
    id: &str,
    tag_id: Option<&str>,
    tag_source: &str,
) -> Result<(), String> {
    let now = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string();

    conn.execute(
        "UPDATE transactions SET tag_id = ?, tag_source = ?, updated_at = ? WHERE id = ?",
        params![tag_id, tag_source, now, id],
    )
    .map_err(|e| format!("更新交易标签失败（id: {}）: {}", id, e))?;

    Ok(())
}

/// 批量更新交易标签
///
/// # 参数
/// - `conn`: 数据库连接
/// - `ids`: 交易 ID 列表
/// - `tag_id`: 要设置的标签 ID
/// - `tag_source`: 标签来源
///
/// # 返回
/// 实际更新的记录数
pub fn batch_update_tags(
    conn: &Connection,
    ids: &[String],
    tag_id: &str,
    tag_source: &str,
) -> Result<usize, String> {
    if ids.is_empty() {
        return Ok(0);
    }

    let now = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string();

    conn.execute_batch("BEGIN")
        .map_err(|e| format!("开启事务失败: {}", e))?;

    let mut count = 0usize;
    for id in ids {
        let result = conn.execute(
            "UPDATE transactions SET tag_id = ?, tag_source = ?, updated_at = ? WHERE id = ?",
            params![tag_id, tag_source, now, id],
        );
        match result {
            Ok(_) => count += 1,
            Err(e) => {
                conn.execute_batch("ROLLBACK").ok();
                return Err(format!("更新交易标签失败（id: {}）: {}", id, e));
            }
        }
    }

    conn.execute_batch("COMMIT")
        .map_err(|e| format!("提交事务失败: {}", e))?;

    Ok(count)
}

// ====================================================================
// 以下为 Step 5 新增方法（月度汇总下钻）
// ====================================================================

/// 按月份和标签 ID 列表查询交易明细（供汇总下钻使用）
///
/// # 参数
/// - `conn`: 数据库连接
/// - `month`: 月份（YYYY-MM）
/// - `tag_ids`: 标签 ID 列表（为空时返回当月全部有效支出交易）
///
/// # 返回
/// 符合条件的交易列表（按时间倒序）
pub fn list_transactions_by_tags(
    conn: &Connection,
    month: &str,
    tag_ids: &[String],
) -> Result<Vec<Transaction>, String> {
    if tag_ids.is_empty() {
        let mut stmt = conn
            .prepare(
                "SELECT * FROM transactions
                 WHERE month = ?
                   AND direction = 'expense'
                   AND is_excluded_from_summary = 0
                 ORDER BY transaction_time DESC",
            )
            .map_err(|e| format!("准备查询失败: {}", e))?;
        let transactions = stmt
            .query_map(params![month], row_to_transaction)
            .map_err(|e| format!("查询下钻数据失败: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("映射下钻数据失败: {}", e))?;
        return Ok(transactions);
    }

    let placeholders: Vec<String> = tag_ids
        .iter()
        .enumerate()
        .map(|(i, _)| format!("?{}", i + 2))
        .collect();
    let sql = format!(
        "SELECT * FROM transactions
         WHERE month = ?1
           AND tag_id IN ({})
           AND is_excluded_from_summary = 0
         ORDER BY transaction_time DESC",
        placeholders.join(",")
    );

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("准备查询失败: {}", e))?;

    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    param_values.push(Box::new(month.to_string()));
    for id in tag_ids {
        param_values.push(Box::new(id.clone()));
    }
    let params_ref: Vec<&dyn rusqlite::types::ToSql> =
        param_values.iter().map(|p| p.as_ref()).collect();

    let transactions = stmt
        .query_map(params_ref.as_slice(), row_to_transaction)
        .map_err(|e| format!("查询下钻数据失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("映射下钻数据失败: {}", e))?;

    Ok(transactions)
}

// ====================================================================
// 以下为 Step 6 新增方法（字段级更新 + 批量操作）
// ====================================================================

/// 更新单条交易的归属人
pub fn update_transaction_payer(
    conn: &Connection,
    id: &str,
    payer: Option<&str>,
) -> Result<(), String> {
    let now = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string();
    conn.execute(
        "UPDATE transactions SET payer = ?, payer_source = 'manual', updated_at = ? WHERE id = ?",
        params![payer, now, id],
    )
    .map_err(|e| format!("更新归属人失败（id: {}）: {}", id, e))?;
    Ok(())
}

/// 更新单条交易的刚需标记
pub fn update_transaction_rigid(
    conn: &Connection,
    id: &str,
    is_rigid: bool,
) -> Result<(), String> {
    let now = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string();
    let value: i64 = if is_rigid { 1 } else { 0 };
    conn.execute(
        "UPDATE transactions SET is_rigid = ?, updated_at = ? WHERE id = ?",
        params![value, now, id],
    )
    .map_err(|e| format!("更新刚需标记失败（id: {}）: {}", id, e))?;
    Ok(())
}

/// 批量更新归属人
pub fn batch_update_payer(
    conn: &Connection,
    ids: &[String],
    payer: &str,
) -> Result<usize, String> {
    if ids.is_empty() {
        return Ok(0);
    }
    let now = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string();
    conn.execute_batch("BEGIN").map_err(|e| format!("开启事务失败: {}", e))?;
    let mut count = 0usize;
    for id in ids {
        conn.execute(
            "UPDATE transactions SET payer = ?, payer_source = 'manual', updated_at = ? WHERE id = ?",
            params![payer, now, id],
        )
        .map_err(|e| {
            conn.execute_batch("ROLLBACK").ok();
            format!("批量更新归属人失败: {}", e)
        })?;
        count += 1;
    }
    conn.execute_batch("COMMIT").map_err(|e| format!("提交事务失败: {}", e))?;
    Ok(count)
}

/// 批量更新刚需标记
pub fn batch_update_rigid(
    conn: &Connection,
    ids: &[String],
    is_rigid: bool,
) -> Result<usize, String> {
    if ids.is_empty() {
        return Ok(0);
    }
    let now = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string();
    let value: i64 = if is_rigid { 1 } else { 0 };
    conn.execute_batch("BEGIN").map_err(|e| format!("开启事务失败: {}", e))?;
    let mut count = 0usize;
    for id in ids {
        conn.execute(
            "UPDATE transactions SET is_rigid = ?, updated_at = ? WHERE id = ?",
            params![value, now, id],
        )
        .map_err(|e| {
            conn.execute_batch("ROLLBACK").ok();
            format!("批量更新刚需标记失败: {}", e)
        })?;
        count += 1;
    }
    conn.execute_batch("COMMIT").map_err(|e| format!("提交事务失败: {}", e))?;
    Ok(count)
}

// ====================================================================
// 以下为 Step 7 新增（AI 建议写入）
// ====================================================================

/// 写入 AI 建议标签和置信度
pub fn update_ai_suggestion(
    conn: &Connection,
    id: &str,
    tag_id: &str,
    confidence: f64,
) -> Result<(), String> {
    let now = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string();
    conn.execute(
        "UPDATE transactions SET ai_suggested_tag = ?, ai_confidence = ?, updated_at = ? WHERE id = ?",
        params![tag_id, confidence, now, id],
    )
    .map_err(|e| format!("更新 AI 建议失败: {}", e))?;
    Ok(())
}

/// 查询已跳过的交易（tag_source='manual' AND tag_id IS NULL AND 未排除）
pub fn list_skipped_transactions(conn: &Connection) -> Result<Vec<Transaction>, String> {
    let mut stmt = conn
        .prepare("SELECT * FROM transactions WHERE tag_source = 'manual' AND tag_id IS NULL AND is_excluded_from_summary = 0 ORDER BY updated_at DESC")
        .map_err(|e| format!("查询跳过交易失败: {}", e))?;
    let rows: Vec<Transaction> = stmt.query_map([], row_to_transaction)
        .map_err(|e| format!("映射跳过交易失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("读取跳过交易失败: {}", e))?;
    Ok(rows)
}

/// 批量删除交易
pub fn batch_delete_transactions(conn: &Connection, ids: &[String]) -> Result<usize, String> {
    if ids.is_empty() { return Ok(0); }
    conn.execute_batch("BEGIN").map_err(|e| format!("开启事务失败: {}", e))?;
    let mut count = 0usize;
    for id in ids {
        conn.execute("DELETE FROM transactions WHERE id = ?", params![id])
            .map_err(|e| { conn.execute_batch("ROLLBACK").ok(); format!("删除失败: {}", e) })?;
        count += 1;
    }
    conn.execute_batch("COMMIT").map_err(|e| format!("提交事务失败: {}", e))?;
    Ok(count)
}

/// 恢复跳过的交易（tag_source 和 tag_id 重置为 NULL，重新进入复核队列）
pub fn unskip_transaction(conn: &Connection, id: &str) -> Result<(), String> {
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    conn.execute(
        "UPDATE transactions SET tag_source = NULL, tag_id = NULL, updated_at = ? WHERE id = ?",
        params![now, id],
    )
    .map_err(|e| format!("恢复跳过交易失败: {}", e))?;
    Ok(())
}
