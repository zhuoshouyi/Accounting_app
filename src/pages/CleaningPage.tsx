import { useState } from "react";
import {
  Box,
  Typography,
  Button,
  Alert,
  CircularProgress,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableRow,
  TableContainer,
  Paper,
  Checkbox,
  Chip,
  Stack,
  Card,
  CardContent,
  Divider,
} from "@mui/material";
import CleaningServicesIcon from "@mui/icons-material/CleaningServices";
import CheckCircleIcon from "@mui/icons-material/CheckCircle";
import ArrowBackIcon from "@mui/icons-material/ArrowBack";
import {
  previewCleaning,
  executeCleaning,
  type CleaningPreviewResult,
  type CleaningExecuteResult,
} from "../api/cleaning";

/**
 * 数据清洗页面
 *
 * 流程：开始清洗 → 预览（待过滤 + 待修改）→ 确认执行 → 结果摘要
 */
function CleaningPage() {
  const [phase, setPhase] = useState<"idle" | "preview" | "done">("idle");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [preview, setPreview] = useState<CleaningPreviewResult | null>(null);
  const [executeResult, setExecuteResult] =
    useState<CleaningExecuteResult | null>(null);
  const [checkedExcludeIds, setCheckedExcludeIds] = useState<Set<string>>(
    new Set()
  );
  const [checkedModifyIds, setCheckedModifyIds] = useState<Set<string>>(
    new Set()
  );

  // ----------------------------------------------------------------
  // 辅助函数
  // ----------------------------------------------------------------

  const sourceLabel = (source: string): string => {
    switch (source) {
      case "wechat":
        return "微信";
      case "alipay":
        return "支付宝";
      default:
        return source;
    }
  };

  const sourceColor = (
    source: string
  ): "success" | "primary" | "default" => {
    switch (source) {
      case "wechat":
        return "success";
      case "alipay":
        return "primary";
      default:
        return "default";
    }
  };

  const directionLabel = (dir: string): string => {
    switch (dir) {
      case "expense":
        return "支出";
      case "income":
        return "收入";
      case "neutral":
        return "不计收支";
      default:
        return dir;
    }
  };

  const formatAmount = (amount: number | null | undefined): string => {
    if (amount === null || amount === undefined) return "-";
    return amount.toFixed(2);
  };

  // ----------------------------------------------------------------
  // 事件处理
  // ----------------------------------------------------------------

  /** 点击"开始清洗" — 获取预览数据 */
  const handlePreview = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await previewCleaning();
      setPreview(result);
      setCheckedExcludeIds(
        new Set(result.to_exclude.map((item) => item.id))
      );
      setCheckedModifyIds(
        new Set(result.to_modify.map((item) => item.id))
      );
      setPhase("preview");
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  /** 点击"确认执行" — 执行清洗 */
  const handleExecute = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await executeCleaning(
        Array.from(checkedExcludeIds),
        Array.from(checkedModifyIds)
      );
      setExecuteResult(result);
      setPhase("done");
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  /** 重置到初始状态 */
  const handleReset = () => {
    setPhase("idle");
    setPreview(null);
    setExecuteResult(null);
    setError(null);
    setCheckedExcludeIds(new Set());
    setCheckedModifyIds(new Set());
  };

  /** 切换单个排除项的勾选状态 */
  const toggleExclude = (id: string) => {
    setCheckedExcludeIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  };

  /** 切换单个修改项的勾选状态 */
  const toggleModify = (id: string) => {
    setCheckedModifyIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  };

  /** 全选/取消全选排除项 */
  const toggleAllExclude = (checked: boolean) => {
    if (!preview) return;
    setCheckedExcludeIds(
      checked ? new Set(preview.to_exclude.map((i) => i.id)) : new Set()
    );
  };

  /** 全选/取消全选修改项 */
  const toggleAllModify = (checked: boolean) => {
    if (!preview) return;
    setCheckedModifyIds(
      checked ? new Set(preview.to_modify.map((i) => i.id)) : new Set()
    );
  };

  // ----------------------------------------------------------------
  // 渲染
  // ----------------------------------------------------------------

  return (
    <Box>
      <Typography variant="h4" gutterBottom>
        数据清洗
      </Typography>

      {error && (
        <Alert severity="error" sx={{ mb: 2 }} onClose={() => setError(null)}>
          {error}
        </Alert>
      )}

      {loading && (
        <Box sx={{ display: "flex", justifyContent: "center", py: 4 }}>
          <CircularProgress />
        </Box>
      )}

      {/* ============================================================ */}
      {/* 初始状态：说明 + 开始清洗按钮 */}
      {/* ============================================================ */}
      {phase === "idle" && !loading && (
        <Box>
          <Card sx={{ mb: 3 }}>
            <CardContent>
              <Typography variant="h6" gutterBottom>
                清洗规则说明
              </Typography>
              <Stack spacing={1}>
                <Typography variant="body2">
                  <strong>待过滤（软删除，不参与汇总）：</strong>
                </Typography>
                <Typography
                  variant="body2"
                  component="div"
                  sx={{ pl: 2, color: "text.secondary" }}
                >
                  • 状态为「还款成功、交易关闭、退款成功、不计收入」的交易
                  <br />
                  • 支出方向且金额 ≤ 3 元的交易
                </Typography>
                <Typography variant="body2" sx={{ mt: 1 }}>
                  <strong>待修改（部分退款处理）：</strong>
                </Typography>
                <Typography
                  variant="body2"
                  component="div"
                  sx={{ pl: 2, color: "text.secondary" }}
                >
                  • 状态包含「部分退款」的交易
                  <br />• 退款金额 ≤ 3 元 → 状态改为「支付成功」，金额不变
                  <br />• 退款金额 {">"} 3 元 → 状态改为「部分退款」，金额 = 原金额 -
                  退款金额
                  <br />• 无法提取退款金额 → 仅修改状态，金额不变
                </Typography>
              </Stack>
            </CardContent>
          </Card>
          <Button
            variant="contained"
            size="large"
            startIcon={<CleaningServicesIcon />}
            onClick={handlePreview}
          >
            开始清洗
          </Button>
        </Box>
      )}

      {/* ============================================================ */}
      {/* 预览状态：展示待过滤 + 待修改列表 */}
      {/* ============================================================ */}
      {phase === "preview" && !loading && preview && (
        <Box>
          {preview.exclude_count === 0 && preview.modify_count === 0 ? (
            <Box>
              <Alert severity="success" sx={{ mb: 2 }}>
                没有需要清洗的交易，所有数据均已处理完毕。
              </Alert>
              <Button
                variant="outlined"
                startIcon={<ArrowBackIcon />}
                onClick={handleReset}
              >
                返回
              </Button>
            </Box>
          ) : (
            <>
              {/* 待过滤列表 */}
              {preview.to_exclude.length > 0 && (
                <Box sx={{ mb: 4 }}>
                  <Stack
                    direction="row"
                    alignItems="center"
                    spacing={1}
                    sx={{ mb: 1 }}
                  >
                    <Typography variant="h6">待过滤列表</Typography>
                    <Chip
                      label={`${checkedExcludeIds.size}/${preview.to_exclude.length} 条`}
                      size="small"
                      color="warning"
                    />
                  </Stack>
                  <TableContainer component={Paper} variant="outlined">
                    <Table size="small">
                      <TableHead>
                        <TableRow>
                          <TableCell padding="checkbox">
                            <Checkbox
                              checked={
                                checkedExcludeIds.size ===
                                  preview.to_exclude.length &&
                                preview.to_exclude.length > 0
                              }
                              indeterminate={
                                checkedExcludeIds.size > 0 &&
                                checkedExcludeIds.size <
                                  preview.to_exclude.length
                              }
                              onChange={(e) =>
                                toggleAllExclude(e.target.checked)
                              }
                            />
                          </TableCell>
                          <TableCell>交易时间</TableCell>
                          <TableCell>来源</TableCell>
                          <TableCell>交易对方</TableCell>
                          <TableCell>商品</TableCell>
                          <TableCell>收/支</TableCell>
                          <TableCell align="right">金额</TableCell>
                          <TableCell>状态</TableCell>
                          <TableCell>过滤原因</TableCell>
                        </TableRow>
                      </TableHead>
                      <TableBody>
                        {preview.to_exclude.map((item) => (
                          <TableRow key={item.id}>
                            <TableCell padding="checkbox">
                              <Checkbox
                                checked={checkedExcludeIds.has(item.id)}
                                onChange={() => toggleExclude(item.id)}
                              />
                            </TableCell>
                            <TableCell>{item.transaction_time}</TableCell>
                            <TableCell>
                              <Chip
                                label={sourceLabel(item.source)}
                                color={sourceColor(item.source)}
                                size="small"
                              />
                            </TableCell>
                            <TableCell>{item.counterparty || "-"}</TableCell>
                            <TableCell>{item.product || "-"}</TableCell>
                            <TableCell>
                              {directionLabel(item.direction)}
                            </TableCell>
                            <TableCell align="right">
                              {formatAmount(item.amount)}
                            </TableCell>
                            <TableCell>{item.status || "-"}</TableCell>
                            <TableCell>
                              <Typography
                                variant="body2"
                                color="text.secondary"
                              >
                                {item.reason}
                              </Typography>
                            </TableCell>
                          </TableRow>
                        ))}
                      </TableBody>
                    </Table>
                  </TableContainer>
                </Box>
              )}

              {/* 待修改列表 */}
              {preview.to_modify.length > 0 && (
                <Box sx={{ mb: 4 }}>
                  <Stack
                    direction="row"
                    alignItems="center"
                    spacing={1}
                    sx={{ mb: 1 }}
                  >
                    <Typography variant="h6">
                      待修改列表（部分退款处理）
                    </Typography>
                    <Chip
                      label={`${checkedModifyIds.size}/${preview.to_modify.length} 条`}
                      size="small"
                      color="info"
                    />
                  </Stack>
                  <TableContainer component={Paper} variant="outlined">
                    <Table size="small">
                      <TableHead>
                        <TableRow>
                          <TableCell padding="checkbox">
                            <Checkbox
                              checked={
                                checkedModifyIds.size ===
                                  preview.to_modify.length &&
                                preview.to_modify.length > 0
                              }
                              indeterminate={
                                checkedModifyIds.size > 0 &&
                                checkedModifyIds.size <
                                  preview.to_modify.length
                              }
                              onChange={(e) =>
                                toggleAllModify(e.target.checked)
                              }
                            />
                          </TableCell>
                          <TableCell>交易时间</TableCell>
                          <TableCell>来源</TableCell>
                          <TableCell>交易对方</TableCell>
                          <TableCell>原状态 → 新状态</TableCell>
                          <TableCell align="right">
                            原金额 → 新金额
                          </TableCell>
                          <TableCell>备注</TableCell>
                        </TableRow>
                      </TableHead>
                      <TableBody>
                        {preview.to_modify.map((item) => (
                          <TableRow key={item.id}>
                            <TableCell padding="checkbox">
                              <Checkbox
                                checked={checkedModifyIds.has(item.id)}
                                onChange={() => toggleModify(item.id)}
                              />
                            </TableCell>
                            <TableCell>{item.transaction_time}</TableCell>
                            <TableCell>
                              <Chip
                                label={sourceLabel(item.source)}
                                color={sourceColor(item.source)}
                                size="small"
                              />
                            </TableCell>
                            <TableCell>{item.counterparty || "-"}</TableCell>
                            <TableCell>
                              <Typography variant="body2">
                                {item.original_status || "-"} →{" "}
                                <strong>{item.new_status}</strong>
                              </Typography>
                            </TableCell>
                            <TableCell align="right">
                              <Typography variant="body2">
                                {formatAmount(item.original_amount)} →{" "}
                                <strong>
                                  {item.new_amount !== null
                                    ? formatAmount(item.new_amount)
                                    : "不变"}
                                </strong>
                              </Typography>
                            </TableCell>
                            <TableCell>
                              <Typography
                                variant="body2"
                                color="text.secondary"
                              >
                                {item.note}
                              </Typography>
                            </TableCell>
                          </TableRow>
                        ))}
                      </TableBody>
                    </Table>
                  </TableContainer>
                </Box>
              )}

              <Divider sx={{ my: 2 }} />

              <Stack direction="row" spacing={2}>
                <Button
                  variant="contained"
                  color="primary"
                  size="large"
                  startIcon={<CheckCircleIcon />}
                  onClick={handleExecute}
                  disabled={loading}
                >
                  确认执行（过滤 {checkedExcludeIds.size} 条 / 修改{" "}
                  {checkedModifyIds.size} 条）
                </Button>
                <Button
                  variant="outlined"
                  size="large"
                  onClick={handleReset}
                  disabled={loading}
                >
                  取消
                </Button>
              </Stack>
            </>
          )}
        </Box>
      )}

      {/* ============================================================ */}
      {/* 完成状态：展示执行结果 */}
      {/* ============================================================ */}
      {phase === "done" && !loading && executeResult && (
        <Box>
          <Card sx={{ mb: 3 }}>
            <CardContent>
              <Stack
                direction="row"
                alignItems="center"
                spacing={1}
                sx={{ mb: 2 }}
              >
                <CheckCircleIcon color="success" />
                <Typography variant="h6">清洗完成</Typography>
              </Stack>
              <Stack spacing={1}>
                <Typography variant="body1">
                  已过滤（排除）：{executeResult.excluded_count} 条
                </Typography>
                <Typography variant="body1">
                  已修改（部分退款）：{executeResult.modified_count} 条
                </Typography>
                <Typography variant="body1">
                  剩余有效交易：{executeResult.remaining_count} 条
                </Typography>
              </Stack>
            </CardContent>
          </Card>
          <Button
            variant="outlined"
            startIcon={<ArrowBackIcon />}
            onClick={handleReset}
          >
            返回
          </Button>
        </Box>
      )}
    </Box>
  );
}

export default CleaningPage;
