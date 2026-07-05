# 记账 APP 项目长期记忆

## 项目概况
- 项目名：family_accounting_app
- 目标：将用户 8 年的 Excel 月度记账流程固化为 Mac 桌面应用
- 技术栈：Tauri 2 + React + TypeScript + SQLite + AG Grid + DeepSeek AI
- 跨平台：先 macOS，后续复用做 Windows 版本

## 用户确认的关键决策
- 技术栈：Tauri 2（不用 Electron）
- 存储：SQLite
- AI 服务商：DeepSeek（用户提供 API Key）
- 归属人管理：像联系人一样只存名字，上传账单时选归属人（可跳过→空）
- AI 报表：保存历史到数据库，可随时查阅
- 微信中性交易：本期不处理，后续扩展
- 月份归属：自然归属（过 23:59:59 算第二天/下月）
- 开发方式：增量开发，做一块确认一块

## 数据清洗规则（用户手动细化，重要）
- 过滤状态：【还款成功】【交易关闭】【退款成功】【不计收入】
- 过滤【金额 ≤ 3 元】的交易
- 部分退款处理：
  - 退款 ≤ 3 → 状态改为"支付成功"（小额忽略）
  - 退款 > 3 → 状态改为"部分退款"，金额 = 支付金额 - 退款金额
- 按钮触发 → 列出待处理数据 → 用户确认 → 执行
- 从汇总明细表移除，原始导入表保留可溯源

## 真实账单格式
- 微信：XLSX 格式，前 17 行头部，第 18 行表头，第 19 行起数据
- 支付宝：CSV + GBK 编码，前 22 行头部，第 23 行表头，第 24 行起数据
- 支付宝有"不计收支"类型（余额宝收益等）
- 文件名含归属人信息（如 vila支付宝...）

## 文件位置
- 项目根目录：/Users/zhaoziwei/Documents/同步文件夹/workspace/守一 一人变现系统/Accounting_app
- PRD：docs/PRD.md
- 实现方案：docs/实现方案.md
- 使用步骤：docs/使用步骤.md
- 类图：docs/class-diagram.mermaid
- 时序图：docs/sequence-diagram.mermaid
