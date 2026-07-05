/// 数据库迁移与初始数据插入模块
/// 负责在新建数据库时插入初始数据（消费标签、汇总映射、分类规则、应用设置）

use rusqlite::Connection;

/// 插入所有初始数据（仅在新建数据库时调用）
pub fn insert_initial_data(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    insert_category_tags(conn)?;
    insert_summary_mappings(conn)?;
    insert_category_rules(conn)?;
    insert_app_settings(conn)?;
    Ok(())
}

/// 检查并补充缺失的初始数据（已存在的数据库调用）
/// 如果 category_tags 表为空，说明初始数据未插入，执行完整插入
pub fn ensure_initial_data(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let tag_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM category_tags", [], |row| row.get(0))
        .unwrap_or(0);

    if tag_count == 0 {
        println!("[DB] 检测到 category_tags 为空，补充初始数据");
        insert_initial_data(conn)?;
    }
    Ok(())
}

// ============================================================
// 消费标签（24 个）
// ============================================================
fn insert_category_tags(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let sql = r#"
INSERT INTO category_tags (id, name, is_system, sort_order, created_at, updated_at) VALUES
('tag_fangzu',        '房租',       1, 1,  datetime('now'), datetime('now')),
('tag_maicai',        '买菜',       1, 2,  datetime('now'), datetime('now')),
('tag_canyin',        '餐饮',       1, 3,  datetime('now'), datetime('now')),
('tag_dacan',         '大餐',       1, 4,  datetime('now'), datetime('now')),
('tag_shuiguo',       '水果',       1, 5,  datetime('now'), datetime('now')),
('tag_yifumeizhuang', '衣服美妆',   1, 6,  datetime('now'), datetime('now')),
('tag_lingshiyinliao','零食饮料',   1, 7,  datetime('now'), datetime('now')),
('tag_huafei',        '话费',       1, 8,  datetime('now'), datetime('now')),
('tag_jiaotong',      '交通',       1, 9,  datetime('now'), datetime('now')),
('tag_riyongpin',     '日用品',     1, 10, datetime('now'), datetime('now')),
('tag_yiliao',        '医疗药品',   1, 11, datetime('now'), datetime('now')),
('tag_jiujiu',        '九九',       1, 12, datetime('now'), datetime('now')),
('tag_huiyuan',       '会员',       1, 13, datetime('now'), datetime('now')),
('tag_yundong',       '运动',       1, 14, datetime('now'), datetime('now')),
('tag_qita',          '其他',       1, 15, datetime('now'), datetime('now')),
('tag_hongbao',       '荭包',       1, 16, datetime('now'), datetime('now')),
('tag_jiaju',         '家具',       1, 17, datetime('now'), datetime('now')),
('tag_youwan',        '游玩',       1, 18, datetime('now'), datetime('now')),
('tag_lvyou',         '旅游',       1, 19, datetime('now'), datetime('now')),
('tag_xuexi',         '学习',       1, 20, datetime('now'), datetime('now')),
('tag_liwu',          '礼物',       1, 21, datetime('now'), datetime('now')),
('tag_geiwodebao',    '给我的宝',   1, 22, datetime('now'), datetime('now')),
('tag_chezi',         '车子',       1, 23, datetime('now'), datetime('now')),
('tag_hongbei',       '烘焙',       1, 24, datetime('now'), datetime('now'));
"#;
    conn.execute_batch(sql)?;
    println!("[DB] 插入 24 条消费标签");
    Ok(())
}

