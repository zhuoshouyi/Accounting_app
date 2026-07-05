# 家庭记账桌面应用

> 将个人独有的 Excel 记账方式转化为 Mac 桌面应用。

## 技术栈

| 层 | 技术 | 版本 |
|----|------|------|
| 桌面框架 | Tauri | 2.x |
| 前端框架 | React | 18.x |
| 类型系统 | TypeScript | 5.x |
| 构建工具 | Vite | 5.x |
| UI 组件库 | MUI (Material-UI) | 5.x |
| CSS 方案 | Tailwind CSS | 3.x |
| 数据表格 | AG Grid Community | 32.x |
| 图表库 | Recharts | 2.x |
| 状态管理 | Zustand | 4.x |
| 路由 | React Router | 6.x |
| 后端语言 | Rust | 1.75+ |
| 数据库 | SQLite (rusqlite) | - |

## 目录结构

```
Accounting_app/
├── docs/                    # 文档目录
│   ├── PRD.md               # 产品需求文档
│   ├── 实现方案.md           # 技术实现方案
│   ├── 使用步骤.md           # Excel 记账使用步骤
│   ├── class-diagram.mermaid # 类图
│   └── sequence-diagram.mermaid # 时序图
├── src/                     # 前端源码
│   ├── main.tsx             # React 入口
│   ├── App.tsx              # 根组件 + 路由
│   ├── components/          # 公共组件
│   ├── pages/               # 页面组件
│   ├── api/                 # API 封装
│   ├── types/               # 类型定义
│   └── styles/              # 全局样式
├── src-tauri/               # Rust 后端
│   ├── src/
│   │   ├── main.rs          # 应用入口
│   │   ├── lib.rs           # 模块注册
│   │   ├── db/              # 数据库模块
│   │   ├── commands/        # Tauri 命令
│   │   └── models/          # 数据模型
│   ├── migrations/          # 数据库迁移 SQL
│   ├── icons/               # 应用图标
│   ├── Cargo.toml           # Rust 依赖
│   └── tauri.conf.json      # Tauri 配置
├── index.html               # HTML 入口
├── package.json             # 前端依赖
├── vite.config.ts           # Vite 配置
├── tsconfig.json            # TypeScript 配置
├── tailwind.config.ts       # Tailwind 配置
└── README.md                # 本文件
```

## 开发环境

### 前置条件

- Node.js 18+
- Rust 1.75+ (安装: https://rustup.rs)
- Tauri 2 CLI

### 安装与运行

```bash
# 安装前端依赖
npm install

# 开发模式运行（同时启动 Vite 和 Tauri）
npm run tauri dev

# 构建生产版本
npm run tauri build
```

### 数据库

数据库文件位置: `~/Library/Application Support/family-accounting-app/database.db`

应用启动时自动创建数据库、建表、插入初始数据。

## 功能模块

| 模块 | 路由 | 说明 |
|------|------|------|
| 导入账单 | /import | 导入微信/支付宝账单文件 |
| 数据清洗 | /cleaning | 清洗和标准化导入数据 |
| 交易明细 | /transactions | 查看所有交易记录 |
| 人工复核 | /review | 人工复核分类结果 |
| 月度汇总 | /summary | 月度收支汇总 |
| 报表中心 | /reports | 图表分析 |
| 规则管理 | /rules | 分类规则管理 |
| 归属人管理 | /owners | 账户归属人管理 |
| 设置 | /settings | 应用设置 |
