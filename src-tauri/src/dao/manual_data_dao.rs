use rusqlite::{params, Connection};
use crate::models::monthly_manual_data::MonthlyManualData;

pub fn upsert_manual_data(conn: &Connection, data: &MonthlyManualData) -> Result<MonthlyManualData, String> {
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let existing: Option<String> = conn.query_row("SELECT id FROM monthly_manual_data WHERE month = ?", params![data.month], |row| row.get(0)).ok();
    match existing {
        Some(existing_id) => {
            conn.execute(
                "UPDATE monthly_manual_data SET total_assets=?, joey_income=?, vila_income=?, mortgage_savings=?, investment=?, insurance=?, analysis_text=?, details_json=?, updated_at=? WHERE id=?",
                params![data.total_assets, data.joey_income, data.vila_income, data.mortgage_savings, data.investment, data.insurance, data.analysis_text, data.details_json, now, existing_id],
            ).map_err(|e| format!("更新手动数据失败: {}", e))?;
            Ok(MonthlyManualData { id: existing_id, month: data.month.clone(), total_assets: data.total_assets, joey_income: data.joey_income, vila_income: data.vila_income, mortgage_savings: data.mortgage_savings, investment: data.investment, insurance: data.insurance, analysis_text: data.analysis_text.clone(), details_json: data.details_json.clone(), created_at: String::new(), updated_at: now })
        }
        None => {
            let id = uuid::Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO monthly_manual_data (id, month, total_assets, joey_income, vila_income, mortgage_savings, investment, insurance, analysis_text, details_json, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![id, data.month, data.total_assets, data.joey_income, data.vila_income, data.mortgage_savings, data.investment, data.insurance, data.analysis_text, data.details_json, now, now],
            ).map_err(|e| format!("插入手动数据失败: {}", e))?;
            Ok(MonthlyManualData { id, month: data.month.clone(), total_assets: data.total_assets, joey_income: data.joey_income, vila_income: data.vila_income, mortgage_savings: data.mortgage_savings, investment: data.investment, insurance: data.insurance, analysis_text: data.analysis_text.clone(), details_json: data.details_json.clone(), created_at: now.clone(), updated_at: now })
        }
    }
}

pub fn get_manual_data(conn: &Connection, month: &str) -> Result<Option<MonthlyManualData>, String> {
    let mut stmt = conn.prepare("SELECT * FROM monthly_manual_data WHERE month = ?").map_err(|e| format!("查询手动数据失败: {}", e))?;
    let result = stmt.query_row(params![month], |row| Ok(MonthlyManualData {
        id: row.get("id")?, month: row.get("month")?,
        total_assets: row.get("total_assets")?, joey_income: row.get("joey_income")?,
        vila_income: row.get("vila_income")?, mortgage_savings: row.get("mortgage_savings")?,
        investment: row.get("investment")?, insurance: row.get("insurance")?,
        analysis_text: row.get("analysis_text")?, details_json: row.get("details_json")?,
        created_at: row.get("created_at")?, updated_at: row.get("updated_at")?,
    })).ok();
    Ok(result)
}
