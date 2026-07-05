import { useState, useEffect, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import {
  Box, Typography, Alert, CircularProgress, Table, TableBody, TableCell,
  TableHead, TableRow, TableContainer, Paper, Stack, Select, MenuItem,
  FormControl, InputLabel, Button, Snackbar, Tooltip,
} from "@mui/material";
import { getAllMonthsSummary, type MonthlySummaryRow, type SummaryCategory } from "../api/summary";
import { buildDrillDownUrl } from "../utils/navigation";
import ManualEntryForm from "../components/transactions/ManualEntryForm";

/** 从 category.tags 构建 tooltip 文本 */
const catTooltip = (cat: SummaryCategory) => cat.tags.map((t) => `${t.tag_name}(¥${t.amount.toFixed(0)})`).join(" + ") || cat.summary_category;

/** 从 details_json 构建手动数据字段的 tooltip */
const detailTooltip = (row: MonthlySummaryRow, fieldKey: string, defaultHint: string) => {
  if (!row.details_json) return defaultHint;
  try {
    const d = JSON.parse(row.details_json);
    const items = d[fieldKey] || [];
    if (!items.length) return defaultHint;
    return items.map((it: any) => `${it.name} ¥${Number(it.amount).toFixed(2)}`).join(" + ");
  } catch { return defaultHint; }
};

function SummaryPage() {
  const navigate = useNavigate();
  const [rows, setRows] = useState<MonthlySummaryRow[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [snack, setSnack] = useState<string | null>(null);
  const [selectedMonth, setSelectedMonth] = useState("");

  // 汇总类列表（取自第一条数据）
  const catNames = rows.length > 0 ? rows[0].categories.map((c) => c.summary_category) : [];

  const loadData = useCallback(async () => {
    setLoading(true); setError(null);
    try { setRows(await getAllMonthsSummary()); } catch (e) { setError(String(e)); }
    finally { setLoading(false); }
  }, []);
  useEffect(() => { loadData(); }, [loadData]);

  const handleDrillDown = (month: string, cat: SummaryCategory) => {
    const tagIds = cat.tags.map((t) => t.tag_id);
    navigate(buildDrillDownUrl(month, tagIds));
  };

  const fmt = (v: number | null | undefined) => v != null ? `¥${v.toFixed(2)}` : "-";

  return (
    <Box>
      <Typography variant="h4" gutterBottom>月度汇总</Typography>
      {error && <Alert severity="error" sx={{ mb: 2 }} onClose={() => setError(null)}>{error}</Alert>}
      <Snackbar open={!!snack} autoHideDuration={2000} onClose={() => setSnack(null)} message={snack} anchorOrigin={{ vertical: "top", horizontal: "center" }} />

      {loading && <Box sx={{ display: "flex", justifyContent: "center", py: 4 }}><CircularProgress /></Box>}

      {!loading && rows.length > 0 && (
        <TableContainer component={Paper} variant="outlined" sx={{ mb: 3, overflow: "auto" }}>
          <Table size="small" sx={{ minWidth: 1200 }}>
            <TableHead>
              <TableRow>
                <TableCell sx={{ position: "sticky", left: 0, bgcolor: "white", zIndex: 2, fontWeight: 600 }}>月份</TableCell>
                <TableCell align="right">总资产</TableCell>
                <TableCell align="right">Joey收入</TableCell>
                <TableCell align="right">Vila收入</TableCell>
                <TableCell align="right" sx={{ fontWeight: 600, color: "error.main" }}>总支出</TableCell>
                <TableCell align="right">房贷/存钱</TableCell>
                {catNames.map((name) => {
                  const firstCat = rows[0]?.categories.find((c) => c.summary_category === name);
                  return (
                    <Tooltip key={name} title={firstCat ? catTooltip(firstCat) : name} arrow>
                      <TableCell align="right" sx={{ whiteSpace: "nowrap", fontWeight: 600, cursor: "help" }}>{name}</TableCell>
                    </Tooltip>
                  );
                })}
                <TableCell align="right">理财</TableCell>
                <TableCell align="right">保险</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {rows.map((row) => (
                <TableRow key={row.month} hover>
                  <TableCell sx={{ position: "sticky", left: 0, bgcolor: "white", fontWeight: 500 }}>{row.month}</TableCell>
                  <Tooltip title={detailTooltip(row, "total_assets", "中行+莞行+微信+支付宝+招行+基金")} arrow>
                    <TableCell align="right">{row.total_assets != null ? `¥${row.total_assets.toFixed(2)}` : "-"}</TableCell>
                  </Tooltip>
                  <Tooltip title={detailTooltip(row, "joey_income", "工资+奖金+其他")} arrow>
                    <TableCell align="right">{fmt(row.joey_income)}</TableCell>
                  </Tooltip>
                  <Tooltip title={detailTooltip(row, "vila_income", "工资+奖金+其他")} arrow>
                    <TableCell align="right">{fmt(row.vila_income)}</TableCell>
                  </Tooltip>
                  <TableCell align="right" sx={{ fontWeight: 600, color: "error.main" }}>{fmt(row.total_expense)}</TableCell>
                  <Tooltip title={detailTooltip(row, "mortgage_savings", "月供+额外还款")} arrow>
                    <TableCell align="right">{fmt(row.mortgage_savings)}</TableCell>
                  </Tooltip>
                  {catNames.map((name) => {
                    const cat = row.categories.find((c) => c.summary_category === name);
                    const amt = cat?.total_amount ?? 0;
                    return (
                      <Tooltip key={name} title={cat ? catTooltip(cat) : ""} arrow>
                        <TableCell align="right" sx={{ cursor: amt > 0 ? "pointer" : "default", color: amt > 0 ? "primary.main" : "text.secondary" }}
                          onClick={() => cat && cat.tags.length > 0 && handleDrillDown(row.month, cat)}>
                          {amt > 0 ? `¥${amt.toFixed(0)}` : "-"}
                        </TableCell>
                      </Tooltip>
                    );
                  })}
                  <Tooltip title={detailTooltip(row, "investment", "基金+股票+定期")} arrow>
                    <TableCell align="right">{fmt(row.investment)}</TableCell>
                  </Tooltip>
                  <Tooltip title={detailTooltip(row, "insurance", "年缴保险均摊")} arrow>
                    <TableCell align="right">{fmt(row.insurance)}</TableCell>
                  </Tooltip>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      )}

      {!loading && rows.length === 0 && <Typography color="text.secondary">暂无数据</Typography>}

      {/* 手动录入 — 选中有数据的最近月份 */}
      <ManualEntryForm month={rows.length > 0 ? rows[0].month : ""} />
    </Box>
  );
}

export default SummaryPage;
