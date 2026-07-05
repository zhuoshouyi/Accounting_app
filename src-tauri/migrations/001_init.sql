-- 001_init.sql — 初始化建表脚本
-- 包含 10 张表的创建语句和索引
-- 数据库路径: ~/Library/Application Support/family-accounting-app/database.db

PRAGMA foreign_keys = ON;

-- ============================================================
-- 表 1：transactions（交易明细表）
-- ============================================================
CREATE TABLE IF NOT EXISTS transactions (
    id TEXT PRIMARY KEY,
    transaction_time TEXT NOT NULL,
    tag_id TEXT,
    tag_source TEXT,
    source TEXT NOT NULL,
    transaction_type TEXT,
    counterparty TEXT,
    product TEXT,
    direction TEXT NOT NULL,
    amount REAL NOT NULL,
    payment_method TEXT,
    status TEXT,
    payer TEXT,
    payer_source TEXT,
    is_rigid INTEGER,
    is_excluded_from_summary INTEGER DEFAULT 0,
    ai_suggested_tag TEXT,
    ai_confidence REAL,
    payment_purpose TEXT,
    month TEXT NOT NULL,
    raw_data TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (tag_id) REFERENCES category_tags(id),
    FOREIGN KEY (payer) REFERENCES account_owners(name),
    FOREIGN KEY (ai_suggested_tag) REFERENCES category_tags(id)
);

CREATE INDEX IF NOT EXISTS idx_transactions_month ON transactions(month);
CREATE INDEX IF NOT EXISTS idx_transactions_tag_id ON transactions(tag_id);
CREATE INDEX IF NOT EXISTS idx_transactions_payer ON transactions(payer);
CREATE INDEX IF NOT EXISTS idx_transactions_time ON transactions(transaction_time);
CREATE INDEX IF NOT EXISTS idx_transactions_direction ON transactions(direction);
CREATE INDEX IF NOT EXISTS idx_transactions_excluded ON transactions(is_excluded_from_summary);
CREATE INDEX IF NOT EXISTS idx_transactions_status ON transactions(status);

-- ============================================================
-- 表 2：account_owners（归属人表）
-- ============================================================
CREATE TABLE IF NOT EXISTS account_owners (
    id TEXT PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    sort_order INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- ============================================================
-- 表 3：category_rules（分类规则表）
-- ============================================================
CREATE TABLE IF NOT EXISTS category_rules (
    id TEXT PRIMARY KEY,
    match_field TEXT NOT NULL,
    match_type TEXT NOT NULL,
    match_value TEXT NOT NULL,
    target_tag_id TEXT NOT NULL,
    priority INTEGER DEFAULT 100,
    enabled INTEGER DEFAULT 1,
    source TEXT DEFAULT 'builtin',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (target_tag_id) REFERENCES category_tags(id)
);

CREATE INDEX IF NOT EXISTS idx_rules_enabled ON category_rules(enabled);
CREATE INDEX IF NOT EXISTS idx_rules_priority ON category_rules(priority);
CREATE INDEX IF NOT EXISTS idx_rules_source ON category_rules(source);

-- ============================================================
-- 表 4：category_tags（消费标签表）
-- ============================================================
CREATE TABLE IF NOT EXISTS category_tags (
    id TEXT PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    is_system INTEGER DEFAULT 1,
    sort_order INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- ============================================================
-- 表 5：summary_mappings（汇总类映射表）
-- ============================================================
CREATE TABLE IF NOT EXISTS summary_mappings (
    id TEXT PRIMARY KEY,
    summary_category TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    sort_order INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    FOREIGN KEY (tag_id) REFERENCES category_tags(id)
);

-- ============================================================
-- 表 6：monthly_manual_data（月度手动数据表）
-- ============================================================
CREATE TABLE IF NOT EXISTS monthly_manual_data (
    id TEXT PRIMARY KEY,
    month TEXT UNIQUE NOT NULL,
    total_assets REAL,
    joey_income REAL,
    vila_income REAL,
    mortgage_savings REAL,
    investment REAL,
    insurance REAL,
    analysis_text TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- ============================================================
-- 表 7：import_records（导入记录表）
-- ============================================================
CREATE TABLE IF NOT EXISTS import_records (
    id TEXT PRIMARY KEY,
    month TEXT,
    source TEXT NOT NULL,
    payer TEXT,
    file_name TEXT NOT NULL,
    file_hash TEXT,
    account_info TEXT,
    total_count INTEGER,
    valid_count INTEGER,
    filtered_count INTEGER,
    imported_at TEXT NOT NULL,
    FOREIGN KEY (payer) REFERENCES account_owners(name)
);

-- ============================================================
-- 表 8：ai_learning_rules（AI 学习规则表）
-- ============================================================
CREATE TABLE IF NOT EXISTS ai_learning_rules (
    id TEXT PRIMARY KEY,
    match_field TEXT NOT NULL,
    match_value TEXT NOT NULL,
    match_type TEXT NOT NULL,
    target_tag_id TEXT NOT NULL,
    confidence REAL DEFAULT 0.5,
    confirm_count INTEGER DEFAULT 0,
    correct_count INTEGER DEFAULT 0,
    source TEXT,
    enabled INTEGER DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (target_tag_id) REFERENCES category_tags(id)
);

CREATE INDEX IF NOT EXISTS idx_ai_rules_field_value ON ai_learning_rules(match_field, match_value);
CREATE INDEX IF NOT EXISTS idx_ai_rules_enabled ON ai_learning_rules(enabled);

-- ============================================================
-- 表 9：ai_reports（AI 报表历史表）
-- ============================================================
CREATE TABLE IF NOT EXISTS ai_reports (
    id TEXT PRIMARY KEY,
    month TEXT NOT NULL,
    report_type TEXT NOT NULL,
    title TEXT,
    content TEXT,
    summary_data TEXT,
    model_name TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_reports_month ON ai_reports(month);
CREATE INDEX IF NOT EXISTS idx_reports_type ON ai_reports(report_type);
CREATE INDEX IF NOT EXISTS idx_reports_created ON ai_reports(created_at);

-- ============================================================
-- 表 10：app_settings（应用设置表）
-- ============================================================
CREATE TABLE IF NOT EXISTS app_settings (
    key TEXT PRIMARY KEY,
    value TEXT,
    description TEXT,
    updated_at TEXT NOT NULL
);
