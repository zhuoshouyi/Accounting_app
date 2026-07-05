import { invokeCommand } from "./client";
import type {
  Transaction,
  CleaningPreviewResult,
  CleaningExecuteResult,
} from "../types";

// 重新导出类型，方便页面统一从 API 模块导入
export type { CleaningPreviewResult, CleaningExecuteResult };

/**
 * 数据清洗 API
 */

/** 清洗预览 — 扫描未排除的交易，分类为待过滤和待修改 */
export async function previewCleaning(): Promise<CleaningPreviewResult> {
  return invokeCommand<CleaningPreviewResult>("preview_cleaning");
}

/** 执行清洗 — 对确认的交易执行排除和修改 */
export async function executeCleaning(
  excludeIds: string[],
  modifyIds: string[]
): Promise<CleaningExecuteResult> {
  return invokeCommand<CleaningExecuteResult>("execute_cleaning", {
    excludeIds,
    modifyIds,
  });
}

/** 查询交易列表（month 为空时查全部，按时间倒序） */
export async function listTransactions(month?: string): Promise<Transaction[]> {
  return invokeCommand<Transaction[]>("list_transactions", {
    month: month ?? null,
  });
}

/** 获取所有交易月份（去重，降序） */
export async function getDistinctMonths(): Promise<string[]> {
  return invokeCommand<string[]>("get_distinct_months");
}
