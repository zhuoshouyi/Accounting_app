import { invokeCommand } from "./client";
import { ImportRecord } from "../types";

/**
 * 账单导入 API
 */

/** 导入结果 */
export interface ImportResult {
  /** 数据来源（wechat / alipay） */
  source: string;
  /** 账户信息（从文件提取） */
  account_info: string;
  /** 解析出的总交易数 */
  total_count: number;
  /** 实际写入数据库的数 */
  imported_count: number;
  /** 跳过的数量 */
  skipped_count: number;
  /** 涉及的月份列表 */
  months: string[];
  /** 归属人 */
  payer: string | null;
  /** 文件名 */
  file_name: string;
}

/** 导入账单文件 */
export async function importBillFile(
  filePath: string,
  source: string,
  payer: string | null
): Promise<ImportResult> {
  return invokeCommand<ImportResult>("import_bill_file", {
    filePath,
    source,
    payer,
  });
}

/** 查询所有导入记录 */
export async function listImportRecords(): Promise<ImportRecord[]> {
  return invokeCommand<ImportRecord[]>("list_import_records");
}

/** 检查文件是否已导入过（通过 SHA-256 哈希去重） */
export async function checkDuplicateImport(filePath: string): Promise<boolean> {
  return invokeCommand<boolean>("check_duplicate_import", { filePath });
}