// ============================================================
// 汇总类映射（16 个汇总类，24 条映射记录）
// ============================================================
fn insert_summary_mappings(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let sql = r#"
INSERT INTO summary_mappings (id, summary_category, tag_id, sort_order, created_at) VALUES
('sm_01',  '房租',       'tag_fangzu',        1,  datetime('now')),
('sm_02',  '买菜',       'tag_maicai',        2,  datetime('now')),
('sm_03',  '买菜',       'tag_shuiguo',       2,  datetime('now')),
('sm_04',  '餐饮',       'tag_canyin',        3,  datetime('now')),
('sm_05',  '交通',       'tag_jiaotong',      4,  datetime('now')),
('sm_06',  '家用',       'tag_riyongpin',     5,  datetime('now')),
('sm_07',  '家用',       'tag_yifumeizhuang', 5,  datetime('now')),
('sm_08',  '家用',       'tag_jiaju',         5,  datetime('now')),
('sm_09',  '话费',       'tag_huafei',        6,  datetime('now')),
('sm_10',  '玩的开心',   'tag_youwan',        7,  datetime('now')),
('sm_11',  '玩的开心',   'tag_lvyou',         7,  datetime('now')),
('sm_12',  '零食饮料',   'tag_lingshiyinliao',8,  datetime('now')),
('sm_13',  '学习运动',   'tag_xuexi',         9,  datetime('now')),
('sm_14',  '学习运动',   'tag_yundong',       9,  datetime('now')),
('sm_15',  '九九',       'tag_jiujiu',        10, datetime('now')),
('sm_16',  '大餐',       'tag_dacan',         11, datetime('now')),
('sm_17',  '大餐',       'tag_hongbei',       11, datetime('now')),
('sm_18',  '荭包礼物',   'tag_hongbao',       12, datetime('now')),
('sm_19',  '荭包礼物',   'tag_liwu',          12, datetime('now')),
('sm_20',  '荭包礼物',   'tag_geiwodebao',    12, datetime('now')),
('sm_21',  '医疗药品',   'tag_yiliao',        13, datetime('now')),
('sm_22',  '其他',       'tag_qita',          14, datetime('now')),
('sm_23',  '会员',       'tag_huiyuan',       15, datetime('now')),
('sm_24',  '车子',       'tag_chezi',         16, datetime('now'));
"#;
    conn.execute_batch(sql)?;
    println!("[DB] 插入 24 条汇总映射（16 个汇总类）");
    Ok(())
}

