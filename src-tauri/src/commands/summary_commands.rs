// 汇总相关 Tauri 命令
// 供前端通过 invoke 调用

use tauri::State;

use crate::db::DatabaseState;
use crate::models::transaction::Transaction;
use crate::services::summary_service::{self, MonthlySummary};

/// 获取月度汇总
///
/// 按月份 + 消费标签 GROUP BY + SUM，通过 summary_mappings 合并子标签到汇总类。
///
/// # 参数
/// - `month`: 月份（YYYY-MM 格式）
///
/// # 返回
/// MonthlySummary — 含各汇总类金额、子标签明细、总支出
#[tauri::command]
pub fn get_monthly_summary(
    state: State<'_, DatabaseState>,
    month: String,
) -> Result<MonthlySummary, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    summary_service::get_monthly_summary(&conn, &month)
}

/// 按月份和标签列表查询交易明细（供汇总下钻跳转使用）
///
/// # 参数
/// - `month`: 月份（YYYY-MM）
/// - `tag_ids`: 标签 ID 列表（为空时返回当月全部有效支出）
///
/// # 返回
/// 符合条件的交易列表
#[tauri::command]
pub fn get_transactions_by_tags(
    state: State<'_, DatabaseState>,
    month: String,
    tag_ids: Vec<String>,
) -> Result<Vec<Transaction>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    summary_service::get_transactions_by_tags(&conn, &month, &tag_ids)
}

/// 获取全部月份汇总透视表（每行一个月的完整数据，含手动数据）
#[tauri::command]
pub fn get_all_months_summary(
    state: State<'_, DatabaseState>,
) -> Result<Vec<summary_service::MonthlySummaryRow>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    summary_service::get_all_months_summary(&conn)
}
