use tauri::Manager;

mod ai;
mod commands;
mod dao;
mod db;
mod models;
mod parsers;
mod services;

/// 启动 Tauri 应用
/// 注册插件、初始化数据库、注册命令处理器
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // 初始化数据库（创建文件、建表、插入初始数据）
            match db::init_database() {
                Ok(state) => {
                    app.manage(state);
                    println!("[DB] 数据库初始化成功");
                }
                Err(e) => {
                    eprintln!("[DB] 数据库初始化失败: {}", e);
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_info,
            commands::account_owner_commands::list_account_owners,
            commands::account_owner_commands::create_account_owner,
            commands::account_owner_commands::update_account_owner,
            commands::account_owner_commands::delete_account_owner,
            commands::import_commands::import_bill_file,
            commands::import_commands::list_import_records,
            commands::import_commands::check_duplicate_import,
            commands::cleaning_commands::preview_cleaning,
            commands::cleaning_commands::execute_cleaning,
            commands::cleaning_commands::list_transactions,
            commands::cleaning_commands::get_distinct_months,
            commands::classification_commands::classify_transactions,
            commands::classification_commands::list_unclassified,
            commands::classification_commands::update_transaction_tag,
            commands::classification_commands::batch_update_tags,
            commands::classification_commands::list_category_tags,
            commands::classification_commands::list_skipped,
            commands::classification_commands::unskip_transaction,
            commands::summary_commands::get_monthly_summary,
            commands::summary_commands::get_transactions_by_tags,
            commands::summary_commands::get_all_months_summary,
            commands::transaction_commands::save_manual_data,
            commands::transaction_commands::get_manual_data,
            commands::transaction_commands::update_transaction_payer,
            commands::transaction_commands::update_transaction_rigid,
            commands::transaction_commands::batch_update_payer,
            commands::transaction_commands::batch_update_rigid,
            commands::transaction_commands::create_manual_transaction,
            commands::transaction_commands::batch_create_transactions,
            commands::transaction_commands::batch_delete_transactions,
            commands::transaction_commands::update_transaction_field,
            commands::ai_commands::ai_classify,
            commands::ai_commands::ai_analyze,
            commands::ai_commands::ai_generate_report,
            commands::ai_commands::test_ai_connection,
            commands::ai_commands::ai_generate_chart,
            commands::ai_commands::confirm_ai_tag,
            commands::ai_commands::correct_ai_tag,
            commands::settings_commands::get_all_settings,
            commands::settings_commands::save_setting,
            commands::report_commands::save_report,
            commands::report_commands::get_report_history,
            commands::report_commands::get_report_by_id,
            commands::report_commands::get_trend_data,
            commands::rule_commands::list_all_rules,
            commands::rule_commands::create_rule,
            commands::rule_commands::update_rule,
            commands::rule_commands::delete_rule,
            commands::rule_commands::toggle_rule,
            commands::rule_commands::list_ai_rules,
            commands::rule_commands::delete_ai_rule,
            commands::rule_commands::create_tag,
            commands::rule_commands::update_tag,
            commands::rule_commands::delete_tag,
            commands::rule_commands::list_mappings,
            commands::rule_commands::create_mapping,
            commands::rule_commands::delete_mapping,
            commands::rule_commands::test_rule_match,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
