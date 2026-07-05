/**
 * 基础类型定义
 * Step 1 只包含最基础的类型，后续 Step 会逐步补充
 */

/** 收支方向 */
export type Direction = "income" | "expense" | "neutral";

/** 交易记录（前端类型） */
export interface Transaction {
  id: string;
  transaction_time: string;
  tag_id: string | null;
  tag_source: string | null;
  source: string;
  transaction_type: string | null;
  counterparty: string | null;
  product: string | null;
  direction: Direction;
  amount: number;
  payment_method: string | null;
  status: string | null;
  payer: string | null;
  payer_source: string | null;
  is_rigid: number | null;
  is_excluded_from_summary: number;
  ai_suggested_tag: string | null;
  ai_confidence: number | null;
  payment_purpose: string | null;
  month: string;
  raw_data: string | null;
  created_at: string;
  updated_at: string;
}

/** 归属人 */
export interface AccountOwner {
  id: string;
  name: string;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

/** 消费标签 */
export interface CategoryTag {
  id: string;
  name: string;
  is_system: number;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

/** 分类规则 */
export interface CategoryRule {
  id: string;
  match_field: string;
  match_type: string;
  match_value: string;
  target_tag_id: string;
  priority: number;
  enabled: number;
  source: string;
  created_at: string;
  updated_at: string;
}

/** 汇总类映射 */
export interface SummaryMapping {
  id: string;
  summary_category: string;
  tag_id: string;
  sort_order: number;
  created_at: string;
}

/** 月度手动数据 */
export interface MonthlyManualData {
  id: string;
  month: string;
  total_assets: number | null;
  joey_income: number | null;
  vila_income: number | null;
  mortgage_savings: number | null;
  investment: number | null;
  insurance: number | null;
  analysis_text: string | null;
  details_json: string | null;
  created_at: string;
  updated_at: string;
}

/** 导入记录 */
export interface ImportRecord {
  id: string;
  month: string | null;
  source: string;
  payer: string | null;
  file_name: string;
  file_hash: string | null;
  account_info: string | null;
  total_count: number | null;
  valid_count: number | null;
  filtered_count: number | null;
  imported_at: string;
}

/** AI 学习规则 */
export interface AiLearningRule {
  id: string;
  match_field: string;
  match_value: string;
  match_type: string;
  target_tag_id: string;
  confidence: number;
  confirm_count: number;
  correct_count: number;
  source: string | null;
  enabled: number;
  counterparty: string | null;
  product: string | null;
  transaction_type: string | null;
  amount: number | null;
  created_at: string;
  updated_at: string;
}

/** AI 报表 */
export interface AiReport {
  id: string;
  month: string;
  report_type: string;
  title: string | null;
  content: string | null;
  summary_data: string | null;
  model_name: string | null;
  created_at: string;
}

/** 应用设置 */
export interface AppSetting {
  key: string;
  value: string | null;
  description: string | null;
  updated_at: string;
}

// ====================================================================
// 清洗相关类型（Step 3）
// ====================================================================

/** 待过滤的交易项（清洗预览） */
export interface CleaningTransactionItem {
  id: string;
  transaction_time: string;
  source: string;
  transaction_type: string | null;
  counterparty: string | null;
  product: string | null;
  direction: string;
  amount: number;
  status: string | null;
  /** 过滤原因（如 "状态:退款成功" 或 "金额≤3元（2.50元）"） */
  reason: string;
}

/** 待修改的交易项（部分退款处理） */
export interface CleaningModifyItem {
  id: string;
  transaction_time: string;
  source: string;
  counterparty: string | null;
  product: string | null;
  original_status: string | null;
  original_amount: number;
  new_status: string;
  /** 新金额（null 表示不变） */
  new_amount: number | null;
  /** 提取到的退款金额（null 表示未提取到） */
  refund_amount: number | null;
  /** 处理说明 */
  note: string;
}

/** 清洗预览结果 */
export interface CleaningPreviewResult {
  to_exclude: CleaningTransactionItem[];
  to_modify: CleaningModifyItem[];
  exclude_count: number;
  modify_count: number;
}

/** 清洗执行结果 */
export interface CleaningExecuteResult {
  excluded_count: number;
  modified_count: number;
  remaining_count: number;
}

// ====================================================================
// 分类相关类型（Step 4）
// ====================================================================

/** 自动分类结果统计 */
export interface ClassifyResult {
  /** 待分类交易总数 */
  total: number;
  /** 成功匹配的数量 */
  classified: number;
  /** 未匹配（留空）的数量 */
  unclassified: number;
}
