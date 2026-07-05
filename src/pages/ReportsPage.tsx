import { useState, useEffect, useCallback } from "react";
import {
  Box,
  Typography,
  Alert,
  CircularProgress,
  Button,
  Card,
  CardContent,
  Stack,
  Select,
  MenuItem,
  FormControl,
  InputLabel,
  Chip,
  Divider,
  TextField,
} from "@mui/material";
import AutoModeIcon from "@mui/icons-material/AutoMode";
import AssessmentIcon from "@mui/icons-material/Assessment";
import HistoryIcon from "@mui/icons-material/History";
import FullscreenIcon from "@mui/icons-material/Fullscreen";
import CloseFullscreenIcon from "@mui/icons-material/FullscreenExit";
import { getDistinctMonths } from "../api/cleaning";
import { getMonthlySummary } from "../api/summary";
import { getReportHistory, getReportById, saveReport } from "../api/report";
import { aiAnalyze, aiGenerateReport, aiGenerateChart, type AiGenerateResult } from "../api/ai";
import ChartViewer from "../components/reports/ChartViewer";
import type { AiReport } from "../types";

function ReportsPage() {
  const [months, setMonths] = useState<string[]>([]);
  const [selectedMonth, setSelectedMonth] = useState("");
  const [error, setError] = useState<string | null>(null);

  const [analysis, setAnalysis] = useState<AiGenerateResult | null>(null);
  const [analyzing, setAnalyzing] = useState(false);

  const [htmlReport, setHtmlReport] = useState<string | null>(null);
  const [generating, setGenerating] = useState(false);

  const [history, setHistory] = useState<AiReport[]>([]);
  const [viewingReport, setViewingReport] = useState<AiReport | null>(null);
  const [fullscreenReport, setFullscreenReport] = useState<string | null>(null);

  // AI 图表
  const [chartPrompt, setChartPrompt] = useState("");
  const [chartConfig, setChartConfig] = useState<any>(null);
  const [chartLoading, setChartLoading] = useState(false);

  const handleGenerateChart = async () => {
    if (!chartPrompt) return;
    setChartLoading(true); setError(null); setChartConfig(null);
    try {
      const json = await aiGenerateChart(chartPrompt);
      const cfg = JSON.parse(json);
      if (!cfg.data || !cfg.type) { setError("AI 返回的数据不完整，请重试"); return; }
      setChartConfig(cfg);
    } catch (e) { setError("图表生成失败: " + String(e)); }
    finally { setChartLoading(false); }
  };


  useEffect(() => {
    getDistinctMonths()
      .then((m) => { setMonths(m); if (m.length > 0) setSelectedMonth(m[0]); })
      .catch((e) => setError(String(e)));
  }, []);

  const loadHistory = useCallback(async () => {
    try { setHistory(await getReportHistory()); } catch (e) { setError(String(e)); }
  }, []);

  useEffect(() => { loadHistory(); }, [loadHistory]);

  const handleAnalyze = async () => {
    if (!selectedMonth) return;
    setAnalyzing(true); setError(null);
    try {
      const result = await aiAnalyze(selectedMonth);
      setAnalysis(result);
      await saveReport({
        month: selectedMonth, report_type: "analysis",
        title: `${selectedMonth} AI 月度分析`, content: result.content,
        summary_json: "{}", model_name: result.model_name,
      });
      loadHistory();
    } catch (e) { setError(String(e)); }
    finally { setAnalyzing(false); }
  };

  const handleGenerateReport = async () => {
    if (!selectedMonth) return;
    setGenerating(true); setError(null);
    try {
      const summary = await getMonthlySummary(selectedMonth);
      const result = await aiGenerateReport(selectedMonth);
      setHtmlReport(result.content);
      await saveReport({
        month: selectedMonth, report_type: "html_report",
        title: `${selectedMonth} 家庭收支报表`, content: result.content,
        summary_json: JSON.stringify(summary), model_name: result.model_name,
      });
      loadHistory();
    } catch (e) { setError(String(e)); }
    finally { setGenerating(false); }
  };

  const handleViewReport = async (id: string) => {
    try { setViewingReport(await getReportById(id)); }
    catch (e) { setError(String(e)); }
  };

  return (
    <Box>
      <Typography variant="h4" gutterBottom>报表中心</Typography>

      {error && <Alert severity="error" sx={{ mb: 2 }} onClose={() => setError(null)}>{error}</Alert>}

      <Stack direction="row" spacing={2} alignItems="center" sx={{ mb: 3 }} flexWrap="wrap">
        <FormControl sx={{ minWidth: 180 }} size="small">
          <InputLabel>月份</InputLabel>
          <Select value={selectedMonth} label="月份" onChange={(e) => setSelectedMonth(e.target.value)}>
            {months.map((m) => <MenuItem key={m} value={m}>{m}</MenuItem>)}
          </Select>
        </FormControl>
        <Button variant="contained" color="primary"
          startIcon={analyzing ? <CircularProgress size={16} color="inherit" /> : <AutoModeIcon />}
          onClick={handleAnalyze} disabled={analyzing || !selectedMonth}>
          AI 分析
        </Button>
        <Button variant="contained" color="secondary"
          startIcon={generating ? <CircularProgress size={16} color="inherit" /> : <AssessmentIcon />}
          onClick={handleGenerateReport} disabled={generating || !selectedMonth}>
          生成 HTML 报表
        </Button>
      </Stack>

      {/* AI 分析结果 */}
      {analysis && (
        <Card variant="outlined" sx={{ mb: 3 }}>
          <CardContent>
            <Stack direction="row" spacing={1} alignItems="center" sx={{ mb: 1 }}>
              <Typography variant="h6">{selectedMonth} AI 月度分析</Typography>
              <Chip label={`模型: ${analysis.model_name}`} size="small" variant="outlined" />
            </Stack>
            <Typography variant="body1" sx={{ whiteSpace: "pre-wrap", lineHeight: 1.8 }}>
              {analysis.content}
            </Typography>
          </CardContent>
        </Card>
      )}

      {/* AI HTML 报表 */}
      {htmlReport && (
        <Card variant="outlined" sx={{ mb: 3 }}>
          <CardContent>
            <Stack direction="row" alignItems="center" justifyContent="space-between" sx={{ mb: 1 }}>
              <Typography variant="h6">{selectedMonth} HTML 报表</Typography>
              <Button size="small" startIcon={<FullscreenIcon />}
                onClick={() => setFullscreenReport(htmlReport)}>全屏</Button>
            </Stack>
            <Box sx={{ border: "1px solid #e0e0e0", borderRadius: 1, height: 500, overflow: "auto" }}>
              <iframe srcDoc={htmlReport} title="AI 报表" sandbox="allow-same-origin"
                style={{ width: "100%", height: "100%", border: "none" }} />
            </Box>
          </CardContent>
        </Card>
      )}

      <Divider sx={{ my: 2 }} />

      {/* AI 图表生成 */}
      <Stack direction="row" spacing={1} alignItems="center" sx={{ mb: 2 }}>
        <TextField size="small" label="描述你想要的图表" fullWidth
          value={chartPrompt} onChange={(e) => setChartPrompt(e.target.value)}
          placeholder='例如：各月餐饮和买菜支出对比柱状图'
          onKeyDown={(e) => { if (e.key === "Enter") handleGenerateChart(); }}
        />
        <Button variant="contained"
          startIcon={chartLoading ? <CircularProgress size={16} color="inherit" /> : <AutoModeIcon />}
          onClick={handleGenerateChart} disabled={chartLoading || !chartPrompt}>
          生成
        </Button>
      </Stack>
      {chartConfig && (
        <Card variant="outlined" sx={{ mb: 3 }}>
          <CardContent>
            <Typography variant="h6" gutterBottom>{chartConfig.title || "图表"}</Typography>
            <ChartViewer config={chartConfig} />
          </CardContent>
        </Card>
      )}

      {/* 报表历史 */}
      <Stack direction="row" spacing={1} alignItems="center" sx={{ mb: 2 }}>
        <HistoryIcon color="action" />
        <Typography variant="h6">报表历史</Typography>
      </Stack>

      {history.length === 0 && <Typography color="text.secondary">暂无历史报表</Typography>}
      <Stack spacing={1}>
        {history.map((r) => (
          <Card key={r.id} variant="outlined"
            sx={{ cursor: "pointer", "&:hover": { bgcolor: "action.hover" } }}
            onClick={() => handleViewReport(r.id)}>
            <CardContent sx={{ py: 1.5 }}>
              <Stack direction="row" spacing={1} alignItems="center">
                <Chip label={r.report_type === "html_report" ? "报表" : "分析"} size="small"
                  color={r.report_type === "html_report" ? "secondary" : "primary"} />
                <Typography variant="body2" fontWeight="medium">{r.title || `${r.month} 报表`}</Typography>
                <Typography variant="caption" color="text.secondary">{r.created_at}</Typography>
              </Stack>
            </CardContent>
          </Card>
        ))}
      </Stack>

      {/* 查看历史报表详情 */}
      {viewingReport && (
        <Card variant="outlined" sx={{ mt: 2 }}>
          <CardContent>
            <Stack direction="row" justifyContent="space-between" alignItems="center" sx={{ mb: 1 }}>
              <Typography variant="subtitle1">{viewingReport.title || viewingReport.month}</Typography>
              <Stack direction="row" spacing={1}>
                {viewingReport.report_type === "html_report" && (
                  <Button size="small" startIcon={<FullscreenIcon />}
                    onClick={() => setFullscreenReport(viewingReport.content)}>全屏</Button>
                )}
                <Button size="small" onClick={() => setViewingReport(null)}>关闭</Button>
              </Stack>
            </Stack>
            {viewingReport.report_type === "html_report" && viewingReport.content ? (
              <Box sx={{ border: "1px solid #e0e0e0", borderRadius: 1, height: 400, overflow: "auto" }}>
                <iframe srcDoc={viewingReport.content} title="历史报表" sandbox="allow-same-origin"
                  style={{ width: "100%", height: "100%", border: "none" }} />
              </Box>
            ) : (
              <Typography variant="body2" sx={{ whiteSpace: "pre-wrap" }}>{viewingReport.content}</Typography>
            )}
          </CardContent>
        </Card>
      )}
      {/* 全屏报表 */}
      {fullscreenReport && (
        <Box sx={{ position: "fixed", top: 0, left: 0, width: "100vw", height: "100vh", zIndex: 9999, bgcolor: "#fff" }}>
          <Box sx={{ position: "absolute", top: 8, right: 8, zIndex: 1 }}>
            <Button variant="contained" size="small" startIcon={<CloseFullscreenIcon />}
              onClick={() => setFullscreenReport(null)}>退出全屏</Button>
          </Box>
          <iframe srcDoc={fullscreenReport} title="全屏报表"
            style={{ width: "100%", height: "100%", border: "none" }} />
        </Box>
      )}
    </Box>
  );
}

export default ReportsPage;
