import { useState, useEffect } from "react";
import {
  Box,
  Typography,
  TextField,
  Button,
  Card,
  CardContent,
  Alert,
  CircularProgress,
  Stack,
  Switch,
  FormControlLabel,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableRow,
  Paper,
} from "@mui/material";
import SaveIcon from "@mui/icons-material/Save";
import CloudQueueIcon from "@mui/icons-material/CloudQueue";
import { getAllSettings, saveSetting } from "../api/settings";
import { testAiConnection } from "../api/ai";
import { getAppInfo, type AppInfo } from "../api/client";

function SettingsPage() {
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [testing, setTesting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null);

  const [aiEnabled, setAiEnabled] = useState(false);
  const [apiKey, setApiKey] = useState("");
  const [baseUrl, setBaseUrl] = useState("https://api.deepseek.com");
  const [model, setModel] = useState("deepseek-chat");

  useEffect(() => {
    Promise.all([getAllSettings(), getAppInfo()])
      .then(([settings, info]) => {
        setAiEnabled(settings.ai_enabled === "true");
        setApiKey(settings.ai_api_key || "");
        setBaseUrl(settings.ai_base_url || "https://api.deepseek.com");
        setModel(settings.ai_model || "deepseek-chat");
        setAppInfo(info);
      })
      .catch((e) => setError(String(e)))
      .finally(() => setLoading(false));
  }, []);

  const handleSave = async () => {
    setSaving(true);
    setError(null);
    setSuccess(null);
    try {
      await saveSetting("ai_enabled", aiEnabled ? "true" : "false");
      await saveSetting("ai_api_key", apiKey);
      await saveSetting("ai_base_url", baseUrl);
      await saveSetting("ai_model", model);
      setSuccess("设置已保存");
    } catch (e) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  };

  const handleTest = async () => {
    setTesting(true);
    setError(null);
    setSuccess(null);
    try {
      const ok = await testAiConnection();
      setSuccess(ok ? "连接成功！DeepSeek API 可用" : "连接失败：API 返回异常");
    } catch (e) {
      setError("连接失败: " + String(e));
    } finally {
      setTesting(false);
    }
  };

  if (loading) {
    return (
      <Box sx={{ display: "flex", justifyContent: "center", py: 4 }}>
        <CircularProgress />
      </Box>
    );
  }

  return (
    <Box>
      <Typography variant="h4" gutterBottom>设置</Typography>

      {error && <Alert severity="error" sx={{ mb: 2 }} onClose={() => setError(null)}>{error}</Alert>}
      {success && <Alert severity="success" sx={{ mb: 2 }} onClose={() => setSuccess(null)}>{success}</Alert>}

      {/* AI 配置 */}
      <Card variant="outlined" sx={{ mb: 3 }}>
        <CardContent>
          <Typography variant="h6" gutterBottom>AI 配置（DeepSeek）</Typography>
          <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
            配置后可在人工复核页面使用 AI 辅助分类，在报表中心使用 AI 分析和报表生成。
            API Key 仅存储在本地数据库中。
          </Typography>
          <Stack spacing={2}>
            <FormControlLabel
              control={<Switch checked={aiEnabled} onChange={(e) => setAiEnabled(e.target.checked)} />}
              label="启用 AI 功能"
            />
            <TextField
              label="API Key"
              type="password"
              fullWidth
              size="small"
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
              placeholder="sk-..."
              helperText="从 platform.deepseek.com 获取 API Key"
              disabled={!aiEnabled}
            />
            <TextField
              label="API 地址"
              fullWidth
              size="small"
              value={baseUrl}
              onChange={(e) => setBaseUrl(e.target.value)}
              disabled={!aiEnabled}
            />
            <TextField
              label="模型名称"
              fullWidth
              size="small"
              value={model}
              onChange={(e) => setModel(e.target.value)}
              disabled={!aiEnabled}
            />
            <Stack direction="row" spacing={2}>
              <Button
                variant="contained"
                startIcon={saving ? <CircularProgress size={16} color="inherit" /> : <SaveIcon />}
                onClick={handleSave}
                disabled={saving}
              >保存设置</Button>
              <Button
                variant="outlined"
                startIcon={testing ? <CircularProgress size={16} /> : <CloudQueueIcon />}
                onClick={handleTest}
                disabled={testing || !aiEnabled || !apiKey}
              >测试连接</Button>
            </Stack>
          </Stack>
        </CardContent>
      </Card>

      {/* 应用信息 */}
      {appInfo && (
        <Card variant="outlined">
          <CardContent>
            <Typography variant="h6" gutterBottom>应用信息</Typography>
            <TableContainer component={Paper} variant="outlined">
              <Table size="small">
                <TableBody>
                  <TableRow>
                    <TableCell component="th" scope="row" sx={{ fontWeight: "bold", width: "30%" }}>应用名称</TableCell>
                    <TableCell>{appInfo.app_name}</TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell component="th" scope="row" sx={{ fontWeight: "bold" }}>版本</TableCell>
                    <TableCell>{appInfo.app_version}</TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell component="th" scope="row" sx={{ fontWeight: "bold" }}>数据库路径</TableCell>
                    <TableCell sx={{ wordBreak: "break-all", fontFamily: "monospace", fontSize: "0.8rem" }}>{appInfo.db_path}</TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell component="th" scope="row" sx={{ fontWeight: "bold" }}>数据表数量</TableCell>
                    <TableCell>{appInfo.table_count}</TableCell>
                  </TableRow>
                </TableBody>
              </Table>
            </TableContainer>
          </CardContent>
        </Card>
      )}
    </Box>
  );
}

export default SettingsPage;
