import { invokeCommand } from "./client";
import type { CategoryRule, CategoryTag, AiLearningRule, SummaryMapping } from "../types";

// ---- 分类规则 ----
export async function listAllRules(): Promise<CategoryRule[]> {
  return invokeCommand<CategoryRule[]>("list_all_rules");
}
export async function createRule(rule: CategoryRule): Promise<CategoryRule> {
  return invokeCommand<CategoryRule>("create_rule", { rule });
}
export async function updateRule(rule: CategoryRule): Promise<void> {
  return invokeCommand<void>("update_rule", { rule });
}
export async function deleteRule(id: string): Promise<void> {
  return invokeCommand<void>("delete_rule", { id });
}
export async function toggleRule(id: string, enabled: boolean): Promise<void> {
  return invokeCommand<void>("toggle_rule", { id, enabled });
}

// ---- AI 学习规则 ----
export async function listAiRules(): Promise<AiLearningRule[]> {
  return invokeCommand<AiLearningRule[]>("list_ai_rules");
}
export async function deleteAiRule(id: string): Promise<void> {
  return invokeCommand<void>("delete_ai_rule", { id });
}

// ---- 标签 ----
export async function createTag(name: string, sortOrder: number): Promise<CategoryTag> {
  return invokeCommand<CategoryTag>("create_tag", { name, sortOrder });
}
export async function updateTag(id: string, name: string, sortOrder: number): Promise<void> {
  return invokeCommand<void>("update_tag", { id, name, sortOrder });
}
export async function deleteTag(id: string): Promise<void> {
  return invokeCommand<void>("delete_tag", { id });
}

// ---- 汇总映射 ----
export async function listMappings(): Promise<SummaryMapping[]> {
  return invokeCommand<SummaryMapping[]>("list_mappings");
}
export async function createMapping(summaryCategory: string, tagId: string, sortOrder: number): Promise<SummaryMapping> {
  return invokeCommand<SummaryMapping>("create_mapping", { summaryCategory, tagId, sortOrder });
}
export async function deleteMapping(id: string): Promise<void> {
  return invokeCommand<void>("delete_mapping", { id });
}

// ---- 规则测试 ----
export async function testRuleMatch(counterparty: string, product: string, transactionType: string): Promise<string | null> {
  return invokeCommand<string | null>("test_rule_match", { counterparty, product, transactionType });
}
