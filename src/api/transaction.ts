import { invokeCommand } from "./client";
import type { MonthlyManualData } from "../types";

/**
 * 交易 & 手动数据 API
 */

// ====================================================================
// 手动数据
// ====================================================================

/** 保存月度手动数据（UPSERT） */
export async function saveManualData(params: {
  month: string;
  total_assets?: number | null;
  joey_income?: number | null;
  vila_income?: number | null;
  mortgage_savings?: number | null;
  investment?: number | null;
  insurance?: number | null;
  analysis_text?: string | null;
  details_json?: string | null;
}): Promise<MonthlyManualData> {
  return invokeCommand<MonthlyManualData>("save_manual_data", {
    month: params.month,
    totalAssets: params.total_assets ?? null,
    joeyIncome: params.joey_income ?? null,
    vilaIncome: params.vila_income ?? null,
    mortgageSavings: params.mortgage_savings ?? null,
    investment: params.investment ?? null,
    insurance: params.insurance ?? null,
    analysisText: params.analysis_text ?? null,
    detailsJson: params.details_json ?? null,
  });
}

/** 获取月度手动数据 */
export async function getManualData(
  month: string
): Promise<MonthlyManualData | null> {
  return invokeCommand<MonthlyManualData | null>("get_manual_data", { month });
}

// ====================================================================
// 字段级更新（供 AG Grid 行内编辑使用）
// ====================================================================

/** 更新单条交易的归属人 */
export async function updateTransactionPayer(
  id: string,
  payer: string | null
): Promise<void> {
  return invokeCommand<void>("update_transaction_payer", { id, payer });
}

/** 更新单条交易的刚需标记 */
export async function updateTransactionRigid(
  id: string,
  isRigid: boolean
): Promise<void> {
  return invokeCommand<void>("update_transaction_rigid", {
    id,
    isRigid,
  });
}

// ====================================================================
// 批量操作
// ====================================================================

/** 批量更新归属人 */
export async function batchUpdatePayer(
  ids: string[],
  payer: string
): Promise<number> {
  return invokeCommand<number>("batch_update_payer", { ids, payer });
}

/** 批量新增交易 */
export async function batchCreateTransactions(items: {
  transaction_time: string; amount: number; counterparty: string; product: string;
  direction: string; tag_id?: string | null; payer?: string | null; payment_method?: string | null;
}[]): Promise<number> {
  return invokeCommand<number>("batch_create_transactions", { items });
}

/** 批量删除交易 */
export async function batchDeleteTransactions(ids: string[]): Promise<number> {
  return invokeCommand<number>("batch_delete_transactions", { ids });
}

/** 更新单条交易单个字段 */
export async function updateTransactionField(id: string, field: string, value: string): Promise<void> {
  return invokeCommand<void>("update_transaction_field", { id, field, value });
}

/** 批量更新刚需标记 */
export async function batchUpdateRigid(
  ids: string[],
  isRigid: boolean
): Promise<number> {
  return invokeCommand<number>("batch_update_rigid", { ids, isRigid });
}
