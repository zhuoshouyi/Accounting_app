// 分类相关 Tauri 命令
// 供前端通过 invoke 调用

use tauri::State;

use crate::db::DatabaseState;
use crate::dao::tag_dao;
use crate::dao::transaction_dao;
use crate::models::category_tag::CategoryTag;
use crate::models::transaction::Transaction;
use crate::services::classification_engine::{self, ClassifyResult};

/// 执行自动分类
///
/// 对所有 tag_source IS NULL 且 is_excluded_from_summary=0 的交易
/// 执行三层匹配（AI 学习规则 → 内置规则 → 留空），更新标签。
///
/// # 返回
/// 分类结果统计（总数 / 已分类 / 未分类）
#[tauri::command]
pub fn classify_transactions(state: State<'_, DatabaseState>) -> Result<ClassifyResult, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    classification_engine::classify_transactions(&conn)
}

/// 获取未分类交易列表
///
/// 查询 tag_source IS NULL AND is_excluded_from_summary=0 的交易，
/// 按 transaction_time DESC 排序。
///
/// # 返回
/// 未分类的有效交易列表
#[tauri::command]
pub fn list_unclassified(state: State<'_, DatabaseState>) -> Result<Vec<Transaction>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    transaction_dao::list_unclassified_transactions(&conn)
}

/// 修改单条交易的标签
///
/// # 参数
/// - `id`: 交易 ID
/// - `tag_id`: 标签 ID（null 表示置空标签，即取消分类）
/// - `tag_source`: 标签来源（manual / rule / ai_learned / ai）
#[tauri::command]
pub fn update_transaction_tag(
    state: State<'_, DatabaseState>,
    id: String,
    tag_id: Option<String>,
    tag_source: String,
) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    transaction_dao::update_transaction_tag(&conn, &id, tag_id.as_deref(), &tag_source)
}

/// 批量修改交易标签
///
/// # 参数
/// - `ids`: 交易 ID 列表
/// - `tag_id`: 要设置的标签 ID
/// - `tag_source`: 标签来源
///
/// # 返回
/// 实际更新的记录数
#[tauri::command]
pub fn batch_update_tags(
    state: State<'_, DatabaseState>,
    ids: Vec<String>,
    tag_id: String,
    tag_source: String,
) -> Result<usize, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    transaction_dao::batch_update_tags(&conn, &ids, &tag_id, &tag_source)
}

/// 获取所有消费标签
///
/// 查询 category_tags 表全部记录，按 sort_order 升序排列。
///
/// # 返回
/// 标签列表
#[tauri::command]
pub fn list_category_tags(state: State<'_, DatabaseState>) -> Result<Vec<CategoryTag>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    tag_dao::list_all_tags(&conn)
}

/// 查询已跳过的交易列表
#[tauri::command]
pub fn list_skipped(state: State<'_, DatabaseState>) -> Result<Vec<Transaction>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    transaction_dao::list_skipped_transactions(&conn)
}

/// 恢复跳过的交易（重新进入复核队列）
#[tauri::command]
pub fn unskip_transaction(state: State<'_, DatabaseState>, id: String) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    transaction_dao::unskip_transaction(&conn, &id)
}
