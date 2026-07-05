import { useState, useEffect, useCallback } from "react";
import {
  Box, Typography, Alert, CircularProgress, Button, Card, CardContent,
  Stack, Select, MenuItem, TextField, Table, TableBody, TableCell,
  TableHead, TableRow, TableContainer, Paper, Chip, Tabs, Tab,
  IconButton, Dialog, DialogTitle, DialogContent, DialogActions,
  Switch, FormControlLabel, Divider, Snackbar, FormControl, InputLabel,
} from "@mui/material";
import AddIcon from "@mui/icons-material/Add";
import DeleteIcon from "@mui/icons-material/Delete";
import EditIcon from "@mui/icons-material/Edit";
import SearchIcon from "@mui/icons-material/Search";
import {
  listAllRules, createRule, updateRule, deleteRule, toggleRule,
  listAiRules, deleteAiRule,
  createTag, updateTag, deleteTag,
  listMappings, createMapping, deleteMapping,
  testRuleMatch,
} from "../api/rule";
import { listCategoryTags } from "../api/classification";
import { listAccountOwners, createAccountOwner, updateAccountOwner, deleteAccountOwner } from "../api/account_owner";
import type { CategoryRule, CategoryTag, AiLearningRule, SummaryMapping, AccountOwner } from "../types";

type TabKey = "rules" | "ai" | "tags" | "mappings" | "test" | "owners";

const FIELD_LABELS: Record<string, string> = { counterparty: "交易对方", product: "商品", transaction_type: "交易类型" };
const TYPE_LABELS: Record<string, string> = { exact: "精确匹配", like: "包含匹配", in: "列表匹配" };
const TAG_COLORS: Record<string, string> = { "房租":"#e91e63","买菜":"#4caf50","餐饮":"#ff9800","大餐":"#f44336","水果":"#8bc34a","衣服美妆":"#9c27b0","零食饮料":"#ff5722","话费":"#2196f3","交通":"#607d8b","日用品":"#795548","医疗药品":"#00bcd4","九九":"#cddc39","会员":"#3f51b5","运动":"#009688","其他":"#9e9e9e","荭包":"#e91e63","家具":"#795548","游玩":"#ff9800","旅游":"#03a9f4","学习":"#4caf50","礼物":"#e91e63","给我的宝":"#ff4081","车子":"#607d8b","烘焙":"#ff9800" };

const tagStyle = (name: string) => name ? { display:"inline-block",padding:"2px 10px",borderRadius:12,fontSize:11,fontWeight:600,color:"#fff",background:TAG_COLORS[name]||"#666",whiteSpace:"nowrap" as const,lineHeight:"20px" } : {};

