/**
 * 导航工具 — 汇总下钻跳转
 *
 * 在汇总页面点击某个汇总类金额时，跳转到交易明细页面并自动筛选：
 * - month: 月份参数（YYYY-MM）
 * - tags: 该汇总类包含的标签 ID 列表（逗号分隔）
 */

/**
 * 构建汇总下钻跳转的 URL 参数
 *
 * @param month - 月份（YYYY-MM）
 * @param tagIds - 该汇总类下的标签 ID 列表
 * @returns URL search params 字符串，如 "?month=2026-06&tags=tag_maicai,tag_shuiguo"
 */
export function buildDrillDownUrl(month: string, tagIds: string[]): string {
  const params = new URLSearchParams();
  params.set("month", month);
  params.set("tags", tagIds.join(","));
  return `/transactions?${params.toString()}`;
}

/**
 * 从 URL search params 中解析筛选条件
 *
 * @param searchParams - URLSearchParams 对象
 * @returns { month: string | null, tagIds: string[] }
 */
export function parseFilterParams(searchParams: URLSearchParams): {
  month: string | null;
  tagIds: string[];
} {
  const month = searchParams.get("month");
  const tagsParam = searchParams.get("tags");
  const tagIds = tagsParam ? tagsParam.split(",").filter(Boolean) : [];
  return { month, tagIds };
}
