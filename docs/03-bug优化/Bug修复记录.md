# Bug 修复记录

> 本文档记录开发过程中遇到的所有 bug 及其修复方案，供后续 AI 参考，避免重复踩坑。

---

## 一、Rust 后端

### 1.1 replace_all 导致代码重复插入

**现象**：使用 Edit 工具 `replace_all: true` 时，匹配到的多个位置都被替换，导致函数体被重复插入多次。

**根因**：`replace_all: true` 会替换文件中所有匹配的字符串，而非仅末尾一处。

**修复**：
- 优先用**唯一上下文**定位目标位置（如用函数签名 + 周围代码片段，而非通用结尾模式）
- 如果多处相同，用更长的唯一前缀/后缀区分
- 受影响文件已达到破坏性状态时，直接 `Write` 重写整个文件

**影响文件**：`src-tauri/src/dao/transaction_dao.rs`（两次）

---

### 1.2 MutexGuard 跨 async 边界报错

**现象**：`#[tauri::command] async fn` 中，`state.conn.lock()?` 返回的 `MutexGuard` 不能跨越 `.await` 点，编译报错 `future is not Send`。

**根因**：Tauri async 命令要求 future 实现 `Send`，而 `MutexGuard` 是 `!Send` 的。

**修复**：在 await 之前手动 drop MutexGuard。将需要的数据从锁内提取到局部变量，锁自动释放后再执行 async 操作。模式：

```rust
#[tauri::command]
pub async fn my_command(state: State<'_, DatabaseState>) -> Result<...> {
    let (data1, data2) = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        // 在锁内完成所有同步操作
        let d1 = dao::query1(&conn)?;
        let d2 = dao::query2(&conn)?;
        (d1, d2)
    }; // 锁在此释放
    // 现在可以安全 await
    some_async_fn(data1, data2).await
}
```

**影响文件**：`src-tauri/src/commands/ai_commands.rs`

---

### 1.3 rusqlite stmt 生命周期问题

**现象**：`stmt.query_map(...).collect()` 链式调用报错 `stmt does not live long enough`。

**根因**：`query_map` 返回的迭代器借用了 `stmt`，在块表达式末尾 `stmt` 先于临时迭代器销毁。

**修复**：将中间结果保存到具名变量后再 collect：

```rust
// ❌ 错误
stmt.query_map(params![m], mapper)
    .map_err(...)?.collect()

// ✅ 正确
let rows: Vec<T> = stmt.query_map(params![m], mapper)
    .map_err(...)?.collect::<Result<Vec<_>, _>>()?;
rows
```

**影响文件**：`src-tauri/src/services/report_service.rs`、`src-tauri/src/dao/transaction_dao.rs`

---

### 1.4 数据清洗遗漏微信已全额退款状态

**现象**：微信账单中「已全额退款」状态未被清洗过滤。

**根因**：`EXCLUDE_STATUSES` 常量只包含 `["还款成功", "交易关闭", "退款成功", "不计收入"]`，缺少微信的 `"已全额退款"`。

**修复**：在 `services/cleaning_service.rs` 的常量中添加 `"已全额退款"`。

**影响文件**：`src-tauri/src/services/cleaning_service.rs`

---

### 1.5 tag_dao.rs match 语句结构被破坏

**现象**：Edit 工具替换 `Ok(None)` 时，只替换了模式匹配中的一部分，导致 `=> Ok(None), Err(e) => ...` 的语法被破坏。

**根因**：`Ok(None)` 在 `match rows.next()` 块中出现，替换时未保留完整的 `=> Ok(None),` 后缀。

**修复**：替换时包含完整的匹配分支上下文：
```rust
match rows.next() {
    Ok(Some(row)) => { ... }
    Ok(None) => Ok(None),
    Err(e) => Err(...),
}
```

**影响文件**：`src-tauri/src/dao/tag_dao.rs`

---

## 二、React 前端

### 2.1 MUI v5 与 v6 API 冲突

**现象**：使用 `Grid size={{ xs: 12 }}` 和 `TextField slotProps` 时 TypeScript 报错。

**根因**：项目使用 MUI **v5.18**，但代码用了 v6 的 API（`size` prop、`slotProps`）。