function RulesPage() {
  const [tab, setTab] = useState<TabKey>("rules");
  const [error, setError] = useState<string | null>(null);
  const [snack, setSnack] = useState<string | null>(null);

  // 分类规则
  const [rules, setRules] = useState<CategoryRule[]>([]);
  const [rulesLoading, setRulesLoading] = useState(false);
  // AI 学习规则
  const [aiRules, setAiRules] = useState<AiLearningRule[]>([]);
  // 标签
  const [tags, setTags] = useState<CategoryTag[]>([]);
  const [tagsLoading, setTagsLoading] = useState(false);
  // 映射
  const [mappings, setMappings] = useState<SummaryMapping[]>([]);
  const [mappingsLoading, setMappingsLoading] = useState(false);

  // 弹窗
  const [ruleDialog, setRuleDialog] = useState(false);
  const [editingRule, setEditingRule] = useState<CategoryRule | null>(null);
  const [tagDialog, setTagDialog] = useState(false);
  const [editingTag, setEditingTag] = useState<CategoryTag | null>(null);
  const [newTagName, setNewTagName] = useState("");
  const [mappingCat, setMappingCat] = useState("");
  const [mappingTagId, setMappingTagId] = useState("");

  // 排序
  const [sortField, setSortField] = useState<string>("priority");
  const [sortDir, setSortDir] = useState<"asc" | "desc">("asc");
  const handleSort = (field: string) => {
    if (sortField === field) setSortDir((d) => d === "asc" ? "desc" : "asc");
    else { setSortField(field); setSortDir("asc"); }
  };
  const sortedRules = [...rules].sort((a: any, b: any) => {
    const va = a[sortField] ?? "", vb = b[sortField] ?? "";
    if (typeof va === "number") return sortDir === "asc" ? va - vb : vb - va;
    return sortDir === "asc" ? String(va).localeCompare(String(vb)) : String(vb).localeCompare(String(va));
  });

  // 归属人
  const [owners, setOwners] = useState<AccountOwner[]>([]);
  const [ownerDialog, setOwnerDialog] = useState(false);
  const [ownerName, setOwnerName] = useState("");
  const [editingOwner, setEditingOwner] = useState<AccountOwner | null>(null);
  const [delOwner, setDelOwner] = useState<AccountOwner | null>(null);
  const loadOwners = useCallback(async () => { try { setOwners(await listAccountOwners()); } catch(e) { setError(String(e)); } }, []);
  useEffect(() => { loadOwners(); }, [loadOwners]);
  const saveOwner = async () => {
    if (!ownerName.trim()) return;
    try {
      if (editingOwner) await updateAccountOwner(editingOwner.id, ownerName.trim());
      else await createAccountOwner(ownerName.trim());
      setOwnerDialog(false); loadOwners();
    } catch(e) { setError(String(e)); }
  };
  const handleDeleteOwner = async () => {
    if (!delOwner) return;
    try { await deleteAccountOwner(delOwner.id); setDelOwner(null); loadOwners(); } catch(e) { setError(String(e)); }
  };

  // 规则测试
  const [testCp, setTestCp] = useState("");
  const [testProd, setTestProd] = useState("");
  const [testType, setTestType] = useState("");
  const [testResult, setTestResult] = useState<string | null>(null);

  // ----------------------------------------------------------------
  const loadRules = useCallback(async () => {
    setRulesLoading(true);
    try { setRules(await listAllRules()); } catch (e) { setError(String(e)); }
    finally { setRulesLoading(false); }
  }, []);
  const loadAiRules = useCallback(async () => {
    try { setAiRules(await listAiRules()); } catch (e) { setError(String(e)); }
  }, []);
  const loadTags = useCallback(async () => {
    setTagsLoading(true);
    try { setTags(await listCategoryTags()); } catch (e) { setError(String(e)); }
    finally { setTagsLoading(false); }
  }, []);
  const loadMappings = useCallback(async () => {
    setMappingsLoading(true);
    try { setMappings(await listMappings()); } catch (e) { setError(String(e)); }
    finally { setMappingsLoading(false); }
  }, []);

  useEffect(() => { loadRules(); loadAiRules(); loadTags(); loadMappings(); },
    [loadRules, loadAiRules, loadTags, loadMappings]);

  // ---- 规则表单字段 ----
  const [rf, setRf] = useState({ match_field: "counterparty", match_type: "exact", match_value: "", target_tag_id: "", priority: 100 });

  const openRuleDialog = (rule?: CategoryRule) => {
    if (rule) {
      setEditingRule(rule);
      setRf({ match_field: rule.match_field, match_type: rule.match_type, match_value: rule.match_value, target_tag_id: rule.target_tag_id, priority: rule.priority });
    } else {
      setEditingRule(null);
      setRf({ match_field: "counterparty", match_type: "exact", match_value: "", target_tag_id: tags[0]?.id || "", priority: 100 });
    }
    setRuleDialog(true);
  };

  const saveRule = async () => {
    try {
      if (editingRule) {
        await updateRule({ ...editingRule, ...rf });
      } else {
        await createRule({ id: "", match_field: rf.match_field, match_type: rf.match_type, match_value: rf.match_value, target_tag_id: rf.target_tag_id, priority: rf.priority, enabled: 1, source: "user", created_at: "", updated_at: "" });
      }
      setSnack("规则已保存"); setRuleDialog(false); loadRules();
    } catch (e) { setError(String(e)); }
  };

  // ---- 标签 ----
  const openTagDialog = (tag?: CategoryTag) => {
    setEditingTag(tag || null);
    setNewTagName(tag?.name || "");
    setTagDialog(true);
  };

  const saveTag = async () => {
    try {
      if (editingTag) {
        await updateTag(editingTag.id, newTagName, editingTag.sort_order);
      } else {
        await createTag(newTagName, tags.length + 1);
      }
      setSnack("标签已保存"); setTagDialog(false); loadTags();
    } catch (e) { setError(String(e)); }
  };

  // ---- 映射 ----
  const handleAddMapping = async () => {
    if (!mappingCat || !mappingTagId) return;
    try {
      await createMapping(mappingCat, mappingTagId, mappings.length + 1);
      setSnack("映射已添加"); setMappingCat(""); setMappingTagId(""); loadMappings();
    } catch (e) { setError(String(e)); }
  };

  // ---- 规则测试 ----
  const handleTest = async () => {
    try {
      const r = await testRuleMatch(testCp, testProd, testType);
      setTestResult(r ? `匹配成功：${r}` : "未匹配到任何规则");
    } catch (e) { setError(String(e)); }
  };

  const tagNames = (ids: string) => ids.split(",").map((tid) => {
    const t = tags.find((x) => x.id === tid.trim());
    return t?.name || tid;
  }).join(", ");

  return (
    <Box>
      <Typography variant="h4" gutterBottom>规则管理</Typography>
      {error && <Alert severity="error" sx={{ mb: 2 }} onClose={() => setError(null)}>{error}</Alert>}
      <Snackbar open={!!snack} autoHideDuration={2000} onClose={() => setSnack(null)} message={snack} anchorOrigin={{ vertical: "top", horizontal: "center" }} />

      <Tabs value={tab} onChange={(_, v) => setTab(v)} sx={{ mb: 2 }}>
        <Tab label="分类规则" value="rules" />
        <Tab label="AI 学习规则" value="ai" />
        <Tab label="标签管理" value="tags" />
        <Tab label="汇总映射" value="mappings" />
        <Tab label="规则测试" value="test" />
        <Tab label="归属人" value="owners" />
      </Tabs>

      {/* === 分类规则 === */}
      {tab === "rules" && (
        <>
          <Stack direction="row" spacing={2} sx={{ mb: 2 }}>
            <Button variant="contained" startIcon={<AddIcon />} onClick={() => openRuleDialog()}>新增规则</Button>
            <Typography variant="body2" color="text.secondary" sx={{ alignSelf: "center" }}>
              共 {rules.length} 条（内置 122 + 用户自定义）
            </Typography>
          </Stack>
          {rulesLoading ? <CircularProgress /> : (
            <TableContainer component={Paper} variant="outlined">
              <Table size="small">
                <TableHead>
                  <TableRow>
                    <TableCell sx={{ width: 40 }}>#</TableCell>
                    <TableCell sx={{ cursor:"pointer" }} onClick={() => handleSort("match_field")}>
                      匹配字段 {sortField === "match_field" ? (sortDir === "asc" ? "▲" : "▼") : ""}
                    </TableCell>
                    <TableCell sx={{ cursor:"pointer" }} onClick={() => handleSort("match_type")}>
                      匹配方式 {sortField === "match_type" ? (sortDir === "asc" ? "▲" : "▼") : ""}
                    </TableCell>
                    <TableCell>匹配值</TableCell>
                    <TableCell sx={{ cursor:"pointer" }} onClick={() => handleSort("target_tag_id")}>
                      目标标签 {sortField === "target_tag_id" ? (sortDir === "asc" ? "▲" : "▼") : ""}
                    </TableCell>
                    <TableCell sx={{ cursor:"pointer" }} onClick={() => handleSort("priority")}>
                      优先级 {sortField === "priority" ? (sortDir === "asc" ? "▲" : "▼") : ""}
                    </TableCell>
                    <TableCell sx={{ cursor:"pointer" }} onClick={() => handleSort("source")}>
                      来源 {sortField === "source" ? (sortDir === "asc" ? "▲" : "▼") : ""}
                    </TableCell>
                    <TableCell>启用</TableCell>
                    <TableCell align="center">操作</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {sortedRules.map((r, i) => (
                    <TableRow key={r.id}>
                      <TableCell>{i + 1}</TableCell>
                      <TableCell>{FIELD_LABELS[r.match_field] || r.match_field}</TableCell>
                      <TableCell><Chip label={TYPE_LABELS[r.match_type] || r.match_type} size="small" /></TableCell>
                      <TableCell sx={{ maxWidth: 200, overflow: "hidden", textOverflow: "ellipsis" }}>{r.match_value}</TableCell>
                      <TableCell><span style={tagStyle(tagNames(r.target_tag_id))}>{tagNames(r.target_tag_id)}</span></TableCell>
                      <TableCell>{r.priority}</TableCell>
                      <TableCell><Chip label={r.source === "builtin" ? "内置" : "用户"} size="small" color={r.source === "builtin" ? "default" : "primary"} /></TableCell>
                      <TableCell>
                        <Switch size="small" checked={r.enabled === 1} onChange={async (_, v) => { await toggleRule(r.id, v); loadRules(); }} />
                      </TableCell>
                      <TableCell align="center">
                        <IconButton size="small" onClick={() => openRuleDialog(r)}><EditIcon fontSize="small" /></IconButton>
                        <IconButton size="small" onClick={async () => { await deleteRule(r.id); loadRules(); }}><DeleteIcon fontSize="small" /></IconButton>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </TableContainer>
          )}
        </>
      )}

      {/* === AI 学习规则 === */}
      {tab === "ai" && (
        <TableContainer component={Paper} variant="outlined">
          <Table size="small">
            <TableHead>
              <TableRow>
                <TableCell sx={{ width: 40 }}>#</TableCell>
                <TableCell>交易对方</TableCell><TableCell>商品</TableCell><TableCell>交易类型</TableCell>
                <TableCell align="right">金额</TableCell>
                <TableCell>标签</TableCell><TableCell>置信度</TableCell>
                <TableCell>确认/修正</TableCell><TableCell align="center">操作</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {aiRules.map((r, i) => (
                <TableRow key={r.id}>
                  <TableCell>{i + 1}</TableCell>
                  <TableCell>{r.counterparty || "-"}</TableCell>
                  <TableCell>{r.product || "-"}</TableCell>
                  <TableCell>{r.transaction_type || "-"}</TableCell>
                  <TableCell align="right">{r.amount != null ? `¥${r.amount.toFixed(2)}` : "-"}</TableCell>
                  <TableCell><span style={tagStyle(tagNames(r.target_tag_id))}>{tagNames(r.target_tag_id)}</span></TableCell>
                  <TableCell>{(r.confidence * 100).toFixed(0)}%</TableCell>
                  <TableCell>{r.confirm_count}/{r.correct_count}</TableCell>
                  <TableCell align="center">
                    <IconButton size="small" onClick={async () => { await deleteAiRule(r.id); loadAiRules(); }}><DeleteIcon fontSize="small" /></IconButton>
                  </TableCell>
                </TableRow>
              ))}
              {aiRules.length === 0 && (
                <TableRow><TableCell colSpan={9} align="center">暂无 AI 学习规则（AI 辅助分类后自动生成）</TableCell></TableRow>
              )}
            </TableBody>
          </Table>
        </TableContainer>
      )}

      {/* === 标签管理 === */}
      {tab === "tags" && (
        <>
          <Button variant="contained" startIcon={<AddIcon />} sx={{ mb: 2 }} onClick={() => openTagDialog()}>新增标签</Button>
          {tagsLoading ? <CircularProgress /> : (
            <TableContainer component={Paper} variant="outlined">
              <Table size="small">
                <TableHead><TableRow><TableCell sx={{ width: 40 }}>#</TableCell><TableCell>ID</TableCell><TableCell>名称</TableCell><TableCell>系统</TableCell><TableCell>排序</TableCell><TableCell align="center">操作</TableCell></TableRow></TableHead>
                <TableBody>
                  {tags.map((t, i) => (
                    <TableRow key={t.id}>
                      <TableCell>{i + 1}</TableCell>
                      <TableCell>{t.id}</TableCell><TableCell>{t.name}</TableCell>
                      <TableCell>{t.is_system === 1 ? "✓" : ""}</TableCell><TableCell>{t.sort_order}</TableCell>
                      <TableCell align="center">
                        <IconButton size="small" onClick={() => openTagDialog(t)}><EditIcon fontSize="small" /></IconButton>
                        <IconButton size="small" disabled={t.is_system === 1} onClick={async () => { await deleteTag(t.id); loadTags(); }}><DeleteIcon fontSize="small" /></IconButton>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </TableContainer>
          )}
        </>
      )}

      {/* === 汇总映射 === */}
      {tab === "mappings" && (
        <>
          <Stack direction="row" spacing={2} sx={{ mb: 2 }}>
            <TextField size="small" label="汇总类名称" value={mappingCat} onChange={(e) => setMappingCat(e.target.value)} sx={{ width: 160 }} />
            <FormControl size="small" sx={{ minWidth: 140 }}>
              <InputLabel>子标签</InputLabel>
              <Select value={mappingTagId} label="子标签" onChange={(e) => setMappingTagId(e.target.value)}>
                {tags.map((t) => <MenuItem key={t.id} value={t.id}>{t.name}</MenuItem>)}
              </Select>
            </FormControl>
            <Button variant="contained" onClick={handleAddMapping} disabled={!mappingCat || !mappingTagId}>添加映射</Button>
          </Stack>
          {mappingsLoading ? <CircularProgress /> : (
            <TableContainer component={Paper} variant="outlined">
              <Table size="small">
                <TableHead><TableRow><TableCell sx={{ width: 40 }}>#</TableCell><TableCell>汇总类</TableCell><TableCell>标签</TableCell><TableCell align="center">操作</TableCell></TableRow></TableHead>
                <TableBody>
                  {mappings.map((m, i) => (
                    <TableRow key={m.id}>
                      <TableCell>{i + 1}</TableCell>
                      <TableCell>{m.summary_category}</TableCell>
                      <TableCell>{tagNames(m.tag_id)}</TableCell>
                      <TableCell align="center">
                        <IconButton size="small" onClick={async () => { await deleteMapping(m.id); loadMappings(); }}><DeleteIcon fontSize="small" /></IconButton>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </TableContainer>
          )}
        </>
      )}

      {/* === 规则测试 === */}
      {tab === "test" && (
        <Card variant="outlined">
          <CardContent>
            <Typography variant="h6" gutterBottom>测试规则匹配</Typography>
            <Stack spacing={2} sx={{ maxWidth: 400 }}>
              <TextField size="small" label="交易对方" value={testCp} onChange={(e) => setTestCp(e.target.value)} placeholder="例如：蜜雪冰城" />
              <TextField size="small" label="商品" value={testProd} onChange={(e) => setTestProd(e.target.value)} />
              <TextField size="small" label="交易类型" value={testType} onChange={(e) => setTestType(e.target.value)} placeholder="例如：交通出行" />
              <Button variant="contained" startIcon={<SearchIcon />} onClick={handleTest}>测试匹配</Button>
              {testResult && <Alert severity={testResult.includes("成功") ? "success" : "info"}>{testResult}</Alert>}
            </Stack>
          </CardContent>
        </Card>
      )}

      {/* 规则编辑弹窗 */}
      <Dialog open={ruleDialog} onClose={() => setRuleDialog(false)} maxWidth="sm" fullWidth>
        <DialogTitle>{editingRule ? "编辑规则" : "新增规则"}</DialogTitle>
        <DialogContent>
          <Stack spacing={2} sx={{ mt: 1 }}>
            <FormControl fullWidth size="small">
              <InputLabel>匹配字段</InputLabel>
              <Select value={rf.match_field} label="匹配字段" onChange={(e) => setRf({ ...rf, match_field: e.target.value })}>
                <MenuItem value="counterparty">交易对方</MenuItem>
                <MenuItem value="product">商品</MenuItem>
                <MenuItem value="transaction_type">交易类型</MenuItem>
              </Select>
            </FormControl>
            <FormControl fullWidth size="small">
              <InputLabel>匹配方式</InputLabel>
              <Select value={rf.match_type} label="匹配方式" onChange={(e) => setRf({ ...rf, match_type: e.target.value })}>
                <MenuItem value="exact">精确匹配 (exact)</MenuItem>
                <MenuItem value="like">包含匹配 (like)</MenuItem>
                <MenuItem value="in">列表匹配 (in)</MenuItem>
              </Select>
            </FormControl>
            <TextField size="small" label="匹配值" fullWidth value={rf.match_value} onChange={(e) => setRf({ ...rf, match_value: e.target.value })}
              helperText={rf.match_type === "in" ? "多个值用逗号分隔" : ""} />
            <FormControl fullWidth size="small">
              <InputLabel>目标标签</InputLabel>
              <Select value={rf.target_tag_id} label="目标标签" onChange={(e) => setRf({ ...rf, target_tag_id: e.target.value })}>
                {tags.map((t) => <MenuItem key={t.id} value={t.id}>{t.name}</MenuItem>)}
              </Select>
            </FormControl>
            <TextField size="small" label="优先级" type="number" fullWidth value={rf.priority} onChange={(e) => setRf({ ...rf, priority: parseInt(e.target.value) || 100 })} />
          </Stack>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setRuleDialog(false)}>取消</Button>
          <Button variant="contained" onClick={saveRule}>保存</Button>
        </DialogActions>
      </Dialog>

      {/* === 归属人 === */}
      {tab === "owners" && (
        <>
          <Stack direction="row" spacing={2} sx={{ mb: 2 }}>
            <Button variant="contained" startIcon={<AddIcon />} onClick={() => { setEditingOwner(null); setOwnerName(""); setOwnerDialog(true); }}>新增归属人</Button>
          </Stack>
          <TableContainer component={Paper} variant="outlined">
            <Table size="small">
              <TableHead><TableRow><TableCell sx={{ width: 40 }}>#</TableCell><TableCell>名称</TableCell><TableCell>创建时间</TableCell><TableCell align="center">操作</TableCell></TableRow></TableHead>
              <TableBody>
                {owners.map((o, i) => (
                  <TableRow key={o.id}>
                    <TableCell>{i + 1}</TableCell><TableCell>{o.name}</TableCell><TableCell>{o.created_at}</TableCell>
                    <TableCell align="center">
                      <IconButton size="small" onClick={() => { setEditingOwner(o); setOwnerName(o.name); setOwnerDialog(true); }}><EditIcon fontSize="small" /></IconButton>
                      <IconButton size="small" color="error" onClick={() => setDelOwner(o)}><DeleteIcon fontSize="small" /></IconButton>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </TableContainer>
        </>
      )}

      {/* 标签编辑弹窗 */}
      <Dialog open={tagDialog} onClose={() => setTagDialog(false)} maxWidth="xs" fullWidth>
        <DialogTitle>{editingTag ? "编辑标签" : "新增标签"}</DialogTitle>
        <DialogContent>
          <TextField autoFocus fullWidth size="small" label="标签名称" value={newTagName} onChange={(e) => setNewTagName(e.target.value)} sx={{ mt: 1 }} />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setTagDialog(false)}>取消</Button>
          <Button variant="contained" onClick={saveTag} disabled={!newTagName}>保存</Button>
        </DialogActions>
      </Dialog>
      {/* 归属人弹窗 */}
      <Dialog open={ownerDialog} onClose={() => setOwnerDialog(false)} maxWidth="xs" fullWidth>
        <DialogTitle>{editingOwner ? "编辑归属人" : "新增归属人"}</DialogTitle>
        <DialogContent>
          <TextField autoFocus fullWidth size="small" label="归属人名称" value={ownerName} onChange={(e) => setOwnerName(e.target.value)} sx={{ mt: 1 }} />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setOwnerDialog(false)}>取消</Button>
          <Button variant="contained" onClick={saveOwner} disabled={!ownerName.trim()}>保存</Button>
        </DialogActions>
      </Dialog>
      <Dialog open={!!delOwner} onClose={() => setDelOwner(null)} maxWidth="xs" fullWidth>
        <DialogTitle>确认删除</DialogTitle>
        <DialogContent><Typography>确定删除归属人「{delOwner?.name}」吗？关联交易将变为空。</Typography></DialogContent>
        <DialogActions>
          <Button onClick={() => setDelOwner(null)}>取消</Button>
          <Button variant="contained" color="error" onClick={handleDeleteOwner}>删除</Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
}

export default RulesPage;
