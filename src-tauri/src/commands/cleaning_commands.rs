// 数据清洗 Tauri 命令
// 供前端通过 invoke 调用

use tauri::State;

use crate::db::DatabaseState;
use crate::dao::transaction_dao;
use crate::models::transaction::Transaction;
use crate::services::cleaning_service::{self, CleaningExecuteResult, CleaningPreviewResult};

/// 清洗预览
///
/// 扫描所有未排除的交易，分类为待过滤和待修改列表
/// 前端展示后由用户确认，再调用 execute_cleaning 执行
#[tauri::command]
pub fn preview_cleaning(state: State<'_, DatabaseState>) -> Result<CleaningPreviewResult, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    cleaning_service::preview_cleaning(&conn)
}

/// 执行清洗
///
/// # 参数
/// - `exclude_ids`: 确认要排除（软删除）的交易 ID 列表
/// - `modify_ids`: 确认要修改（部分退款处理）的交易 ID 列表
#[tauri::command]
pub fn execute_cleaning(
    state: State<'_, DatabaseState>,
    exclude_ids: Vec<String>,
    modify_ids: Vec<String>,
) -> Result<CleaningExecuteResult, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    cleaning_service::execute_cleaning(&conn, &exclude_ids, &modify_ids)
}

/// 查询交易列表
///
/// # 参数
/// - `month`: 月份筛选（如 "2026-06"），为 null 时查询全部
///
/// # 返回
/// 按 transaction_time DESC 排序的交易列表
#[tauri::command]
pub fn list_transactions(
    state: State<'_, DatabaseState>,
    month: Option<String>,
) -> Result<Vec<Transaction>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    transaction_dao::list_transactions_by_month(&conn, month)
}

/// 获取所有交易的月份（去重，降序）
#[tauri::command]
pub fn get_distinct_months(state: State<'_, DatabaseState>) -> Result<Vec<String>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    transaction_dao::get_distinct_months(&conn)
}
