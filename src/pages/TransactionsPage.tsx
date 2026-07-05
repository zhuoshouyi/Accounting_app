import { useState, useEffect, useCallback, useRef, useMemo } from "react";
import { useSearchParams } from "react-router-dom";
import { AgGridReact } from "ag-grid-react";
import type { ColDef, CellValueChangedEvent } from "ag-grid-community";
import "ag-grid-community/styles/ag-grid.css";
import "ag-grid-community/styles/ag-theme-alpine.css";
import "../styles/ag-grid.css";
import {
  Box, Typography, Alert, CircularProgress, Stack, Select, MenuItem,
  FormControl, InputLabel, Button, Snackbar, TextField, Dialog,
  DialogTitle, DialogContent, DialogActions, IconButton,
} from "@mui/material";
import AddIcon from "@mui/icons-material/Add";
import RefreshIcon from "@mui/icons-material/Refresh";
import CloseIcon from "@mui/icons-material/Close";
import DownloadIcon from "@mui/icons-material/Download";
import { listTransactions, getDistinctMonths } from "../api/cleaning";
import { listCategoryTags, updateTransactionTag, batchUpdateTags } from "../api/classification";
import { updateTransactionPayer, updateTransactionRigid, batchUpdatePayer, batchUpdateRigid, batchDeleteTransactions, updateTransactionField, batchCreateTransactions } from "../api/transaction";
import { listAccountOwners } from "../api/account_owner";
import { createRule } from "../api/rule";
import type { Transaction, CategoryTag, AccountOwner } from "../types";
import PayerFilter from "../components/transactions/PayerFilter";
import { parseFilterParams } from "../utils/navigation";

// 存储 key
const COL_STATE_KEY = "txn-grid-columns-v4";

// 标签颜色
const TAG_COLORS: Record<string, string> = {
  "房租": "#e91e63", "买菜": "#4caf50", "餐饮": "#ff9800", "大餐": "#f44336",
  "水果": "#8bc34a", "衣服美妆": "#9c27b0", "零食饮料": "#ff5722", "话费": "#2196f3",
  "交通": "#607d8b", "日用品": "#795548", "医疗药品": "#00bcd4", "九九": "#cddc39",
  "会员": "#3f51b5", "运动": "#009688", "其他": "#9e9e9e", "荭包": "#e91e63",
  "家具": "#795548", "游玩": "#ff9800", "旅游": "#03a9f4", "学习": "#4caf50",
  "礼物": "#e91e63", "给我的宝": "#ff4081", "车子": "#607d8b", "烘焙": "#ff9800",
};

// 标签 Chip 组件
const TagChipRenderer = (p: { value: string }) => {
  if (!p.value) return <span style={{ color: "#bbb", fontStyle: "italic" }}>—</span>;
  const bg = TAG_COLORS[p.value] || "#666";
  return <span style={{ display: "inline-block", padding: "2px 10px", borderRadius: 12, fontSize: 12, fontWeight: 600, color: "#fff", background: bg, whiteSpace: "nowrap", lineHeight: "20px" }}>{p.value}</span>;
};

// AG Grid 中文 locale
const ZH_LOCALE: Record<string, string> = {
  filterOoo: "搜索...",
  equals: "等于", notEqual: "不等于", contains: "包含", notContains: "不包含",
  startsWith: "开头是", endsWith: "结尾是", blank: "空白", notBlank: "非空白",
  dateFormatOoo: "yyyy-mm-dd",
  andCondition: "且", orCondition: "或",
  applyFilter: "应用", resetFilter: "重置", clearFilter: "清除",
  cancelFilter: "取消",
  textFilter: "文本筛选", numberFilter: "数字筛选", dateFilter: "日期筛选",
  lessThan: "小于", greaterThan: "大于", lessThanOrEqual: "≤", greaterThanOrEqual: "≥",
  inRange: "范围内",
  selectAll: "全选",
  pinColumn: "固定列", autosizeThiscolumn: "自动调整此列", autosizeAllColumns: "自动调整所有列",
  resetColumns: "重置列", copy: "复制", ctrlC: "Ctrl+C", copyWithHeaders: "含表头复制",
  noRowsToShow: "无数据",
  loadingOoo: "加载中...",
};

