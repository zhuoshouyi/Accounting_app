import { useState, useEffect } from "react";
import {
  Box, TextField, Button, Typography, Card, CardContent,
  Alert, CircularProgress, Select, MenuItem, FormControl, InputLabel,
  Stack, IconButton, Chip, Divider, Tooltip,
} from "@mui/material";
import SaveIcon from "@mui/icons-material/Save";
import AddIcon from "@mui/icons-material/Add";
import CloseIcon from "@mui/icons-material/Close";
import { saveManualData, getManualData } from "../../api/transaction";
import { getDistinctMonths } from "../../api/cleaning";

interface DetailItem { name: string; amount: string; done: boolean }
interface FieldDetails { total: string; items: DetailItem[] }
interface ManualEntryFormProps { month: string }

const FIELDS = [
  { key: "total_assets", label: "总资产", hint: "中行+莞行+微信+支付宝+招行+基金" },
  { key: "joey_income", label: "Joey收入", hint: "工资+奖金+其他" },
  { key: "vila_income", label: "Vila收入", hint: "工资+奖金+其他" },
  { key: "mortgage_savings", label: "房贷/存钱", hint: "月供+额外还款" },
  { key: "investment", label: "理财", hint: "基金+股票+定期" },
  { key: "insurance", label: "保险", hint: "年缴保险均摊" },
];

