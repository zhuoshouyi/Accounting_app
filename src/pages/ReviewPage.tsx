import { useState, useEffect, useCallback } from "react";
import {
  Box, Typography, Alert, CircularProgress, Table, TableBody, TableCell,
  TableHead, TableRow, TableContainer, Paper, Chip, Stack, Select, MenuItem,
  FormControl, InputLabel, Button, Checkbox, Card, CardContent, Tooltip,
  Dialog, DialogTitle, DialogContent, DialogActions, TextField,
} from "@mui/material";
import AutoModeIcon from "@mui/icons-material/AutoMode";
import PsychologyIcon from "@mui/icons-material/Psychology";
import RefreshIcon from "@mui/icons-material/Refresh";
import CheckIcon from "@mui/icons-material/Check";
import SkipNextIcon from "@mui/icons-material/SkipNext";
import {
  classifyTransactions, listUnclassified, updateTransactionTag,
  batchUpdateTags, listCategoryTags, listSkipped, unskipTransaction,
  type ClassifyResult,
} from "../api/classification";
import { aiClassify, confirmAiTag, correctAiTag } from "../api/ai";
import { createRule } from "../api/rule";
import type { Transaction, CategoryTag } from "../types";

function ReviewPage() {
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [tags, setTags] = useState<CategoryTag[]>([]);
  const [loading, setLoading] = useState(false);
  const [classifying, setClassifying] = useState(false);
  const [aiClassifying, setAiClassifying] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [classifyResult, setClassifyResult] = useState<ClassifyResult | null>(null);
  const [aiResultCount, setAiResultCount] = useState<number | null>(null);

  const [rowTagSelections, setRowTagSelections] = useState<Record<string, string>>({});
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [batchTagId, setBatchTagId] = useState<string>("");
  const [actionLoadingId, setActionLoadingId] = useState<string | null>(null);

  const [skipped, setSkipped] = useState<Transaction[]>([]);
  const [showSkipped, setShowSkipped] = useState(false);

  // 添加规则
  const [ruleDialog, setRuleDialog] = useState(false);
  const [ruleTx, setRuleTx] = useState<Transaction | null>(null);
  const [ruleField, setRuleField] = useState("counterparty");
  const [ruleType, setRuleType] = useState("exact");
  const [ruleValue, setRuleValue] = useState("");
  const [ruleTagId, setRuleTagId] = useState("");

  const sl = (s: string) => ({ wechat:"微信",alipay:"支付宝",manual:"手动" }[s]||s);
  const sc = (s: string) => ({ wechat:"success" as const,alipay:"primary" as const }[s]||"default" as const);
  const dl = (d: string) => ({ expense:"支出",income:"收入",neutral:"不计" }[d]||d);
  const dc = (d: string) => ({ expense:"error" as const,income:"success" as const }[d]||"default" as const);
  const tagMap = new Map<string,string>(); tags.forEach((t)=>tagMap.set(t.id,t.name));

  const loadData = useCallback(async () => {
    setLoading(true);setError(null);
    try { const [tx,tg] = await Promise.all([listUnclassified(),listCategoryTags()]); setTransactions(tx);setTags(tg);setRowTagSelections({});setSelectedIds(new Set()); }
    catch(e){setError(String(e))} finally{setLoading(false)}
  },[]);
  useEffect(()=>{loadData()},[loadData]);

  const loadSkipped = useCallback(async () => { try { setSkipped(await listSkipped()); } catch(e){setError(String(e))} },[]);

  const removeTx = (id:string) => {
    setTransactions(p=>p.filter(t=>t.id!==id));
    setRowTagSelections(p=>{const n={...p};delete n[id];return n});
    setSelectedIds(p=>{const n=new Set(p);n.delete(id);return n});
  };

  // ---- 自动分类 ----
  const handleClassify = async () => {
    setClassifying(true);setError(null);
    try { setClassifyResult(await classifyTransactions()); await loadData(); } catch(e){setError(String(e))} finally{setClassifying(false)}
  };
  // ---- AI 分类 ----
  const handleAiClassify = async () => {
    setAiClassifying(true);setError(null);setAiResultCount(null);
    try { const r=await aiClassify(); setAiResultCount(r.filter(([,x])=>x.tag_name).length); await loadData(); } catch(e){setError(String(e))} finally{setAiClassifying(false)}
  };
  // ---- AI 采纳/修正 ----
  const handleConfirmAi = async (tx:Transaction) => {
    if(!tx.ai_suggested_tag)return; setActionLoadingId(tx.id);setError(null);
    try { await confirmAiTag(tx.id,tx.ai_suggested_tag,tx.counterparty||"",tx.product||"",tx.transaction_type||"",tx.amount); removeTx(tx.id); } catch(e){setError(String(e))} finally{setActionLoadingId(null)}
  };
  const handleCorrectAi = async (tx:Transaction) => {
    const tid=rowTagSelections[tx.id]; if(!tid){setError("请先选择标签");return}
    setActionLoadingId(tx.id);setError(null);
    try { await correctAiTag(tx.id,tid,tx.counterparty||"",tx.product||"",tx.transaction_type||"",tx.amount); removeTx(tx.id); } catch(e){setError(String(e))} finally{setActionLoadingId(null)}
  };
  // ---- 手动 ----
  const handleConfirmTag = async (txId:string) => {
    const tid=rowTagSelections[txId]; if(!tid){setError("请先选择标签");return}
    setActionLoadingId(txId);setError(null);
    try { await updateTransactionTag(txId,tid,"manual"); removeTx(txId); } catch(e){setError(String(e))} finally{setActionLoadingId(null)}
  };
  const handleSkip = async (txId:string) => {
    setActionLoadingId(txId);setError(null);
    try { const tx=transactions.find(t=>t.id===txId); await updateTransactionTag(txId,null,"manual"); removeTx(txId); if(tx)setSkipped(p=>[tx,...p]); } catch(e){setError(String(e))} finally{setActionLoadingId(null)}
  };
  // ---- 恢复 ----
  const handleUnskip = async (id:string) => {
    setActionLoadingId(id);setError(null);
    try { const tx=skipped.find(t=>t.id===id); await unskipTransaction(id); setSkipped(p=>p.filter(t=>t.id!==id)); if(tx)setTransactions(p=>[tx,...p]); } catch(e){setError(String(e))} finally{setActionLoadingId(null)}
  };
  // ---- 批量 ----
  const handleToggleSelect = (id:string) => setSelectedIds(p=>{const n=new Set(p);n.has(id)?n.delete(id):n.add(id);return n});
  const handleToggleSelectAll = () => setSelectedIds(p=>p.size===transactions.length?new Set():new Set(transactions.map(t=>t.id)));
  const handleBatchConfirm = async () => {
    if(!batchTagId||selectedIds.size===0)return; setActionLoadingId("__batch__");setError(null);
    try { const ids=Array.from(selectedIds); await batchUpdateTags(ids,batchTagId,"manual"); setTransactions(p=>p.filter(t=>!ids.includes(t.id))); setSelectedIds(new Set());setBatchTagId(""); } catch(e){setError(String(e))} finally{setActionLoadingId(null)}
  };
  const handleBatchAcceptAi = async () => {
    if(selectedIds.size===0)return; setActionLoadingId("__batch__");setError(null);
    try { const toAccept=transactions.filter(t=>selectedIds.has(t.id)&&t.ai_suggested_tag); for(const tx of toAccept) await confirmAiTag(tx.id,tx.ai_suggested_tag!,tx.counterparty||"",tx.product||"",tx.transaction_type||"",tx.amount); setTransactions(p=>p.filter(t=>!toAccept.find(a=>a.id===t.id))); setSelectedIds(new Set()); } catch(e){setError(String(e))} finally{setActionLoadingId(null)}
  };
  // ---- 加规则 ----
  const openRuleDialog = (tx:Transaction) => {
    setRuleTx(tx); const f=tx.counterparty?"counterparty":tx.product?"product":"transaction_type";
    setRuleField(f); setRuleValue(f==="counterparty"?(tx.counterparty||""):f==="product"?(tx.product||""):(tx.transaction_type||"")); setRuleType("exact"); setRuleTagId(tags[0]?.id||""); setRuleDialog(true);
  };
  const handleAddRule = async () => {
    if(!ruleTx||!ruleTagId||!ruleValue)return;
    try { await createRule({id:"",match_field:ruleField,match_type:ruleType,match_value:ruleValue,target_tag_id:ruleTagId,priority:100,enabled:1,source:"user",created_at:"",updated_at:""}); setRuleDialog(false);setError(null); } catch(e){setError(String(e))}
  };

  const allSelected = transactions.length>0 && selectedIds.size===transactions.length;

  return (
    <Box sx={{ height:"calc(100vh - 120px)", display:"flex", flexDirection:"column" }}>
      <Typography variant="h4" gutterBottom>人工复核</Typography>

      {error && <Alert severity="error" sx={{ mb:1 }} onClose={()=>setError(null)}>{error}</Alert>}

      {/* 顶部操作栏 + 批量操作（合并） */}
      <Paper variant="outlined" sx={{ p:1.5, mb:2, position:"sticky", top:0, zIndex:10, bgcolor:selectedIds.size>0?"#e3f2fd":"white" }}>
        <Stack direction="row" spacing={1.5} alignItems="center" flexWrap="wrap">
          <Button variant="contained" color="primary" size="small" startIcon={classifying?<CircularProgress size={14} color="inherit"/>:<AutoModeIcon/>} onClick={handleClassify} disabled={classifying||loading}>① 自动分类</Button>
          <Button variant="contained" color="secondary" size="small" startIcon={aiClassifying?<CircularProgress size={14} color="inherit"/>:<PsychologyIcon/>} onClick={handleAiClassify} disabled={aiClassifying||loading||transactions.length===0}>② AI 分类</Button>
          <Button variant="outlined" size="small" startIcon={<RefreshIcon/>} onClick={loadData} disabled={loading||classifying||aiClassifying}>刷新</Button>
          <Typography variant="body2" color="text.secondary">待复核：{transactions.length} 条</Typography>
          {selectedIds.size > 0 && (<>
            <Typography variant="body2" sx={{ml:1}}>已选 <strong>{selectedIds.size}</strong></Typography>
            <FormControl size="small" sx={{minWidth:120}}><InputLabel>批量标签</InputLabel><Select value={batchTagId} label="批量标签" onChange={e=>setBatchTagId(e.target.value)}>{tags.map(t=><MenuItem key={t.id} value={t.id}>{t.name}</MenuItem>)}</Select></FormControl>
            <Button variant="contained" size="small" color="success" onClick={handleBatchConfirm} disabled={actionLoadingId==="__batch__"||!batchTagId}>确认</Button>
            <Button variant="contained" size="small" color="secondary" onClick={handleBatchAcceptAi} disabled={actionLoadingId==="__batch__"}>批量采纳</Button>
          </>)}
        </Stack>
      </Paper>

      {/* 结果 */}
      {(classifyResult||aiResultCount!==null) && (
        <Card sx={{mb:1}}><CardContent><Stack direction="row" spacing={3}>
          {classifyResult && <Typography variant="body2">规则引擎：<strong>{classifyResult.classified}</strong> 已分类 / <strong>{classifyResult.unclassified}</strong> 未匹配</Typography>}
          {aiResultCount!==null && <Typography variant="body2" color="secondary.main">AI 建议：<strong>{aiResultCount}</strong> 条</Typography>}
        </Stack></CardContent></Card>
      )}

      {loading && <Box sx={{display:"flex",justifyContent:"center",py:4}}><CircularProgress/></Box>}
      {!loading && transactions.length===0 && <Typography color="text.secondary" sx={{py:2}}>暂无待复核交易。</Typography>}

      {!loading && transactions.length>0 && (
        <Box sx={{overflow:"auto",flex:1,minHeight:0}}>
          <TableContainer component={Paper} variant="outlined">
            <Table size="small">
              <TableHead><TableRow>
                <TableCell padding="checkbox"><Checkbox size="small" checked={allSelected} indeterminate={selectedIds.size>0&&!allSelected} onChange={handleToggleSelectAll}/></TableCell>
                <TableCell>#</TableCell><TableCell>交易时间</TableCell><TableCell>来源</TableCell><TableCell>交易类型</TableCell>
                <TableCell>交易对方</TableCell><TableCell>商品</TableCell><TableCell>收/支</TableCell><TableCell align="right">金额</TableCell>
                <TableCell sx={{minWidth:160}}>消费标签</TableCell><TableCell align="center">操作</TableCell>
              </TableRow></TableHead>
              <TableBody>
                {transactions.map((tx,i)=>{
                  const hasAi=!!(tx.ai_suggested_tag&&tx.ai_confidence);
                  const aiName=tx.ai_suggested_tag?tagMap.get(tx.ai_suggested_tag):null;
                  return (<TableRow key={tx.id} selected={selectedIds.has(tx.id)} hover>
                    <TableCell padding="checkbox"><Checkbox size="small" checked={selectedIds.has(tx.id)} onChange={()=>handleToggleSelect(tx.id)}/></TableCell>
                    <TableCell>{i+1}</TableCell><TableCell>{tx.transaction_time}</TableCell>
                    <TableCell><Chip label={sl(tx.source)} color={sc(tx.source)} size="small"/></TableCell>
                    <TableCell>{tx.transaction_type||"-"}</TableCell><TableCell>{tx.counterparty||"-"}</TableCell>
                    <TableCell><Tooltip title={tx.product||""}><span>{tx.product&&tx.product.length>20?tx.product.substring(0,20)+"...":tx.product||"-"}</span></Tooltip></TableCell>
                    <TableCell><Chip label={dl(tx.direction)} color={dc(tx.direction)} size="small" variant="outlined"/></TableCell>
                    <TableCell align="right">{tx.amount.toFixed(2)}</TableCell>
                    <TableCell>
                      {hasAi?(
                        <Stack direction="row" spacing={0.5} alignItems="center">
                          <Select size="small" sx={{flex:1,minWidth:0}} value={rowTagSelections[tx.id]||""} onChange={e=>setRowTagSelections(p=>({...p,[tx.id]:e.target.value}))} displayEmpty>
                            <MenuItem value=""><em>选择</em></MenuItem>{tags.map(t=><MenuItem key={t.id} value={t.id}>{t.name}</MenuItem>)}
                          </Select>
                          <Chip label={`${aiName} ${((tx.ai_confidence||0)*100).toFixed(0)}%`} size="small" color="secondary" variant="outlined" sx={{flexShrink:0,maxWidth:120}}/>
                        </Stack>
                      ):(
                        <Select size="small" fullWidth value={rowTagSelections[tx.id]||""} onChange={e=>setRowTagSelections(p=>({...p,[tx.id]:e.target.value}))} displayEmpty>
                          <MenuItem value=""><em>选择标签</em></MenuItem>{tags.map(t=><MenuItem key={t.id} value={t.id}>{t.name}</MenuItem>)}
                        </Select>
                      )}
                    </TableCell>
                    <TableCell align="center">
                      {hasAi?(
                        <Stack direction="row" spacing={0.5} justifyContent="center">
                          <Button size="small" variant="contained" color="secondary" disabled={actionLoadingId===tx.id} onClick={()=>handleConfirmAi(tx)}>采纳</Button>
                          <Button size="small" variant="outlined" color="warning" disabled={!rowTagSelections[tx.id]||actionLoadingId===tx.id} onClick={()=>handleCorrectAi(tx)}>修改</Button>
                          <Button size="small" variant="outlined" disabled={actionLoadingId===tx.id} onClick={()=>handleSkip(tx.id)}>跳过</Button>
                          <Button size="small" variant="outlined" color="info" disabled={actionLoadingId===tx.id} onClick={()=>openRuleDialog(tx)}>加规则</Button>
                        </Stack>
                      ):(
                        <Stack direction="row" spacing={0.5} justifyContent="center">
                          <Button size="small" variant="contained" color="success" disabled={!rowTagSelections[tx.id]||actionLoadingId===tx.id} onClick={()=>handleConfirmTag(tx.id)}>确认</Button>
                          <Button size="small" variant="outlined" disabled={actionLoadingId===tx.id} onClick={()=>handleSkip(tx.id)}>跳过</Button>
                          <Button size="small" variant="outlined" color="info" disabled={actionLoadingId===tx.id} onClick={()=>openRuleDialog(tx)}>加规则</Button>
                        </Stack>
                      )}
                    </TableCell>
                  </TableRow>);
                })}
              </TableBody>
            </Table>
          </TableContainer>
        </Box>
      )}

      {/* 已跳过 */}
      <Box sx={{mt:3}}>
        <Stack direction="row" spacing={1} alignItems="center">
          <Button variant="text" size="small" onClick={async()=>{if(!showSkipped)await loadSkipped();setShowSkipped(!showSkipped)}}>{showSkipped?"隐藏":"查看"}已跳过</Button>
          <Chip label={skipped.length} size="small" variant="outlined"/>
        </Stack>
        {showSkipped && (skipped.length===0?<Typography variant="body2" color="text.secondary" sx={{mt:1}}>暂无跳过记录</Typography>:
          <TableContainer component={Paper} variant="outlined" sx={{mt:1}}><Table size="small">
            <TableHead><TableRow><TableCell>#</TableCell><TableCell>交易时间</TableCell><TableCell>来源</TableCell><TableCell>交易对方</TableCell><TableCell>商品</TableCell><TableCell align="right">金额</TableCell><TableCell align="center">操作</TableCell></TableRow></TableHead>
            <TableBody>{skipped.map((tx,i)=>(<TableRow key={tx.id} sx={{opacity:.7}}>
              <TableCell>{i+1}</TableCell><TableCell>{tx.transaction_time}</TableCell><TableCell><Chip label={sl(tx.source)} size="small" color={sc(tx.source)} variant="outlined"/></TableCell>
              <TableCell>{tx.counterparty||"-"}</TableCell><TableCell><Tooltip title={tx.product||""}><span>{tx.product&&tx.product.length>20?tx.product.substring(0,20)+"...":tx.product||"-"}</span></Tooltip></TableCell>
              <TableCell align="right">{tx.amount.toFixed(2)}</TableCell>
              <TableCell align="center"><Button size="small" variant="outlined" color="primary" disabled={actionLoadingId===tx.id} onClick={()=>handleUnskip(tx.id)}>恢复</Button></TableCell>
            </TableRow>))}</TableBody>
          </Table></TableContainer>
        )}
      </Box>

      {/* 添加规则弹窗 */}
      <Dialog open={ruleDialog} onClose={()=>setRuleDialog(false)} maxWidth="xs" fullWidth>
        <DialogTitle>从此交易添加规则</DialogTitle>
        <DialogContent><Stack spacing={1.5} sx={{mt:1}}>
          <FormControl fullWidth size="small"><InputLabel>匹配字段</InputLabel><Select value={ruleField} label="匹配字段" onChange={e=>{const f=e.target.value;setRuleField(f);if(ruleTx)setRuleValue(f==="counterparty"?(ruleTx.counterparty||""):f==="product"?(ruleTx.product||""):(ruleTx.transaction_type||""))}}>
            {ruleTx?.counterparty&&<MenuItem value="counterparty">交易对方</MenuItem>}{ruleTx?.product&&<MenuItem value="product">商品</MenuItem>}{ruleTx?.transaction_type&&<MenuItem value="transaction_type">交易类型</MenuItem>}
          </Select></FormControl>
          <TextField fullWidth size="small" label="匹配值" value={ruleValue} onChange={e=>setRuleValue(e.target.value)}/>
          <FormControl fullWidth size="small"><InputLabel>匹配方式</InputLabel><Select value={ruleType} label="匹配方式" onChange={e=>setRuleType(e.target.value)}><MenuItem value="exact">精确匹配</MenuItem><MenuItem value="like">包含匹配（模糊）</MenuItem></Select></FormControl>
          <FormControl fullWidth size="small"><InputLabel>目标标签</InputLabel><Select value={ruleTagId} label="目标标签" onChange={e=>setRuleTagId(e.target.value)}>{tags.map(t=><MenuItem key={t.id} value={t.id}>{t.name}</MenuItem>)}</Select></FormControl>
        </Stack></DialogContent>
        <DialogActions><Button onClick={()=>setRuleDialog(false)}>取消</Button><Button variant="contained" onClick={handleAddRule} disabled={!ruleTagId||!ruleValue}>添加规则</Button></DialogActions>
      </Dialog>
    </Box>
  );
}

export default ReviewPage;
