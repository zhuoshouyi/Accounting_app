import { invokeCommand } from "./client";
import type { Transaction, CategoryTag, ClassifyResult } from "../types";

// 重新导出类型，方便页面统一从 API 模块导入
export type { ClassifyResult };

/**
 * 分类相关 API
 */

/** 执行自动分类 — 对未分类交易执行三层匹配规则 */
export async function classifyTransactions(): Promise<ClassifyResult> {
  return invokeCommand<ClassifyResult>("classify_transactions");
}

/** 获取未分类交易列表（tag_source 为空且未排除） */
export async function listUnclassified(): Promise<Transaction[]> {
  return invokeCommand<Transaction[]>("list_unclassified");
}

/** 修改单条交易的标签 */
export async function updateTransactionTag(
  id: string,
  tagId: string | null,
  tagSource: string
): Promise<void> {
  return invokeCommand<void>("update_transaction_tag", {
    id,
    tagId,
    tagSource,
  });
}

/** 批量修改交易标签 */
export async function batchUpdateTags(
  ids: string[],
  tagId: string,
  tagSource: string
): Promise<number> {
  return invokeCommand<number>("batch_update_tags", {
    ids,
    tagId,
    tagSource,
  });
}

/** 获取所有消费标签（按 sort_order 排序） */
export async function listCategoryTags(): Promise<CategoryTag[]> {
  return invokeCommand<CategoryTag[]>("list_category_tags");
}

/** 获取已跳过的交易列表 */
export async function listSkipped(): Promise<Transaction[]> {
  return invokeCommand<Transaction[]>("list_skipped");
}

/** 恢复跳过的交易 */
export async function unskipTransaction(id: string): Promise<void> {
  return invokeCommand<void>("unskip_transaction", { id });
}