// 时间格式化
const fmtTime = (v: string | null) => {
  if (!v) return "";
  return v.replace("T", " ").substring(0, 19);
};

function TransactionsPage() {
  const gridRef = useRef<AgGridReact>(null);
  const [searchParams, setSearchParams] = useSearchParams();

  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [tags, setTags] = useState<CategoryTag[]>([]);
  const [owners, setOwners] = useState<AccountOwner[]>([]);
  const [months, setMonths] = useState<string[]>([]);
  const [selectedMonth, setSelectedMonth] = useState<string>("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [snackMsg, setSnackMsg] = useState<string | null>(null);

  const [drillMonth, setDrillMonth] = useState<string | null>(null);
  const [drillTags, setDrillTags] = useState<string[]>([]);
  const [selectedRows, setSelectedRows] = useState<Transaction[]>([]);
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number; rowData: Transaction; cellValue: string } | null>(null);
  const [batchTagId, setBatchTagId] = useState<string>("");
  const [batchPayer, setBatchPayer] = useState<string>("");

  // 新增交易弹窗
  const [addDialog, setAddDialog] = useState(false);
  const [newTxs, setNewTxs] = useState([{ transaction_time: "", amount: "", counterparty: "", product: "", direction: "expense", tag_id: "", payer: "", payment_method: "" }]);

  const handleBatchAdd = async () => {
    const items = newTxs.map((t) => ({
      transaction_time: t.transaction_time,
      amount: parseFloat(t.amount) || 0,
      counterparty: t.counterparty,
      product: t.product,
      direction: t.direction,
      tag_id: t.tag_id || null,
      payer: t.payer || null,
      payment_method: t.payment_method || null,
    })).filter((t) => t.amount > 0 && t.transaction_time);
    if (!items.length) { setError("请至少填写一条完整的交易"); return; }
    try {
      await batchCreateTransactions(items);
      setSnackMsg(`已新增 ${items.length} 条`);
      setAddDialog(false);
      setNewTxs([{ transaction_time: "", amount: "", counterparty: "", product: "", direction: "expense", tag_id: "", payer: "", payment_method: "" }]);
      loadData();
    } catch (e) { setError(String(e)); }
  };

  // 添加规则弹窗
  const [ruleDialog, setRuleDialog] = useState(false);
  const [ruleField, setRuleField] = useState("counterparty");
  const [ruleValue, setRuleValue] = useState("");
  const [ruleType, setRuleType] = useState("exact");
  const [ruleTagId, setRuleTagId] = useState("");

  const openRuleDialog = (row: Transaction) => {
    const f = row.counterparty ? "counterparty" : row.product ? "product" : "transaction_type";
    setRuleField(f);
    setRuleValue(f === "counterparty" ? (row.counterparty || "") : f === "product" ? (row.product || "") : (row.transaction_type || ""));
    setRuleType("exact");
    setRuleTagId(tags[0]?.id || "");
    setRuleDialog(true);
  };

  const handleAddRule = async () => {
    if (!ruleValue || !ruleTagId) return;
    try {
      await createRule({ id: "", match_field: ruleField, match_type: ruleType, match_value: ruleValue, target_tag_id: ruleTagId, priority: 100, enabled: 1, source: "user", created_at: "", updated_at: "" });
      setRuleDialog(false); setSnackMsg("规则已添加");
    } catch (e) { setError(String(e)); }
  };

  const tagMap = useMemo(() => { const m = new Map<string, string>(); tags.forEach((t) => m.set(t.id, t.name)); return m; }, [tags]);
  const tagNames = useMemo(() => tags.map((t) => t.name), [tags]);
  const ownerNames = useMemo(() => owners.map((o) => o.name), [owners]);

  // ---- URL ----
  useEffect(() => {
    const { month, tagIds } = parseFilterParams(searchParams);
    if (month) { setSelectedMonth(month); setDrillMonth(month); }
    setDrillTags(tagIds.length > 0 ? tagIds : []);
  }, [searchParams]);
  const clearDrillFilter = () => { setSearchParams({}); setDrillMonth(null); setDrillTags([]); };

  // ---- 数据 ----
  const loadData = useCallback(async () => {
    setLoading(true); setError(null);
    try {
      const [monthData, txData, tagData, ownerData] = await Promise.all([
        getDistinctMonths(), listTransactions(selectedMonth || undefined),
        listCategoryTags(), listAccountOwners(),
      ]);
      setMonths(monthData); setTransactions(txData); setTags(tagData); setOwners(ownerData); setSelectedRows([]);
    } catch (e) { setError(String(e)); }
    finally { setLoading(false); }
  }, [selectedMonth]);
  useEffect(() => { loadData(); }, [loadData]);

  // ---- 统计 ----
  const displayed = useMemo(() => drillTags.length > 0 ? transactions.filter((t) => drillTags.includes(t.tag_id || "")) : transactions, [transactions, drillTags]);
  const totalCount = displayed.length;
  const excludedCount = displayed.filter((t) => t.is_excluded_from_summary === 1).length;
  const totalExpense = displayed.filter((t) => t.direction === "expense" && t.is_excluded_from_summary === 0).reduce((s, t) => s + t.amount, 0);
  const totalIncome = displayed.filter((t) => t.direction === "income" && t.is_excluded_from_summary === 0).reduce((s, t) => s + t.amount, 0);

  // ---- 编辑 ----
  const handleCellValueChanged = useCallback(async (e: CellValueChangedEvent<Transaction>) => {
    const { data, colDef, newValue } = e;
    if (!data) return;
    try {
      if (colDef.field === "tag_id") {
        const tid = newValue ? tags.find((t) => t.name === newValue)?.id || null : null;
        await updateTransactionTag(data.id, tid, tid ? "manual" : "manual");
      } else if (colDef.field === "payer") {
        await updateTransactionPayer(data.id, newValue || null);
      } else if (colDef.field === "is_rigid") {
        await updateTransactionRigid(data.id, !!newValue);
      } else if (colDef.field === "product") {
        await updateTransactionField(data.id, "product", String(newValue || ""));
      } else if (colDef.field === "amount") {
        const amt = parseFloat(newValue);
        if (isNaN(amt)) { setSnackMsg("请输入有效数字"); loadData(); return; }
        await updateTransactionField(data.id, "amount", String(amt));
      }
      setSnackMsg("已更新");
    } catch (err) { setError(String(err)); loadData(); }
  }, [tags, loadData]);

  // ---- 右键 ----
  const handleContextMenu = useCallback((event: any) => {
    event.event?.preventDefault();
    if (!event.data) return;
    const target = event.event?.target as HTMLElement;
    const cellValue = target?.textContent?.trim() || "";
    setContextMenu({ x: event.event?.clientX || 0, y: event.event?.clientY || 0, rowData: event.data, cellValue });
  }, []);
  const closeContextMenu = () => setContextMenu(null);
  useEffect(() => { if (!contextMenu) return; const h = () => closeContextMenu(); document.addEventListener("click", h); return () => document.removeEventListener("click", h); }, [contextMenu]);

  // ---- 批量 ----
  const selectedIds = useMemo(() => selectedRows.map((r) => r.id), [selectedRows]);
  const selectedSum = useMemo(() => selectedRows.reduce((s, r) => s + r.amount, 0), [selectedRows]);
  const handleBatchTag = async () => { if (!batchTagId) return; await batchUpdateTags(selectedIds, batchTagId, "manual"); setBatchTagId(""); loadData(); };
  const handleBatchPayer = async () => { if (!batchPayer) return; await batchUpdatePayer(selectedIds, batchPayer); setBatchPayer(""); loadData(); };
  const handleBatchRigid = async (v: boolean) => { await batchUpdateRigid(selectedIds, v); loadData(); };
  const handleBatchDelete = async () => {
    if (!window.confirm(`确定要删除选中的 ${selectedIds.length} 条交易吗？此操作不可恢复。`)) return;
    await batchDeleteTransactions(selectedIds);
    setSnackMsg(`已删除 ${selectedIds.length} 条`);
    // 本地移除，不重载数据，保持滚动位置
    const ids = new Set(selectedIds);
    setTransactions((p) => p.filter((t) => !ids.has(t.id)));
    setSelectedRows([]);
  };

  // ---- 新增交易 ----
  // ---- 列宽持久化 ----
  const restoreFlag = useRef(true);

  const saveColumnState = useCallback(() => {
    const api = gridRef.current?.api;
    if (!api) return;
    try {
      const state = api.getColumnState();
      localStorage.setItem(COL_STATE_KEY, JSON.stringify(state));
    } catch { /* ignore */ }
  }, []);

  const restoreColumnState = useCallback(() => {
    if (!restoreFlag.current) return;
    restoreFlag.current = false;
    try {
      const saved = localStorage.getItem(COL_STATE_KEY);
      if (saved && gridRef.current?.api) {
        gridRef.current.api.applyColumnState({ state: JSON.parse(saved), applyOrder: true });
      }
    } catch { /* ignore */ }
  }, []);

  const handleFirstDataRendered = useCallback(() => {
    const api = gridRef.current?.api;
    if (!api) return;
    const hasSaved = !!localStorage.getItem(COL_STATE_KEY);
    if (!hasSaved) {
      api.autoSizeAllColumns(false);
    }
    setTimeout(() => restoreColumnState(), 50);
  }, [restoreColumnState]);

  // rowData 更新后立即恢复（选中行等操作触发 re-render 后）
  const handleRowDataUpdated = useCallback(() => {
    restoreFlag.current = true;
    setTimeout(() => restoreColumnState(), 0);
  }, [restoreColumnState]);

  // ---- 列定义 ----
  const columnDefs = useMemo<ColDef<Transaction>[]>(() => [
    { headerName: "#", width: 45, pinned: "left", lockPosition: true, suppressMenu: true, suppressMovable: true, filter: false,
      valueGetter: (p) => (p.node?.rowIndex ?? 0) + 1, cellClass: "text-gray-400", sortable: false },
    { headerName: "", width: 40, pinned: "left", lockPosition: true, suppressMenu: true, suppressMovable: true, filter: false, sortable: false,
      checkboxSelection: true, headerCheckboxSelection: true },
    { field: "transaction_time", headerName: "交易时间", width: 155, sort: "desc", sortIndex: 0,
      valueFormatter: (p) => fmtTime(p.value) },
    { field: "tag_id", headerName: "消费标签", width: 130, editable: true, cellEditor: "agSelectCellEditor",
      cellEditorParams: { values: tagNames },
      valueGetter: (p) => p.data?.tag_id ? (tagMap.get(p.data.tag_id) || "") : "",
      valueSetter: (p) => { const t = tags.find((x) => x.name === p.newValue); if (p.data) { p.data.tag_id = t?.id || null; p.data.tag_source = t ? "manual" : null; } return true; },
      cellRenderer: TagChipRenderer,
    },
    { field: "transaction_type", headerName: "交易类型", width: 100 },
    { field: "counterparty", headerName: "交易对方", width: 140 },
    { field: "product", headerName: "商品", width: 160, editable: true,
      cellRenderer: (p: { value: string | null }) => p.value && p.value.length > 25 ? p.value.substring(0, 25) + "..." : p.value || "-" },
    { field: "direction", headerName: "收/支", width: 72,
      cellRenderer: (p: { value: string }) => ({ expense: "支出", income: "收入", neutral: "不计" }[p.value] || p.value || "") },
    { field: "amount", headerName: "金额", width: 95, editable: true,
      aggFunc: "sum",
      valueFormatter: (p) => p.value != null ? `¥${Number(p.value).toFixed(2)}` : "",
      cellClass: (p) => p.data?.direction === "expense" ? "!text-red-600" : "!text-green-600" },
    { field: "status", headerName: "状态", width: 85 },
    { field: "payer", headerName: "归属人", width: 80, editable: true, cellEditor: "agSelectCellEditor",
      cellEditorParams: { values: [...ownerNames, ""] },
      valueGetter: (p) => p.data?.payer || "", valueSetter: (p) => { if (p.data) p.data.payer = p.newValue || null; return true; },
      floatingFilter: true, floatingFilterComponent: PayerFilter,
      floatingFilterComponentParams: { suppressFilterButton: true },
      filterParams: { values: [...ownerNames] } },
    { field: "is_rigid", headerName: "刚需", width: 60, editable: true, cellEditor: "agCheckboxCellEditor",
      cellRenderer: (p: { value: number | null }) => p.value === 1 ? "✓" : "",
      valueSetter: (p) => { if (p.data) p.data.is_rigid = p.newValue ? 1 : 0; return true; },
      filter: "agTextColumnFilter", filterValueGetter: (p) => p.data?.is_rigid === 1 ? "是" : "否" },
    { field: "is_excluded_from_summary", headerName: "排除", width: 60,
      cellRenderer: (p: { value: number }) => p.value === 1 ? "✓" : "",
      filter: "agTextColumnFilter", filterValueGetter: (p) => p.data?.is_excluded_from_summary === 1 ? "是" : "否" },
  ], [tagNames, tagMap, tags, ownerNames]);

  const getRowClass = useCallback((p: { data?: Transaction }) => p.data?.is_excluded_from_summary === 1 ? "row-excluded" : "", []);
  const getRowId = useCallback((p: { data: Transaction }) => p.data.id, []);
  const onSelectionChanged = useCallback((e: { api: { getSelectedRows(): Transaction[] } }) => setSelectedRows(e.api.getSelectedRows()), []);
  const defaultColDef = useMemo<ColDef>(() => ({ sortable: true, filter: true, resizable: true, suppressMovable: false, menuTabs: ["filterMenuTab"] as any,
    filterParams: { buttons: ["reset"] } }), []);
  const rowSelection: any = useMemo(() => ({ mode: "multiRow", checkboxes: false, headerCheckbox: false, selectAll: "filtered" }), []);

  return (
    <Box sx={{ height: "calc(100vh - 120px)", display: "flex", flexDirection: "column" }}>
      <Typography variant="h4" gutterBottom>交易明细</Typography>
      {error && <Alert severity="error" sx={{ mb: 1 }} onClose={() => setError(null)}>{error}</Alert>}
      <Snackbar open={!!snackMsg} autoHideDuration={2000} onClose={() => setSnackMsg(null)} message={snackMsg} anchorOrigin={{ vertical: "top", horizontal: "center" }} />

      {drillTags.length > 0 && (
        <Alert severity="info" icon={false} sx={{ mb: 1 }}
          action={<Button size="small" color="inherit" startIcon={<CloseIcon />} onClick={clearDrillFilter}>清除筛选</Button>}>
          从汇总页面跳转 — {drillMonth}，已筛选 {drillTags.length} 个标签
        </Alert>
      )}

      {/* 工具栏 */}
      <Stack direction="row" spacing={2} alignItems="center" sx={{ mb: 1 }} flexWrap="wrap">
        <FormControl sx={{ minWidth: 160 }} size="small">
          <InputLabel>月份</InputLabel>
          <Select value={selectedMonth} label="月份" onChange={(e) => setSelectedMonth(e.target.value)}>
            <MenuItem value=""><em>全部</em></MenuItem>
            {months.map((m) => <MenuItem key={m} value={m}>{m}</MenuItem>)}
          </Select>
        </FormControl>
        <Button size="small" variant="outlined" startIcon={<RefreshIcon />} onClick={loadData} disabled={loading}>刷新</Button>
        <Button size="small" variant="outlined" onClick={() => { gridRef.current?.api.setFilterModel(null); setSnackMsg("已清除所有筛选"); }}>
          清除筛选
        </Button>
        <Button size="small" variant="contained" startIcon={<AddIcon />} onClick={() => setAddDialog(true)}>新增</Button>
        <Button size="small" variant="outlined" startIcon={<DownloadIcon />}
          onClick={() => gridRef.current?.api.exportDataAsCsv({ fileName: `交易明细_${selectedMonth || "全部"}.csv` })}>导出</Button>
        <Typography variant="body2" color="text.secondary">共 {totalCount} 条（有效 {totalCount - excludedCount}）</Typography>
        <Typography variant="body2" color="error.main">支出 ¥{totalExpense.toFixed(2)}</Typography>
        <Typography variant="body2" color="success.main">收入 ¥{totalIncome.toFixed(2)}</Typography>
      </Stack>

      {/* 批量操作 */}
      {selectedRows.length > 0 && (
        <Box className="batch-action-bar">
          <Typography variant="body2">已选中 <strong>{selectedRows.length}</strong> 条，合计 <strong>¥{selectedSum.toFixed(2)}</strong></Typography>
          <FormControl size="small" sx={{ minWidth: 130 }}><InputLabel>批量标签</InputLabel>
            <Select value={batchTagId} label="批量标签" onChange={(e) => setBatchTagId(e.target.value)}>
              {tags.map((t) => <MenuItem key={t.id} value={t.id}>{t.name}</MenuItem>)}
            </Select></FormControl>
          <Button size="small" variant="contained" onClick={handleBatchTag} disabled={!batchTagId}>应用标签</Button>
          <FormControl size="small" sx={{ minWidth: 100 }}><InputLabel>批量归属</InputLabel>
            <Select value={batchPayer} label="批量归属" onChange={(e) => setBatchPayer(e.target.value)}>
              {owners.map((o) => <MenuItem key={o.id} value={o.name}>{o.name}</MenuItem>)}
            </Select></FormControl>
          <Button size="small" variant="contained" onClick={handleBatchPayer} disabled={!batchPayer}>应用归属</Button>
          <Button size="small" variant="outlined" color="success" onClick={() => handleBatchRigid(true)}>标记刚需</Button>
          <Button size="small" variant="outlined" color="warning" onClick={() => handleBatchRigid(false)}>取消刚需</Button>
          <Button size="small" variant="outlined" color="error" onClick={handleBatchDelete}>删除</Button>
        </Box>
      )}

      {loading && <Box sx={{ display: "flex", justifyContent: "center", py: 4 }}><CircularProgress /></Box>}

      {!loading && (
        <Box className="ag-theme-alpine" sx={{ flexGrow: 1, minHeight: 0 }}
          onContextMenu={(e) => e.preventDefault()}>
          <AgGridReact<Transaction>
            ref={gridRef} rowData={displayed} columnDefs={columnDefs}
            rowSelection={rowSelection}
            getRowId={getRowId} getRowClass={getRowClass}
            defaultColDef={defaultColDef}
            localeText={ZH_LOCALE}
            onCellValueChanged={handleCellValueChanged}
            onSelectionChanged={onSelectionChanged}
            onCellContextMenu={handleContextMenu}
            onColumnMoved={saveColumnState} onColumnResized={saveColumnState} onColumnPinned={saveColumnState} onSortChanged={saveColumnState}
            onFirstDataRendered={handleFirstDataRendered}
            onRowDataUpdated={handleRowDataUpdated}
            suppressContextMenu={true}
            suppressRowClickSelection
            enableCellTextSelection enableRangeSelection copyHeadersToClipboard
            enableCellChangeFlash animateRows={false} domLayout="normal" rowBuffer={50}
            statusBar={{
              statusPanels: [
                { statusPanel: "agTotalAndFilteredRowCountComponent" },
                { statusPanel: "agSelectedRowCountComponent" },
              ],
            }}
          />
        </Box>
      )}

      {/* 右键菜单 */}
      {contextMenu && (
        <Box className="context-menu" sx={{ left: contextMenu.x, top: contextMenu.y, position: "fixed" }} onClick={closeContextMenu}>
          <Box className="context-menu-item" onClick={async () => { const r = contextMenu.rowData; await updateTransactionRigid(r.id, r.is_rigid !== 1); setTransactions(p => p.map(t => t.id === r.id ? {...t, is_rigid: t.is_rigid === 1 ? 0 : 1} : t)); setSnackMsg(r.is_rigid === 1 ? "已取消刚需" : "已标记刚需"); closeContextMenu(); }}>
            {contextMenu.rowData.is_rigid === 1 ? "取消刚需" : "标记刚需"}
          </Box>
          <Box className="context-menu-item" onClick={async () => { const r = contextMenu.rowData; const v = r.is_excluded_from_summary === 1 ? 0 : 1; await updateTransactionField(r.id, "is_excluded_from_summary", String(v)); setTransactions(p => p.map(t => t.id === r.id ? {...t, is_excluded_from_summary: v} : t)); setSnackMsg(v === 1 ? "已标记排除" : "已取消排除"); closeContextMenu(); }}>
            {contextMenu.rowData.is_excluded_from_summary === 1 ? "取消排除" : "标记排除"}
          </Box>
          <Box className="context-menu-divider" />
          <Box className="context-menu-item" onClick={() => { openRuleDialog(contextMenu.rowData); closeContextMenu(); }}>添加规则</Box>
          <Box className="context-menu-item" onClick={() => { navigator.clipboard.writeText(contextMenu.cellValue); closeContextMenu(); }}>复制单元格</Box>
        </Box>
      )}

      {/* 批量新增弹窗 */}
      <Dialog open={addDialog} onClose={() => setAddDialog(false)} maxWidth="md" fullWidth>
        <DialogTitle>批量新增交易</DialogTitle>
        <DialogContent>
          <Stack spacing={1} sx={{ mt: 1 }}>
            {newTxs.map((tx, idx) => (
              <Stack key={idx} direction="row" spacing={0.5} alignItems="center" flexWrap="wrap" useFlexGap
                sx={{ p: 1, bgcolor: idx % 2 === 0 ? "grey.50" : "transparent", borderRadius: 1 }}>
                <TextField size="small" label="时间" value={tx.transaction_time}
                  onChange={(e) => { const n = [...newTxs]; n[idx] = { ...n[idx], transaction_time: e.target.value }; setNewTxs(n); }}
                  placeholder="07-01 12:30" sx={{ width: 140 }} />
                <TextField size="small" label="金额" type="number" value={tx.amount}
                  onChange={(e) => { const n = [...newTxs]; n[idx] = { ...n[idx], amount: e.target.value }; setNewTxs(n); }}
                  inputProps={{ step: "0.01" }} sx={{ width: 90 }} />
                <FormControl size="small" sx={{ width: 80 }}>
                  <Select value={tx.direction} onChange={(e) => { const n = [...newTxs]; n[idx] = { ...n[idx], direction: e.target.value }; setNewTxs(n); }}>
                    <MenuItem value="expense">支出</MenuItem><MenuItem value="income">收入</MenuItem>
                  </Select></FormControl>
                <TextField size="small" label="对方" value={tx.counterparty}
                  onChange={(e) => { const n = [...newTxs]; n[idx] = { ...n[idx], counterparty: e.target.value }; setNewTxs(n); }}
                  sx={{ width: 120 }} />
                <TextField size="small" label="商品" value={tx.product}
                  onChange={(e) => { const n = [...newTxs]; n[idx] = { ...n[idx], product: e.target.value }; setNewTxs(n); }}
                  sx={{ flex: 1, minWidth: 120 }} />
                <FormControl size="small" sx={{ width: 110 }}>
                  <Select value={tx.tag_id} displayEmpty onChange={(e) => { const n = [...newTxs]; n[idx] = { ...n[idx], tag_id: e.target.value }; setNewTxs(n); }}>
                    <MenuItem value=""><em>标签</em></MenuItem>
                    {tags.map((t) => <MenuItem key={t.id} value={t.id}>{t.name}</MenuItem>)}
                  </Select></FormControl>
                <FormControl size="small" sx={{ width: 90 }}>
                  <Select value={tx.payer} displayEmpty onChange={(e) => { const n = [...newTxs]; n[idx] = { ...n[idx], payer: e.target.value }; setNewTxs(n); }}>
                    <MenuItem value=""><em>归属</em></MenuItem>
                    {owners.map((o) => <MenuItem key={o.id} value={o.name}>{o.name}</MenuItem>)}
                  </Select></FormControl>
                <IconButton size="small" color="error"
                  onClick={() => { const n = [...newTxs]; n.splice(idx, 1); setNewTxs(n.length ? n : [{ transaction_time: "", amount: "", counterparty: "", product: "", direction: "expense", tag_id: "", payer: "", payment_method: "" }]); }}
                  disabled={newTxs.length <= 1}>
                  <CloseIcon fontSize="small" />
                </IconButton>
              </Stack>
            ))}
          </Stack>
          <Button size="small" startIcon={<AddIcon />} sx={{ mt: 1 }}
            onClick={() => setNewTxs([...newTxs, { transaction_time: "", amount: "", counterparty: "", product: "", direction: "expense", tag_id: "", payer: "", payment_method: "" }])}>
            添加一行
          </Button>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setAddDialog(false)}>取消</Button>
          <Button variant="contained" onClick={handleBatchAdd}>批量保存</Button>
        </DialogActions>
      </Dialog>

      {/* 添加规则弹窗 */}
      <Dialog open={ruleDialog} onClose={() => setRuleDialog(false)} maxWidth="xs" fullWidth>
        <DialogTitle>添加规则</DialogTitle>
        <DialogContent>
          <Stack spacing={1.5} sx={{ mt: 1 }}>
            <FormControl fullWidth size="small">
              <InputLabel>匹配字段</InputLabel>
              <Select value={ruleField} label="匹配字段" onChange={(e) => {
                const f = e.target.value; setRuleField(f);
                const row = contextMenu?.rowData;
                if (row) setRuleValue(f === "counterparty" ? (row.counterparty || "") : f === "product" ? (row.product || "") : (row.transaction_type || ""));
              }}>
                <MenuItem value="counterparty">交易对方</MenuItem>
                <MenuItem value="product">商品</MenuItem>
                <MenuItem value="transaction_type">交易类型</MenuItem>
              </Select>
            </FormControl>
            <TextField fullWidth size="small" label="匹配值" value={ruleValue} onChange={(e) => setRuleValue(e.target.value)} />
            <FormControl fullWidth size="small">
              <InputLabel>匹配方式</InputLabel>
              <Select value={ruleType} label="匹配方式" onChange={(e) => setRuleType(e.target.value)}>
                <MenuItem value="exact">精确匹配</MenuItem>
                <MenuItem value="like">包含匹配（模糊）</MenuItem>
              </Select>
            </FormControl>
            <FormControl fullWidth size="small">
              <InputLabel>目标标签</InputLabel>
              <Select value={ruleTagId} label="目标标签" onChange={(e) => setRuleTagId(e.target.value)}>
                {tags.map((t) => <MenuItem key={t.id} value={t.id}>{t.name}</MenuItem>)}
              </Select>
            </FormControl>
          </Stack>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setRuleDialog(false)}>取消</Button>
          <Button variant="contained" onClick={handleAddRule} disabled={!ruleTagId || !ruleValue}>添加</Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
}

export default TransactionsPage;
