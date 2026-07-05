import { invokeCommand } from "./client";
import type { AiReport } from "../types";

/** 趋势数据 */
export interface CategoryAmount {
  name: string;
  amount: number;
}

export interface CategorySeries {
  name: string;
  data: number[];
}

export interface TrendData {
  months: string[];
  series: CategorySeries[];
}

/** 保存 AI 报表到历史 */
export async function saveReport(params: {
  month: string;
  report_type: string;
  title: string;
  content: string;
  summary_json: string;
  model_name: string;
}): Promise<AiReport> {
  return invokeCommand<AiReport>("save_report", {
    month: params.month,
    reportType: params.report_type,
    title: params.title,
    content: params.content,
    summaryJson: params.summary_json,
    modelName: params.model_name,
  });
}

/** 获取报表历史 */
export async function getReportHistory(
  month?: string
): Promise<AiReport[]> {
  return invokeCommand<AiReport[]>("get_report_history", {
    month: month ?? null,
  });
}

/** 获取单个报表详情 */
export async function getReportById(
  id: string
): Promise<AiReport | null> {
  return invokeCommand<AiReport | null>("get_report_by_id", { id });
}

/** 获取多月趋势数据（供图表使用） */
export async function getTrendData(
  months: string[]
): Promise<TrendData> {
  return invokeCommand<TrendData>("get_trend_data", { months });
}
