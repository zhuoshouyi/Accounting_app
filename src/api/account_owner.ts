import { invokeCommand } from "./client";
import { AccountOwner } from "../types";

/**
 * 归属人管理 API
 */

/** 查询所有归属人 */
export async function listAccountOwners(): Promise<AccountOwner[]> {
  return invokeCommand<AccountOwner[]>("list_account_owners");
}

/** 新增归属人 */
export async function createAccountOwner(name: string): Promise<AccountOwner> {
  return invokeCommand<AccountOwner>("create_account_owner", { name });
}

/** 修改归属人名称 */
export async function updateAccountOwner(
  id: string,
  name: string
): Promise<AccountOwner> {
  return invokeCommand<AccountOwner>("update_account_owner", { id, name });
}

/** 删除归属人 */
export async function deleteAccountOwner(id: string): Promise<void> {
  return invokeCommand<void>("delete_account_owner", { id });
}
