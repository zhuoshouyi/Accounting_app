import { invokeCommand } from "./client";

/** AI 分类建议结果 */
export interface AiClassifyResult {
  tag_name: string;
  confidence: number;
  reason: string;
}

/** AI 生成结果（分析文本 / HTML 报表） */
export interface AiGenerateResult {
  content: string;
  model_name: string;
}

/** 执行 AI 辅助分类，返回 (transaction_id, result) 列表 */
export async function aiClassify(): Promise<[string, AiClassifyResult][]> {
  return invokeCommand<[string, AiClassifyResult][]>("ai_classify");
}

/** AI 月度分析 */
export async function aiAnalyze(month: string): Promise<AiGenerateResult> {
  return invokeCommand<AiGenerateResult>("ai_analyze", { month });
}

/** AI 生成 HTML 报表 */
export async function aiGenerateReport(month: string): Promise<AiGenerateResult> {
  return invokeCommand<AiGenerateResult>("ai_generate_report", { month });
}

/** 测试 AI 连接 */
export async function testAiConnection(): Promise<boolean> {
  return invokeCommand<boolean>("test_ai_connection");
}

/** AI 生成图表配置 */
export async function aiGenerateChart(prompt: string): Promise<string> {
  return invokeCommand<string>("ai_generate_chart", { prompt });
}

/** 采纳 AI 建议 → 正式分类 + 生成学习规则 */
export async function confirmAiTag(
  id: string, tagId: string, counterparty: string, product: string, transactionType: string, amount: number
): Promise<void> {
  return invokeCommand<void>("confirm_ai_tag", { id, tagId, counterparty, product, transactionType, amount });
}

/** 修正 AI 建议 → 改用指定标签 + 生成学习规则 */
export async function correctAiTag(
  id: string, tagId: string, counterparty: string, product: string, transactionType: string, amount: number
): Promise<void> {
  return invokeCommand<void>("correct_ai_tag", { id, tagId, counterparty, product, transactionType, amount });
}
