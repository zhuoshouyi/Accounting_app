import { invokeCommand } from "./client";

/** 获取所有设置（返回 key-value 映射） */
export async function getAllSettings(): Promise<Record<string, string>> {
  return invokeCommand<Record<string, string>>("get_all_settings");
}

/** 保存单条设置 */
export async function saveSetting(key: string, value: string): Promise<void> {
  return invokeCommand<void>("save_setting", { key, value });
}
