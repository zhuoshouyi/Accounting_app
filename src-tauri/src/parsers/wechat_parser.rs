// 微信账单解析器 — 解析微信支付 XLSX 流水文件
//
// 文件格式：
// - 前 17 行（index 0-16）是头部信息
// - 第 18 行（index 17）是表头
// - 第 19 行（index 18）起是数据
//
// 表头列：交易时间, 交易类型, 交易对方, 商品, 收/支, 金额(元), 支付方式,
//         当前状态, 交易单号, 商户单号, 备注

use calamine::{open_workbook_auto, Data, DataType, Reader};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use super::{extract_month, ParsedTransaction, WechatParseResult};

/// 微信账单表头行索引（表头在 index 17）
const HEADER_ROW_INDEX: usize = 17;

/// 解析微信 XLSX 账单文件
pub fn parse_wechat_xlsx(file_path: &str) -> Result<WechatParseResult, String> {
    // 打开 XLSX 文件
    let mut workbook = open_workbook_auto(file_path)
        .map_err(|e| format!("打开 XLSX 文件失败: {}", e))?;

    // 读取第一个工作表
    let range = workbook
        .worksheet_range_at(0)
        .ok_or_else(|| "XLSX 文件中没有工作表".to_string())?
        .map_err(|e| format!("读取工作表失败: {}", e))?;

    let rows: Vec<&[Data]> = range.rows().collect();

    if rows.len() <= HEADER_ROW_INDEX {
        return Err(format!(
            "XLSX 文件行数不足：{} 行（需要至少 {} 行）",
            rows.len(),
            HEADER_ROW_INDEX + 2
        ));
    }

    // 从头部提取账户信息（index 1: "微信昵称：[Vila]"）
    let account_info = extract_wechat_account_info(&rows);

    // 解析数据行（从 index HEADER_ROW_INDEX + 1 开始）
    let mut transactions = Vec::new();
    for (idx, row) in rows.iter().enumerate() {
        if idx <= HEADER_ROW_INDEX {
            continue;
        }

        // 跳过行数不足的行（可能是空行或尾部说明）
        if row.len() < 8 {
            continue;
        }

        match parse_wechat_row(row, idx) {
            Ok(tx) => transactions.push(tx),
            Err(e) => {
                // 检查是否为数据行解析失败（跳过非数据行如尾部说明）
                let first_cell = cell_to_string(&row[0]);
                if first_cell.starts_with("20") || first_cell.contains("交易时间") {
                    return Err(format!("第 {} 行解析失败: {}", idx + 1, e));
                }
                // 非数据行，静默跳过
                continue;
            }
        }
    }

    Ok(WechatParseResult {
        account_info,
        transactions,
    })
}

/// 从微信 XLSX 头部提取账户信息
/// index 1 行格式: "微信昵称：[Vila]"
fn extract_wechat_account_info(rows: &[&[Data]]) -> String {
    if rows.len() > 1 {
        let line = cell_to_string(&rows[1][0]);
        if let Some(name) = extract_between_brackets(&line) {
            return name;
        }
    }
    "未知".to_string()
}

/// 从字符串中提取方括号内的内容
/// "微信昵称：[Vila]" → "Vila"
fn extract_between_brackets(s: &str) -> Option<String> {
    let start = s.find('[')?;
    let end = s.rfind(']')?;
    if end > start {
        Some(s[start + 1..end].trim().to_string())
    } else {
        None
    }
}

/// 解析微信账单数据行
fn parse_wechat_row(row: &[Data], _row_idx: usize) -> Result<ParsedTransaction, String> {
    // 列定义：
    // 0: 交易时间, 1: 交易类型, 2: 交易对方, 3: 商品,
    // 4: 收/支, 5: 金额(元), 6: 支付方式, 7: 当前状态,
    // 8: 交易单号, 9: 商户单号, 10: 备注

    let transaction_time = parse_datetime_from_cell(&row[0])
        .map_err(|e| format!("交易时间解析失败: {}", e))?;

    let transaction_type = cell_to_string(&row[1]);
    let counterparty = cell_to_string(&row[2]);
    let product = cell_to_string(&row[3]);
    let direction_raw = cell_to_string(&row[4]);
    let amount = cell_to_f64(&row[5])
        .map_err(|e| format!("金额解析失败: {}", e))?;
    let payment_method = cell_to_string(&row[6]);
    let status = cell_to_string(&row[7]);

    // 交易单号、商户单号、备注
    let transaction_id = if row.len() > 8 { cell_to_string(&row[8]) } else { String::new() };
    let merchant_order_id = if row.len() > 9 { cell_to_string(&row[9]) } else { String::new() };
    let remark = if row.len() > 10 { cell_to_string(&row[10]) } else { String::new() };

    // 方向映射：支出→expense, 收入→income
    let direction = match direction_raw.as_str() {
        "支出" => "expense",
        "收入" => "income",
        _ => "neutral",
    }
    .to_string();

    // 月份
    let month = extract_month(&transaction_time);

    // 原始数据 JSON
    let raw_data = serde_json::json!({
        "transaction_id": transaction_id,
        "merchant_order_id": merchant_order_id,
        "remark": remark,
    })
    .to_string();

    Ok(ParsedTransaction {
        transaction_time,
        transaction_type,
        counterparty,
        product,
        direction,
        amount,
        payment_method,
        status,
        month,
        raw_data,
    })
}