function ManualEntryForm({ month: initialMonth }: ManualEntryFormProps) {
  const [months, setMonths] = useState<string[]>([]);
  const [month, setMonth] = useState(initialMonth);
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);
  const [analysisText, setAnalysisText] = useState("");
  const [details, setDetails] = useState<Record<string, FieldDetails>>({});

  useEffect(() => { getDistinctMonths().then(setMonths).catch(() => {}); }, []);
  useEffect(() => { setMonth(initialMonth); }, [initialMonth]);

  useEffect(() => {
    if (!month) return;
    setLoading(true); setError(null);
    getManualData(month).then((data) => {
      const d: Record<string, FieldDetails> = {};
      for (const f of FIELDS) {
        const total = data ? (data as any)[f.key] : null;
        d[f.key] = { total: total != null ? String(total) : "", items: [] };
      }
      if (data?.details_json) {
        try { const p = JSON.parse(data.details_json); for (const f of FIELDS) { if (p[f.key]) d[f.key].items = p[f.key].map((it: any) => ({ ...it, done: true })); } } catch {}
      }
      setDetails(d);
      setAnalysisText(data?.analysis_text ?? "");
    }).catch((e) => setError(String(e))).finally(() => setLoading(false));
  }, [month]);

  const updateTotal = (key: string, v: string) => setDetails((p) => ({ ...p, [key]: { ...p[key], total: v } }));
  const addItem = (key: string) => setDetails((p) => ({ ...p, [key]: { ...p[key], items: [...(p[key]?.items||[]), { name: "", amount: "", done: false }] } }));
  const removeItem = (key: string, i: number) => setDetails((p) => {
    const items = [...(p[key]?.items||[])]; items.splice(i, 1); return { ...p, [key]: { ...p[key], items } };
  });
  const updateItem = (key: string, i: number, field: "name"|"amount", v: string) => setDetails((p) => {
    const items = [...(p[key]?.items||[])]; items[i] = { ...items[i], [field]: v }; return { ...p, [key]: { ...p[key], items } };
  });
  const confirmItem = (key: string, i: number) => setDetails((p) => {
    const items = [...(p[key]?.items||[])];
    if (!items[i]?.name && !items[i]?.amount) { items.splice(i, 1); } // 空的删除
    else { items[i] = { ...items[i], done: true }; } // 标记完成
    return { ...p, [key]: { ...p[key], items } };
  });

  const handleSave = async () => {
    setSaving(true); setError(null); setSuccess(false);
    try {
      const detailsJson = JSON.stringify(Object.fromEntries(FIELDS.map((f) => [f.key, details[f.key]?.items||[]])));
      const dataObj: any = { month, analysis_text: analysisText||null, details_json: detailsJson };
      for (const f of FIELDS) { const t = details[f.key]?.total; dataObj[f.key] = t ? parseFloat(t) : null; }
      await saveManualData(dataObj);
      setSuccess(true); setTimeout(() => setSuccess(false), 3000);
    } catch (e) { setError(String(e)); } finally { setSaving(false); }
  };

  if (loading) return <Box sx={{ display: "flex", justifyContent: "center", py: 2 }}><CircularProgress size={24} /></Box>;

  return (
    <Card variant="outlined">
      <CardContent>
        <Stack direction="row" spacing={2} alignItems="center" sx={{ mb: 2 }}>
          <Typography variant="h6">月度手动数据</Typography>
          <FormControl size="small" sx={{ minWidth: 140 }}>
            <InputLabel>月份</InputLabel>
            <Select value={month} label="月份" onChange={(e) => setMonth(e.target.value)}>
              {months.map((m) => <MenuItem key={m} value={m}>{m}</MenuItem>)}
            </Select>
          </FormControl>
          <Box sx={{ flex: 1 }} />
          <Button variant="contained" size="small" startIcon={saving ? <CircularProgress size={14} color="inherit" /> : <SaveIcon />}
            onClick={handleSave} disabled={saving}>保存</Button>
        </Stack>

        {error && <Alert severity="error" sx={{ mb: 2 }} onClose={() => setError(null)}>{error}</Alert>}
        {success && <Alert severity="success" sx={{ mb: 2 }}>保存成功</Alert>}

        <Stack spacing={1}>
          {FIELDS.map((field) => {
            const fd = details[field.key] || { total: "", items: [] };
            const itemSum = fd.items.reduce((s, it) => s + (parseFloat(it.amount) || 0), 0);
            return (
              <Box key={field.key}>
                <Stack direction="row" alignItems="center" spacing={1} sx={{ py: 0.5 }}>
                  <Typography variant="body2" sx={{ width: 90, flexShrink: 0, color: "text.secondary", textAlign: "right" }}>
                    {field.label}
                  </Typography>
                  <Tooltip title={fd.items.filter(it=>it.done).map(it=>`${it.name} ¥${it.amount}`).join(" + ") || field.hint} arrow>
                    <TextField size="small" type="number" value={fd.total}
                      sx={{ width: 130, flexShrink: 0 }}
                      onChange={(e) => updateTotal(field.key, e.target.value)}
                      inputProps={{ step: "0.01" }}
                      placeholder="0"
                    />
                  </Tooltip>
                  <Typography variant="caption" color="text.secondary" sx={{ flexShrink: 0 }}>元</Typography>
                  {fd.items.map((item, idx) => {
                    return item.done ? (
                      <Chip key={idx} size="small" variant="outlined"
                        label={`${item.name} ¥${item.amount}`}
                        onDelete={() => removeItem(field.key, idx)}
                        deleteIcon={<CloseIcon fontSize="small" />}
                        sx={{ maxWidth: 200 }}
                      />
                    ) : (
                      <Stack key={idx} direction="row" spacing={0.5} alignItems="center">
                        <TextField size="small" placeholder="名称" value={item.name}
                          sx={{ width: 110 }} onChange={(e) => updateItem(field.key, idx, "name", e.target.value)}
                          onKeyDown={(e) => { if (e.key === "Enter") { e.preventDefault(); confirmItem(field.key, idx); } }} />
                        <TextField size="small" placeholder="金额" type="number" value={item.amount}
                          sx={{ width: 80 }} onChange={(e) => updateItem(field.key, idx, "amount", e.target.value)}
                          onKeyDown={(e) => { if (e.key === "Enter") { e.preventDefault(); confirmItem(field.key, idx); } }}
                          inputProps={{ step: "0.01" }} />
                        <Typography variant="caption">元</Typography>
                        <IconButton size="small" onClick={() => removeItem(field.key, idx)}><CloseIcon fontSize="small" /></IconButton>
                      </Stack>
                    );
                  })}
                  {itemSum > 0 && itemSum !== parseFloat(fd.total || "0") && (
                    <Typography variant="caption" color="warning.main">明细¥{itemSum.toFixed(2)}</Typography>
                  )}
                  <IconButton size="small" color="primary" onClick={() => addItem(field.key)}>
                    <AddIcon fontSize="small" />
                  </IconButton>
                </Stack>
              </Box>
            );
          })}
        </Stack>

        <Divider sx={{ my: 1.5 }} />
        <TextField label="月度分析" fullWidth size="small" multiline rows={2}
          value={analysisText} onChange={(e) => setAnalysisText(e.target.value)}
          placeholder="对当月的非日常支出进行分析..." />
      </CardContent>
    </Card>
  );
}

export default ManualEntryForm;
