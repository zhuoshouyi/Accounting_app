import { useState, useEffect, useCallback } from "react";
import {
  Box,
  Typography,
  Button,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Select,
  MenuItem,
  FormControl,
  InputLabel,
  Card,
  CardContent,
  Alert,
  CircularProgress,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableRow,
  TableContainer,
  Paper,
  Chip,
  Stack,
} from "@mui/material";
import UploadFileIcon from "@mui/icons-material/UploadFile";
import { open } from "@tauri-apps/plugin-dialog";
import {
  importBillFile,
  listImportRecords,
  checkDuplicateImport,
  type ImportResult,
} from "../api/import";
import { listAccountOwners } from "../api/account_owner";
import type { AccountOwner, ImportRecord } from "../types";

/**
 * 导入账单页面
 * 支持导入微信 XLSX 和支付宝 CSV(GBK) 账单
 */
function ImportPage() {
  const [owners, setOwners] = useState<AccountOwner[]>([]);
  const [importRecords, setImportRecords] = useState<ImportRecord[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<ImportResult | null>(null);

  // 归属人选择对话框
  const [ownerDialogOpen, setOwnerDialogOpen] = useState(false);
  const [pendingFilePath, setPendingFilePath] = useState<string | null>(null);
  const [pendingSource, setPendingSource] = useState<string>("");
  const [selectedOwner, setSelectedOwner] = useState<string>("");

  // 重复导入确认对话框
  const [duplicateDialogOpen, setDuplicateDialogOpen] = useState(false);

  /** 加载数据 */
  const loadData = useCallback(async () => {
    try {
      const [ownerData, recordData] = await Promise.all([
        listAccountOwners(),
        listImportRecords(),
      ]);
      setOwners(ownerData);
      setImportRecords(recordData);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  useEffect(() => {
    loadData();
  }, [loadData]);

  /** 选择文件并打开归属人选择对话框 */
  const handleSelectFile = async (source: string) => {
    setError(null);
    setSuccess(null);

    const filters =
      source === "wechat"
        ? [{ name: "Excel", extensions: ["xlsx"] }]
        : [{ name: "CSV", extensions: ["csv"] }];

    try {
      const result = await open({ filters, multiple: false });
      if (result === null) return;

      const filePath = typeof result === "string" ? result : result[0];
      if (!filePath) return;

      setPendingFilePath(filePath);
      setPendingSource(source);
      setSelectedOwner("");
      setOwnerDialogOpen(true);
    } catch (e) {
      setError(String(e));
    }
  };

  /** 归属人选择确认 */
  const handleOwnerConfirm = async () => {
    setOwnerDialogOpen(false);
    if (!pendingFilePath) return;

    const payer = selectedOwner || null;

    setLoading(true);
    setError(null);

    try {
      // 检查是否已导入过
      const isDuplicate = await checkDuplicateImport(pendingFilePath);

      if (isDuplicate) {
        setLoading(false);
        setDuplicateDialogOpen(true);
        return;
      }

      // 直接导入
      await doImport(pendingFilePath, pendingSource, payer);
    } catch (e) {
      setError(String(e));
      setLoading(false);
    }
  };

  /** 执行导入 */
  const doImport = async (
    filePath: string,
    source: string,
    payer: string | null
  ) => {
    setLoading(true);
    setError(null);
    try {
      const result = await importBillFile(filePath, source, payer);
      setSuccess(result);
      await loadData();
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  /** 确认重复导入 */
  const handleDuplicateConfirm = () => {
    setDuplicateDialogOpen(false);
    if (pendingFilePath) {
      doImport(pendingFilePath, pendingSource, selectedOwner || null);
    }
  };

  /** 来源显示名称 */
  const sourceLabel = (source: string) => {
    switch (source) {
      case "wechat":
        return "微信";
      case "alipay":
        return "支付宝";
      default:
        return source;
    }
  };

  /** 来源标签颜色 */
  const sourceColor = (source: string) => {
    switch (source) {
      case "wechat":
        return "success" as const;
      case "alipay":
        return "primary" as const;
      default:
        return "default" as const;
    }
  };

  return (
    <Box>
      <Typography variant="h4" gutterBottom>
        导入账单
      </Typography>

      {error && (
        <Alert severity="error" sx={{ mb: 2 }} onClose={() => setError(null)}>
          {error}
        </Alert>
      )}

      {/* 上传区域 */}
      <Stack direction="row" spacing={2} sx={{ mb: 3 }}>
        <Button
          variant="contained"
          size="large"
          startIcon={<UploadFileIcon />}
          onClick={() => handleSelectFile("wechat")}
          disabled={loading}
          color="success"
        >
          导入微信账单
        </Button>
        <Button
          variant="contained"
          size="large"
          startIcon={<UploadFileIcon />}
          onClick={() => handleSelectFile("alipay")}
          disabled={loading}
          color="primary"
        >
          导入支付宝账单
        </Button>
      </Stack>

      {loading && (
        <Box sx={{ display: "flex", justifyContent: "center", py: 4 }}>
          <CircularProgress />
        </Box>
      )}

      {/* 导入结果 */}
      {success && !loading && (
        <Card sx={{ mb: 3 }}>
          <CardContent>
            <Typography variant="h6" gutterBottom>
              导入成功
            </Typography>
            <Stack spacing={1}>
              <Box>
                <Chip
                  label={sourceLabel(success.source)}
                  color={sourceColor(success.source)}
                  size="small"
                  sx={{ mr: 1 }}
                />
                <Typography variant="body2" component="span">
                  文件: {success.file_name}
                </Typography>
              </Box>
              <Typography variant="body2">
                账户信息: {success.account_info}
              </Typography>
              <Typography variant="body2">
                总交易数: {success.total_count} | 导入成功:{" "}
                {success.imported_count} | 跳过: {success.skipped_count}
              </Typography>
              <Typography variant="body2">
                涉及月份: {success.months.join(", ")}
              </Typography>
              {success.payer && (
                <Typography variant="body2">
                  归属人: {success.payer}
                </Typography>
              )}
            </Stack>
          </CardContent>
        </Card>
      )}

      {/* 导入历史 */}
      <Typography variant="h6" gutterBottom>
        导入历史
      </Typography>
      {importRecords.length === 0 ? (
        <Typography color="text.secondary" sx={{ py: 2 }}>
          暂无导入记录
        </Typography>
      ) : (
        <TableContainer component={Paper} variant="outlined">
          <Table size="small">
            <TableHead>
              <TableRow>
                <TableCell>文件名</TableCell>
                <TableCell>来源</TableCell>
                <TableCell>归属人</TableCell>
                <TableCell align="right">条数</TableCell>
                <TableCell>导入时间</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {importRecords.map((record) => (
                <TableRow key={record.id}>
                  <TableCell>{record.file_name}</TableCell>
                  <TableCell>
                    <Chip
                      label={sourceLabel(record.source)}
                      color={sourceColor(record.source)}
                      size="small"
                    />
                  </TableCell>
                  <TableCell>{record.payer || "-"}</TableCell>
                  <TableCell align="right">
                    {record.valid_count ?? 0}
                  </TableCell>
                  <TableCell>{record.imported_at}</TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      )}

      {/* 归属人选择对话框 */}
      <Dialog
        open={ownerDialogOpen}
        onClose={() => setOwnerDialogOpen(false)}
        maxWidth="xs"
        fullWidth
      >
        <DialogTitle>选择归属人</DialogTitle>
        <DialogContent>
          <FormControl fullWidth sx={{ mt: 1 }}>
            <InputLabel>归属人</InputLabel>
            <Select
              value={selectedOwner}
              label="归属人"
              onChange={(e) => setSelectedOwner(e.target.value)}
            >
              <MenuItem value="">
                <em>不选择归属人</em>
              </MenuItem>
              {owners.map((owner) => (
                <MenuItem key={owner.id} value={owner.name}>
                  {owner.name}
                </MenuItem>
              ))}
            </Select>
          </FormControl>
          {owners.length === 0 && (
            <Typography variant="body2" color="text.secondary" sx={{ mt: 2 }}>
              暂无归属人，可选择"不选择归属人"跳过，或先到归属人管理页面添加。
            </Typography>
          )}
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setOwnerDialogOpen(false)}>跳过</Button>
          <Button
            onClick={handleOwnerConfirm}
            variant="contained"
          >
            确认
          </Button>
        </DialogActions>
      </Dialog>

      {/* 重复导入确认对话框 */}
      <Dialog
        open={duplicateDialogOpen}
        onClose={() => setDuplicateDialogOpen(false)}
        maxWidth="xs"
        fullWidth
      >
        <DialogTitle>文件已导入过</DialogTitle>
        <DialogContent>
          <Typography>
            该文件已导入过，是否重复导入？重复导入会创建重复的交易记录。
          </Typography>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setDuplicateDialogOpen(false)}>取消</Button>
          <Button
            onClick={handleDuplicateConfirm}
            color="warning"
            variant="contained"
          >
            重复导入
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
}

export default ImportPage;