/// 从 calamine Data 单元格解析日期时间
/// 支持 DateTime(ExcelDateTime)、DateTimeIso(String)、Float/Int(序列号)、String
fn parse_datetime_from_cell(cell: &Data) -> Result<String, String> {
    match cell {
        Data::DateTime(dt) => {
            // ExcelDateTime → as_f64() 获取序列号 → 转换为 NaiveDateTime
            let serial = dt.as_f64();
            excel_serial_to_datetime(serial)
                .map(|d| d.format("%Y-%m-%dT%H:%M:%S").to_string())
                .ok_or_else(|| format!("Excel DateTime 转换失败: serial={}", serial))
        }
        Data::DateTimeIso(s) => {
            parse_datetime_string(s.trim())
        }
        Data::Float(f) => {
            // 可能是 Excel 日期序列号
            if *f > 1.0 && *f < 100000.0 {
                excel_serial_to_datetime(*f)
                    .map(|d| d.format("%Y-%m-%dT%H:%M:%S").to_string())
                    .ok_or_else(|| format!("日期序列号转换失败: {}", f))
            } else {
                Err(format!("数值不在日期范围内: {}", f))
            }
        }
        Data::Int(i) => {
            excel_serial_to_datetime(*i as f64)
                .map(|d| d.format("%Y-%m-%dT%H:%M:%S").to_string())
                .ok_or_else(|| format!("日期序列号转换失败: {}", i))
        }
        Data::String(s) => {
            parse_datetime_string(s.trim())
        }
        Data::Empty => Err("日期时间单元格为空".to_string()),
        _ => Err(format!("无法解析日期时间: {:?}", cell)),
    }
}

/// Excel 日期序列号转 NaiveDateTime
/// Excel 1900 日期系统：序列号 1 = 1900-01-01
/// 使用 1899-12-30 作为纪元（修正 Excel 1900 闰年 bug）
fn excel_serial_to_datetime(serial: f64) -> Option<NaiveDateTime> {
    let epoch = NaiveDate::from_ymd_opt(1899, 12, 30)?;
    let days = serial.floor() as i64;
    let fractional = serial - serial.floor();
    let total_seconds = (fractional * 86400.0).round() as i64;
    let total_seconds = total_seconds.rem_euclid(86400);
    let date = epoch + chrono::Duration::days(days);
    let time = NaiveTime::from_num_seconds_from_midnight_opt(total_seconds as u32, 0)?;
    Some(NaiveDateTime::new(date, time))
}

/// 解析日期时间字符串
/// 支持多种常见格式
fn parse_datetime_string(s: &str) -> Result<String, String> {
    if s.is_empty() {
        return Err("日期时间字符串为空".to_string());
    }

    let formats = [
        "%Y-%m-%d %H:%M:%S",
        "%Y/%m/%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d",
        "%Y/%m/%d",
    ];

    for fmt in &formats {
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, fmt) {
            return Ok(dt.format("%Y-%m-%dT%H:%M:%S").to_string());
        }
        if let Ok(d) = NaiveDate::parse_from_str(s, fmt) {
            return Ok(d.format("%Y-%m-%dT00:00:00").to_string());
        }
    }

    Err(format!("无法解析日期时间字符串: '{}'", s))
}

/// 将 calamine Data 单元格转为字符串
fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.trim().to_string(),
        Data::DateTimeIso(s) => s.trim().to_string(),
        Data::DurationIso(s) => s.trim().to_string(),
        Data::Int(i) => i.to_string(),
        Data::Float(f) => {
            if f.fract() == 0.0 {
                format!("{}", *f as i64)
            } else {
                f.to_string()
            }
        }
        Data::DateTime(dt) => dt.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::Error(e) => format!("{:?}", e),
    }
}

/// 将 calamine Data 单元格转为 f64
fn cell_to_f64(cell: &Data) -> Result<f64, String> {
    // 优先使用 DataType trait 的 as_f64 方法
    if let Some(f) = cell.as_f64() {
        return Ok(f);
    }
    // 回退：从字符串解析
    if let Data::String(s) = cell {
        let cleaned = s.trim().replace(",", "").replace("\t", "");
        return cleaned
            .parse::<f64>()
            .map_err(|e| format!("'{}' - {}", s, e));
    }
    Err(format!("无法转为 f64: {:?}", cell))
}
