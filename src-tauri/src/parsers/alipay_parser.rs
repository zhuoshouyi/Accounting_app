// 支付宝账单解析器 — 解析支付宝 CSV (GBK 编码) 交易明细
//
// 文件格式：
// - 前 23 行（index 0-22）是头部信息+分隔线
// - 第 24 行（index 23）是表头
// - 第 25 行（index 24）起是数据
//
// 表头列：交易时间,交易分类,交易对方,对方账号,商品说明,收/支,金额,
//         收/付款方式,交易状态,交易订单号,商家订单号,备注
// 注意：每行末尾有额外逗号（多一个空列），部分字段含 tab 字符

use chrono::{NaiveDate, NaiveDateTime};
use csv::ReaderBuilder;
use encoding_rs::GBK;

use super::{extract_month, ParsedTransaction, AlipayParseResult};

/// 支付宝数据起始行索引（表头在 index 23，数据从 index 24 开始）
const DATA_START_INDEX: usize = 24;

/// 解析支付宝 CSV 账单文件
pub fn parse_alipay_csv(file_path: &str) -> Result<AlipayParseResult, String> {
    // 1. 读取文件字节
    let bytes = std::fs::read(file_path)
        .map_err(|e| format!("读取文件失败: {}", e))?;

    // 2. GBK 解码为 UTF-8
    let (utf8_cow, _, _) = GBK.decode(&bytes);
    let utf8_text = utf8_cow.as_ref();

    // 3. 按行分割
    let lines: Vec<&str> = utf8_text
        .lines()
        .map(|l| l.trim_end_matches('\r'))
        .collect();

    if lines.len() <= DATA_START_INDEX {
        return Err(format!(
            "CSV 文件行数不足：{} 行（需要至少 {} 行）",
            lines.len(),
            DATA_START_INDEX + 1
        ));
    }

    // 4. 提取头部账户信息
    let account_info = extract_alipay_account_info(&lines);

    // 5. 提取数据行（从 index DATA_START_INDEX 开始）
    let data_lines: Vec<&str> = lines
        .iter()
        .skip(DATA_START_INDEX)
        .filter(|s| !s.is_empty())
        .copied()
        .collect();

    // 6. 用 csv crate 解析数据行
    let data_text = data_lines.join("\n");
    let mut rdr = ReaderBuilder::new()
        .has_headers(false)
        .flexible(true) // 允许字段数不一致（末尾多一个逗号）
        .from_reader(data_text.as_bytes());

    let mut transactions = Vec::new();
    let mut line_num = DATA_START_INDEX + 1;

    for result in rdr.records() {
        let record = match result {
            Ok(r) => r,
            Err(e) => {
                // CSV 解析失败的行，可能是非数据行（如尾部说明），跳过
                eprintln!("[AlipayParser] 第 {} 行 CSV 解析失败: {}", line_num, e);
                line_num += 1;
                continue;
            }
        };

        // 检查是否为数据行（第一个字段应类似 "2026-06-30 03:21:15"）
        let time_str = record.get(0).unwrap_or("").trim();
        if time_str.is_empty() || !time_str.starts_with("20") {
            line_num += 1;
            continue;
        }

        match parse_alipay_record(&record) {
            Ok(tx) => transactions.push(tx),
            Err(e) => {
                return Err(format!("第 {} 行解析失败: {}", line_num, e));
            }
        }

        line_num += 1;
    }

    Ok(AlipayParseResult {
        account_info,
        transactions,
    })
}

/// 从支付宝 CSV 头部提取账户信息
/// index 2: "姓名：赵紫薇"
/// index 3: "支付宝账户：15730284920"
fn extract_alipay_account_info(lines: &[&str]) -> String {
    let mut name = String::new();
    let mut account = String::new();

    if lines.len() > 2 {
        if let Some(n) = extract_after_colon(lines[2]) {
            name = n;
        }
    }
    if lines.len() > 3 {
        if let Some(a) = extract_after_colon(lines[3]) {
            account = a;
        }
    }

    if name.is_empty() && account.is_empty() {
        "未知".to_string()
    } else if account.is_empty() {
        name
    } else {
        format!("{} ({})", name, account)
    }
}

/// 从字符串中提取冒号后的内容
/// "姓名：赵紫薇" → "赵紫薇"
/// "支付宝账户：15730284920" → "15730284920"
fn extract_after_colon(s: &str) -> Option<String> {
    if let Some((_, after)) = s.split_once('：') {
        return Some(after.trim().to_string());
    }
    if let Some((_, after)) = s.split_once(':') {
        return Some(after.trim().to_string());
    }
    None
}

/// 解析支付宝 CSV 数据行
fn parse_alipay_record(record: &csv::StringRecord) -> Result<ParsedTransaction, String> {
    // 列定义（取前 12 列，忽略末尾空列）：
    // 0: 交易时间, 1: 交易分类, 2: 交易对方, 3: 对方账号,
    // 4: 商品说明, 5: 收/支, 6: 金额, 7: 收/付款方式,
    // 8: 交易状态, 9: 交易订单号, 10: 商家订单号, 11: 备注

    let get_field = |idx: usize| -> String {
        record.get(idx).unwrap_or("").trim().to_string()
    };

    let time_str = get_field(0);
    let transaction_type = get_field(1);
    let counterparty = get_field(2);
    let counterparty_account = get_field(3);
    let product = get_field(4);
    let direction_raw = get_field(5);
    let amount_str = get_field(6);
    let payment_method = get_field(7);
    let status = get_field(8);
    let transaction_order_id = get_field(9);
    let merchant_order_id = get_field(10);
    let remark = get_field(11);

    // 解析交易时间: "2026-06-30 03:21:15" → "2026-06-30T03:21:15"
    let transaction_time = parse_alipay_datetime(&time_str)?;

    // 方向映射
    let direction = match direction_raw.as_str() {
        "支出" => "expense",
        "收入" => "income",
        "不计收支" => "neutral",
        _ => "neutral",
    }
    .to_string();

    // 解析金额（去掉千分位逗号）
    let amount = parse_amount(&amount_str)?;

    // 月份
    let month = extract_month(&transaction_time);

    // 原始数据 JSON
    let raw_data = serde_json::json!({
        "counterparty_account": counterparty_account,
        "transaction_order_id": transaction_order_id,
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

/// 解析支付宝交易时间字符串
/// "2026-06-30 03:21:15" → "2026-06-30T03:21:15"
fn parse_alipay_datetime(s: &str) -> Result<String, String> {
    let s = s.trim();

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

    Err(format!("无法解析交易时间: '{}'", s))
}

/// 解析金额字符串
/// 去掉千分位逗号和 tab 字符
fn parse_amount(s: &str) -> Result<f64, String> {
    let cleaned = s.trim().replace(",", "").replace("\t", "");
    cleaned
        .parse::<f64>()
        .map_err(|e| format!("金额解析失败: '{}' - {}", s, e))
}
