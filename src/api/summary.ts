import { invokeCommand } from "./client";
import type { Transaction } from "../types";

// ====================================================================
// 汇总相关类型定义
// ====================================================================

/** 标签级汇总 */
export interface TagSummary {
  tag_id: string;
  tag_name: string;
  amount: number;
  count: number;
}

/** 汇总类（合并后） */
export interface SummaryCategory {
  summary_category: string;
  total_amount: number;
  total_count: number;
  tags: TagSummary[];
  sort_order: number;
}

/** 多月份汇总行 */
export interface MonthlySummaryRow {
  month: string;
  total_assets: number | null;
  joey_income: number | null;
  vila_income: number | null;
  total_expense: number;
  mortgage_savings: number | null;
  categories: SummaryCategory[];
  investment: number | null;
  insurance: number | null;
  analysis_text: string | null;
  details_json: string | null;
}

/** 月度汇总完整结果 */
export interface MonthlySummary {
  month: string;
  categories: SummaryCategory[];
  total_expense: number;
  transaction_count: number;
}

// ====================================================================
// API 函数
// ====================================================================

/** 获取全部月份汇总透视表 */
export async function getAllMonthsSummary(): Promise<MonthlySummaryRow[]> {
  return invokeCommand<MonthlySummaryRow[]>("get_all_months_summary");
}

/**
 * 获取月度汇总
 *
 * @param month - 月份（YYYY-MM 格式，如 "2026-06"）
 * @returns 月度汇总数据（含各汇总类金额、子标签明细、总支出）
 */
export async function getMonthlySummary(
  month: string
): Promise<MonthlySummary> {
  return invokeCommand<MonthlySummary>("get_monthly_summary", { month });
}

/**
 * 按月份和标签列表查询交易明细（供汇总下钻跳转使用）
 *
 * @param month - 月份（YYYY-MM）
 * @param tagIds - 标签 ID 列表（为空时返回当月全部有效支出）
 * @returns 符合条件的交易列表
 */
export async function getTransactionsByTags(
  month: string,
  tagIds: string[]
): Promise<Transaction[]> {
  return invokeCommand<Transaction[]>("get_transactions_by_tags", {
    month,
    tagIds,
  });
}
