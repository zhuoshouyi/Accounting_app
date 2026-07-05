// 归属人管理 Tauri 命令
// 供前端通过 invoke 调用

use tauri::State;

use crate::db::DatabaseState;
use crate::dao::account_owner_dao;
use crate::models::account_owner::AccountOwner;

/// 查询所有归属人（按 sort_order 排序）
#[tauri::command]
pub fn list_account_owners(state: State<'_, DatabaseState>) -> Vec<AccountOwner> {
    let conn = state.conn.lock().unwrap_or_else(|e| e.into_inner());
    account_owner_dao::list_owners(&conn)
}

/// 新增归属人
#[tauri::command]
pub fn create_account_owner(
    state: State<'_, DatabaseState>,
    name: String,
) -> Result<AccountOwner, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("归属人名称不能为空".to_string());
    }
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    account_owner_dao::create_owner(&conn, name)
}

/// 修改归属人名称
#[tauri::command]
pub fn update_account_owner(
    state: State<'_, DatabaseState>,
    id: String,
    name: String,
) -> Result<AccountOwner, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("归属人名称不能为空".to_string());
    }
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    account_owner_dao::update_owner(&conn, &id, name)
}

/// 删除归属人
/// 注意：如果该归属人已关联交易记录，交易的 payer 字段将变为空
#[tauri::command]
pub fn delete_account_owner(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    account_owner_dao::delete_owner(&conn, &id)
}