// ============================================================
// 分类规则（从用户 Excel IFS 公式迁移，含优化合并）
// ============================================================
fn insert_category_rules(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let sql = r#"
INSERT INTO category_rules (id, match_field, match_type, match_value, target_tag_id, priority, enabled, source, created_at, updated_at) VALUES
-- ========== 房租 (counterparty → tag_fangzu) ==========
('rule_001', 'counterparty', 'exact', '房东-Lyp 刘苑平 3169',           'tag_fangzu', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_002', 'counterparty', 'exact', '东莞市厚街新奥燃气有限公司',       'tag_fangzu', 100, 1, 'builtin', datetime('now'), datetime('now')),

-- ========== 买菜 (counterparty → tag_maicai) ==========
('rule_003', 'counterparty', 'exact', '灿阳妈妈',                       'tag_maicai', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_004', 'counterparty', 'exact', '重庆鲜面店',                     'tag_maicai', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_005', 'counterparty', 'exact', '家家欣超市(汉邦66店)',           'tag_maicai', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_006', 'counterparty', 'exact', '美福匠连锁生活超市',              'tag_maicai', 100, 1, 'builtin', datetime('now'), datetime('now')),

-- ========== 餐饮 (counterparty → tag_canyin) ==========
('rule_007',  'counterparty', 'exact', '美娃子',                        'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_008',  'counterparty', 'exact', '祥华美食',                      'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_009',  'counterparty', 'exact', '厚街茂枝濑粉156268050',          'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_010',  'counterparty', 'exact', '粤味石磨肠粉店',                 'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_011',  'counterparty', 'exact', '檀香叁枝',                      'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_012',  'counterparty', 'exact', '伍光芬',                        'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_013',  'counterparty', 'exact', '程结才',                        'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_014',  'counterparty', 'exact', '姜文灿',                        'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_015',  'counterparty', 'exact', '唐老鸭麻辣烫店',                 'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_016',  'counterparty', 'exact', '贺记牛肉粉',                    'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_017',  'counterparty', 'exact', '黄家一碗面',                    'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_018',  'counterparty', 'exact', '旺顺阁石磨肠粉',                 'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_019',  'counterparty', 'exact', '嘉洲鸡',                        'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_020',  'counterparty', 'exact', '道滘鱼粥',                      'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_021',  'counterparty', 'exact', '粉录记江西米粉（明丰广场店）',     'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_022',  'counterparty', 'exact', '东莞市德记烧鹅店',               'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_023',  'counterparty', 'exact', '螺友情螺蛳粉（66广场店）',        'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_024',  'counterparty', 'exact', '广西扣肉粉',                    'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_025',  'counterparty', 'exact', '大明塘',                        'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_026',  'counterparty', 'exact', '东莞喜上喜饮食店',               'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_027',  'counterparty', 'exact', '烟雨楼(川渝卤味铺)',             'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_028',  'counterparty', 'exact', '常德牛肉粉明丰店',               'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_029',  'counterparty', 'exact', '东莞市勇伯米粉店',               'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_030',  'counterparty', 'exact', '德记烧鹅（厚中店）',             'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_031',  'counterparty', 'exact', '东莞市厚街瑶汤巢餐饮店',          'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_032',  'counterparty', 'exact', '铁板炒饭',                      'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_033',  'counterparty', 'exact', '陈记香喷喷云吞',                 'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_034',  'counterparty', 'exact', '栖头鸭',                        'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_035',  'counterparty', 'exact', '黄小吖鲜卤坊',                  'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_036',  'counterparty', 'exact', '友湘厨第9分店',                 'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_037',  'counterparty', 'exact', '雅冰（潮汕美食）18718282752',     'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_038',  'counterparty', 'exact', '东莞市传承牛肉面',               'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_039',  'counterparty', 'exact', '厚街锅盔老板-姜文灿',            'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_040',  'counterparty', 'exact', '友湘厨社区厨房',                 'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_041',  'counterparty', 'exact', '麻辣烫',                        'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_042',  'counterparty', 'exact', '嘉洲丰盛食品有限公司',            'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_043',  'counterparty', 'exact', '小四川',                        'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_044',  'counterparty', 'exact', '大莞家东莞厚街万科店',            'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_045',  'counterparty', 'exact', '五谷煎饼店',                    'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_046',  'counterparty', 'exact', '荆州锅盔',                      'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_047',  'counterparty', 'exact', '锅盔',                          'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_048',  'counterparty', 'exact', '沙县小吃（金地店）',             'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_049',  'counterparty', 'exact', '兴心诚餐饮管理',                 'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_050',  'counterparty', 'exact', '川琦鲜品（深圳湾科技园店）',       'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_051',  'counterparty', 'exact', '深圳合合乐餐饮管理有限公司',       'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_052',  'counterparty', 'exact', '合合乐团餐',                    'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_053',  'counterparty', 'exact', '特色餐厅',                      'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_054',  'counterparty', 'exact', '麦当劳',                        'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),

-- ========== 餐饮 (product → tag_canyin) ==========
('rule_055', 'product', 'exact', '赵紫薇-餐费充值',                     'tag_canyin', 100, 1, 'builtin', datetime('now'), datetime('now')),

-- ========== 大餐 (counterparty → tag_dacan) ==========
('rule_056', 'counterparty', 'exact', '普雷戈创意西式料理（厚街明丰店）',   'tag_dacan', 100, 1, 'builtin', datetime('now'), datetime('now')),

-- ========== 水果 (counterparty → tag_shuiguo) ==========
('rule_057', 'counterparty', 'exact', '小艾鲜果园',                     'tag_shuiguo', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_058', 'counterparty', 'exact', '生意红火火',                     'tag_shuiguo', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_059', 'counterparty', 'exact', '雨点',                           'tag_shuiguo', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_060', 'counterparty', 'exact', '新鲜水果  小倩',                  'tag_shuiguo', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_061', 'counterparty', 'exact', '湘E果品水果店',                  'tag_shuiguo', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_062', 'counterparty', 'exact', '嘉兴果业店',                     'tag_shuiguo', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_063', 'counterparty', 'exact', '张红志',                         'tag_shuiguo', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_064', 'counterparty', 'exact', '平步青云',                       'tag_shuiguo', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_065', 'counterparty', 'exact', '众果鲜源果品',                   'tag_shuiguo', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_066', 'counterparty', 'exact', '健成水果店',                     'tag_shuiguo', 100, 1, 'builtin', datetime('now'), datetime('now')),

-- ========== 衣服美妆 (transaction_type → tag_yifumeizhuang) ==========
('rule_067', 'transaction_type', 'exact', '美容美发',                   'tag_yifumeizhuang', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_068', 'transaction_type', 'exact', '服饰装扮',                   'tag_yifumeizhuang', 100, 1, 'builtin', datetime('now'), datetime('now')),

-- ========== 衣服美妆 (counterparty → tag_yifumeizhuang) ==========
('rule_069', 'counterparty', 'exact', '深圳市宝安区西乡星明廊理发店',      'tag_yifumeizhuang', 100, 1, 'builtin', datetime('now'), datetime('now')),

-- ========== 零食饮料 (counterparty, like → tag_lingshiyinliao) [优化合并4条exact] ==========
('rule_070', 'counterparty', 'like', '零食有鸣',                        'tag_lingshiyinliao', 100, 1, 'builtin', datetime('now'), datetime('now')),

-- ========== 零食饮料 (counterparty, exact → tag_lingshiyinliao) ==========
('rule_071', 'counterparty', 'exact', '小妍子手工甜品',                  'tag_lingshiyinliao', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_072', 'counterparty', 'exact', '零食很忙',                       'tag_lingshiyinliao', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_073', 'counterparty', 'exact', '左邻右你食品（康乐南店）',          'tag_lingshiyinliao', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_074', 'counterparty', 'exact', '东莞市厚街慢酵烘焙食品店',          'tag_lingshiyinliao', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_075', 'counterparty', 'exact', '泸溪河食品（广东）有限公司',        'tag_lingshiyinliao', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_076', 'counterparty', 'exact', '泸溪河',                         'tag_lingshiyinliao', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_077', 'counterparty', 'exact', '美宜佳便利店',                   'tag_lingshiyinliao', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_078', 'counterparty', 'exact', '如家果汁店',                     'tag_lingshiyinliao', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_079', 'counterparty', 'exact', '沪上阿姨',                       'tag_lingshiyinliao', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_080', 'counterparty', 'exact', '广东赛壹便利店有限公司',            'tag_lingshiyinliao', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_081', 'counterparty', 'exact', 'luckin coffee',                 'tag_lingshiyinliao', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_082', 'counterparty', 'exact', '蜜雪冰城',                       'tag_lingshiyinliao', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_083', 'counterparty', 'exact', '东莞市厚街嘉鸣饮品店',             'tag_lingshiyinliao', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_084', 'counterparty', 'exact', '禹州**司',                       'tag_lingshiyinliao', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_085', 'counterparty', 'exact', '深圳市花栗先生零售有限公司',        'tag_lingshiyinliao', 100, 1, 'builtin', datetime('now'), datetime('now')),

-- ========== 话费 (counterparty, in → tag_huafei) [优化合并3条exact] ==========
('rule_086', 'counterparty', 'in', '中国移动,中国联通,广东联通',         'tag_huafei', 100, 1, 'builtin', datetime('now'), datetime('now')),

-- ========== 交通 (transaction_type → tag_jiaotong) ==========
('rule_087', 'transaction_type', 'exact', '交通出行',                   'tag_jiaotong', 100, 1, 'builtin', datetime('now'), datetime('now')),

-- ========== 交通 (counterparty → tag_jiaotong) ==========
('rule_088', 'counterparty', 'exact', '百度平台商家',                   'tag_jiaotong', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_089', 'counterparty', 'exact', '高德打车',                       'tag_jiaotong', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_090', 'counterparty', 'exact', '滴滴出行',                       'tag_jiaotong', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_091', 'counterparty', 'exact', '滴滴顺风车',                     'tag_jiaotong', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_092', 'counterparty', 'exact', '哈啰顺风车',                     'tag_jiaotong', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_093', 'counterparty', 'exact', '深圳通',                         'tag_jiaotong', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_094', 'counterparty', 'exact', '中铁网络',                       'tag_jiaotong', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_095', 'counterparty', 'exact', '中国铁路广州局集团有限公司',        'tag_jiaotong', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_096', 'counterparty', 'exact', '哈啰',                           'tag_jiaotong', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_097', 'counterparty', 'exact', '嘀嗒出行',                       'tag_jiaotong', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_098', 'counterparty', 'exact', '广州小猪胖胖智能科技有限公司',      'tag_jiaotong', 100, 1, 'builtin', datetime('now'), datetime('now')),

-- ========== 日用品 (counterparty → tag_riyongpin) ==========
('rule_099', 'counterparty', 'exact', '东莞市厚街家家欣日用品店',          'tag_riyongpin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_100', 'counterparty', 'exact', '伟伟便利店',                     'tag_riyongpin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_101', 'counterparty', 'exact', '大润发',                         'tag_riyongpin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_102', 'counterparty', 'exact', '住这儿',                         'tag_riyongpin', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_103', 'counterparty', 'exact', '田丽',                           'tag_riyongpin', 100, 1, 'builtin', datetime('now'), datetime('now')),

-- ========== 日用品 (product, like → tag_riyongpin) [优化合并2条exact] ==========
('rule_104', 'product', 'like', '大润发',                              'tag_riyongpin', 100, 1, 'builtin', datetime('now'), datetime('now')),

-- ========== 医疗药品 (counterparty → tag_yiliao) ==========
('rule_105', 'counterparty', 'exact', '东莞市厚街医院',                 'tag_yiliao', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_106', 'counterparty', 'exact', '叮当快药',                       'tag_yiliao', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_107', 'counterparty', 'exact', '汪群弟 医生',                    'tag_yiliao', 100, 1, 'builtin', datetime('now'), datetime('now')),

-- ========== 九九 (counterparty → tag_jiujiu) ==========
('rule_108', 'counterparty', 'exact', '爱农宠物医院',                   'tag_jiujiu', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_109', 'counterparty', 'exact', '瑞派宠友厚街医院',               'tag_jiujiu', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_110', 'counterparty', 'exact', '国泰财产保险有限责任公司',          'tag_jiujiu', 100, 1, 'builtin', datetime('now'), datetime('now')),

-- ========== 九九 (transaction_type → tag_jiujiu) ==========
('rule_111', 'transaction_type', 'exact', '宠物',                      'tag_jiujiu', 100, 1, 'builtin', datetime('now'), datetime('now')),

-- ========== 会员 (counterparty → tag_huiyuan) ==========
('rule_112', 'counterparty', 'exact', '金山WPS',                       'tag_huiyuan', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_113', 'counterparty', 'exact', '百度',                           'tag_huiyuan', 100, 1, 'builtin', datetime('now'), datetime('now')),

-- ========== 运动 (counterparty → tag_yundong) ==========
('rule_114', 'counterparty', 'exact', '闪动体育科技',                   'tag_yundong', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_115', 'counterparty', 'exact', '粗门',                           'tag_yundong', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_116', 'counterparty', 'exact', '大鹰运动社群',                   'tag_yundong', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_117', 'counterparty', 'exact', '伙卡',                           'tag_yundong', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_118', 'counterparty', 'exact', '都会幸福圈',                     'tag_yundong', 100, 1, 'builtin', datetime('now'), datetime('now')),

-- ========== 运动 (product → tag_yundong) ==========
('rule_119', 'product', 'exact', 'Latte+活动',                        'tag_yundong', 100, 1, 'builtin', datetime('now'), datetime('now')),

-- ========== 其他 (counterparty → tag_qita) ==========
('rule_120', 'counterparty', 'exact', '小猪胖胖',                       'tag_qita', 100, 1, 'builtin', datetime('now'), datetime('now')),
('rule_121', 'counterparty', 'exact', '广东睿扬大数据有限公司',            'tag_qita', 100, 1, 'builtin', datetime('now'), datetime('now')),

-- ========== 荭包 (counterparty → tag_hongbao) ==========
('rule_122', 'counterparty', 'exact', '馨馨',                           'tag_hongbao', 100, 1, 'builtin', datetime('now'), datetime('now'));
"#;
    conn.execute_batch(sql)?;
    println!("[DB] 插入 122 条分类规则");
    Ok(())
}

// ============================================================
// 应用设置（5 条）
// ============================================================
fn insert_app_settings(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let sql = r#"
INSERT INTO app_settings (key, value, description, updated_at) VALUES
('ai_provider',  'deepseek',                  'AI 服务商',                      datetime('now')),
('ai_api_key',   '',                          'DeepSeek API Key（加密存储）',    datetime('now')),
('ai_model',     'deepseek-chat',             'AI 模型名称',                    datetime('now')),
('ai_base_url',  'https://api.deepseek.com',  'DeepSeek API 地址',              datetime('now')),
('ai_enabled',   'false',                     '是否启用 AI 功能',               datetime('now'));
"#;
    conn.execute_batch(sql)?;
    println!("[DB] 插入 5 条应用设置");
    Ok(())
}