**修复**：
- Grid: 用 `item xs={12} sm={6}` 代替 `size={{ xs: 12, sm: 6 }}`
- TextField: 用 `inputProps={{ step: "0.01" }}` 代替 `slotProps={{ htmlInput: ... }}`

**影响文件**：`src/components/transactions/ManualEntryForm.tsx`

---

### 2.2 AG Grid 选中行后列宽还原

**现象**：勾选复选框后，之前手动调整的列宽恢复为默认值。

**根因**：传给 `AgGridReact` 的 `rowSelection`、`defaultColDef`、`getRowId`、`getRowClass`、`onSelectionChanged` 都是**新对象/新函数引用**（每次 render 重新创建），导致 AG Grid 检测到 prop 变化并重置内部状态。

**修复**：用 `useMemo` / `useCallback` 记忆化所有 AG Grid props：

```tsx
const rowSelection = useMemo(() => ({ mode: "multiRow" }), []);
const defaultColDef = useMemo(() => ({ sortable: true, ... }), []);
const getRowId = useCallback((p) => p.data.id, []);
const getRowClass = useCallback((p) => ..., []);
const onSelectionChanged = useCallback((e) => setSelectedRows(...), []);
```

**影响文件**：`src/pages/TransactionsPage.tsx`

---

### 2.3 AG Grid 出现两列复选框

**现象**：表格左侧出现两个复选框列。

**根因**：`rowSelection` 的 `headerCheckbox: true` 和列定义中的 `headerCheckboxSelection: true` 同时起作用，各自添加了一个全选复选框。

**修复**：`rowSelection` 不设置 `headerCheckbox`，仅通过列定义的 `headerCheckboxSelection: true` 控制。同时设 `rowSelection.headerCheckbox: false`。

**影响文件**：`src/pages/TransactionsPage.tsx`

---

### 2.4 cellRenderer HTML 被转义

**现象**：AG Grid 单元格显示 HTML 源码而非渲染后的样式。

**根因**：`cellRenderer` 返回的 HTML 字符串被 AG Grid 当作文本转义。

**修复**：改用 React 组件作为 cellRenderer（不会被转义）：
```tsx
const TagChipRenderer = (p: { value: string }) => (
  <span style={{ background: TAG_COLORS[p.value], ... }}>{p.value}</span>
);
// 列定义中：cellRenderer: TagChipRenderer
```

**影响文件**：`src/pages/TransactionsPage.tsx`

---

### 2.5 AG Grid 右键菜单不触发

**现象**：在表格上右键没有弹出自定义菜单。

**根因**：`onContextMenu` 事件挂在外层 `<Box>` 上，但 AG Grid 内部的 DOM 事件被拦截，不会冒泡到外层。

**修复**：使用 AG Grid 的 `onCellContextMenu` 回调代替 DOM 事件：
```tsx
<AgGridReact onCellContextMenu={handleContextMenu} suppressContextMenu={true} />
```

**影响文件**：`src/pages/TransactionsPage.tsx`

---

### 2.6 右键菜单弹两个

**现象**：右键后出现两个菜单（浏览器原生 + 自定义）。

**根因**：`suppressContextMenu` 只禁用了 AG Grid 内置菜单，浏览器原生右键菜单仍需单独阻止。

**修复**：在外层容器添加 `onContextMenu={(e) => e.preventDefault()}`。

**影响文件**：`src/pages/TransactionsPage.tsx`

---

### 2.7 agSetColumnFilter 不工作

**现象**：设置 `filter: "agSetColumnFilter"` 后筛选 UI 无变化。

**根因**：`agSetColumnFilter` 是 AG Grid **Enterprise** 功能，Community 版不支持。

**修复**：Community 版只能用 `agTextColumnFilter` 或 `agNumberColumnFilter`。如需下拉选择，需自行编写自定义 `floatingFilterComponent`。

**影响文件**：`src/pages/TransactionsPage.tsx`

---

### 2.8 position: sticky 不生效

**现象**：批量操作栏设置 `position: sticky; top: 0` 但不随滚动固定。

**根因**：`sticky` 要求父容器有**滚动上下文**（`overflow: auto/scroll` + 固定高度）。页面的滚动在 body 层级，不在该容器内。

**修复**：外层容器设 `height: calc(100vh - 120px)` + `display: flex; flexDirection: column`，内层包裹表格的 Box 设 `overflow: auto; flex: 1`，使滚动发生在内层，sticky 生效。

