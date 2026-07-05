// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// 应用入口函数
/// 调用 lib 中的 run() 启动 Tauri 应用
fn main() {
    family_accounting_app_lib::run();
}
