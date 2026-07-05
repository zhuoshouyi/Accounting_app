import { invoke } from "@tauri-apps/api/core";

/**
 * Tauri invoke 封装
 * 统一处理后端命令调用，提供类型安全
 */

/**
 * 调用 Tauri 后端命令
 * @param command 命令名称
 * @param args 命令参数
 * @returns Promise<T> 后端返回结果
 */
export async function invokeCommand<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<T> {
  return invoke<T>(command, args);
}

/** 应用信息接口 */
export interface AppInfo {
  app_name: string;
  app_version: string;
  db_path: string;
  db_exists: boolean;
  table_count: number;
}

/**
 * 获取应用信息（Step 1 验证用）
 */
export async function getAppInfo(): Promise<AppInfo> {
  return invokeCommand<AppInfo>("get_app_info");
}