**影响文件**：`src/pages/ReviewPage.tsx`

---

### 2.9 手动数据子项输入框自动消失/提交

**现象**：在子项名称框输入一个字符就消失变 Chip；或在金额框输入就自动提交。

**根因**：
- 第一版：用 `item.name && item.amount` 判断是否转 Chip，导致任一字段有值就转换
- 第二版：加 `done` 标志位后用 `onBlur` 触发确认，但切换到第二个输入框时第一个的 `onBlur` 触发，导致提前确认

**修复**：只用 `done` 标志位控制转换，**去掉 onBlur**，仅保留 **Enter 键**确认：
```tsx
onKeyDown={(e) => { if (e.key === "Enter") { e.preventDefault(); confirmItem(); } }}
```

**影响文件**：`src/components/transactions/ManualEntryForm.tsx`

---

### 2.10 Recharts 图表白屏

**现象**：AI 生成的图表渲染时整个页面白屏。

**根因**：React 渲染错误未被捕获导致整个组件树崩溃。`ResponsiveContainer` 需要父容器有显式宽高。

**修复**：
1. 创建独立的 `ChartViewer` 组件
2. 使用 React `ErrorBoundary` 类组件包裹图表渲染
3. 给图表容器设固定宽高（`width: 100%; height: 380px`）

**影响文件**：`src/components/reports/ChartViewer.tsx`、`src/pages/ReportsPage.tsx`

---

### 2.11 自定义浮动筛选器不生效

**现象**：归属人列的自定义下拉筛选器选值后不筛选。

**根因**：`filter: PayerFilter` 作为 `IFilterComp` 使用时，需要通过 `props.filterChangedCallback()` 或 `onFilterChanged` 通知 AG Grid。初次实现未正确调用。

**修复**：改为 `floatingFilterComponent` 模式，通过 `props.parentFilterInstance(cb)` 获取父过滤器实例，调用 `inst.onFloatingFilterChanged(type, value)` 触发筛选。

**影响文件**：`src/components/transactions/PayerFilter.tsx`

---

### 2.12 autoSizeAllColumns 首次不生效

**现象**：交易明细列宽不自动适配内容。

**根因**：
- `useRef` 标志位在组件卸载后不重置（React strict mode 下可能渲染两次）
- localStorage 已有旧的列宽保存数据，`hasSaved` 检查导致跳过 autoSize
- `autoSizeAllColumns` 在 `onFirstDataRendered` 时可能 grid 尚未完全渲染

**修复**：简化逻辑——无保存时直接调用 `api.autoSizeAllColumns(false)`，有保存时延迟 50ms 后用保存数据覆盖。更换 localStorage key 清除旧数据。

**影响文件**：`src/pages/TransactionsPage.tsx`

---

### 2.13 批量操作后页面跳动

**现象**：删除或标记后页面跳回顶部。

**根因**：`handleBatchDelete` / `handleBatchRigid` 等操作后调用 `loadData()`，触发全量重新加载和 AG Grid 重新渲染，滚动位置丢失。

**修复**：用本地 state 更新代替 `loadData()`：
```tsx
setTransactions((p) => p.filter((t) => !ids.has(t.id)));
setSelectedRows([]);
```

**影响文件**：`src/pages/TransactionsPage.tsx`

---

## 三、开发教训

### 3.1 Edit 工具使用原则
- `replace_all: false` 优先，避免意外多处替换
- 如必须用 `replace_all: true`，确认替换上下文足够唯一
- 替换后立即 `cargo check` / `tsc` 验证

### 3.2 AG Grid Community 限制
- `agSetColumnFilter` → Enterprise only
- `agAggregationComponent` → Enterprise only
- `statusBar` 的聚合面板 → Enterprise only
- 自定义 `floatingFilterComponent` → Community 可用
- CSV 导出 → Community 可用

### 3.3 Tauri 异步命令注意事项
- 不要在 `MutexGuard` 有效期间 `.await`
- 提取数据 → drop 锁 → 再执行异步操作

### 3.4 AG Grid React props 稳定性
- 所有传给 `AgGridReact` 的对象/函数必须用 `useMemo`/`useCallback`
- 新的引用会导致 AG Grid 重置内部状态（列宽、排序、筛选）
