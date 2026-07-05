import { useState } from "react";
import {
  Box, Button, Dialog, DialogTitle, DialogContent, Typography,
  IconButton, Divider,
} from "@mui/material";
import Sidebar from "./Sidebar";
import CloseIcon from "@mui/icons-material/Close";
import HelpIcon from "@mui/icons-material/HelpOutline";

function Layout({ children }: { children: React.ReactNode }) {
  const [helpOpen, setHelpOpen] = useState(false);

  return (
    <Box sx={{ display: "flex", minHeight: "100vh" }}>
      <Sidebar />
      <Box
        component="main"
        sx={{ flexGrow: 1, p: 3, bgcolor: "background.default", overflow: "auto" }}
      >
        {/* 右上角使用说明按钮 */}
        <Box sx={{ position: "fixed", top: 8, right: 16, zIndex: 100 }}>
          <Button size="small" variant="outlined" startIcon={<HelpIcon />}
            onClick={() => setHelpOpen(true)}>使用说明</Button>
        </Box>
        {children}
      </Box>

      {/* 使用说明弹窗 */}
      <Dialog open={helpOpen} onClose={() => setHelpOpen(false)} maxWidth="md" fullWidth>
        <DialogTitle sx={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
          使用说明
          <IconButton onClick={() => setHelpOpen(false)}><CloseIcon /></IconButton>
        </DialogTitle>
        <DialogContent dividers>
          <Typography variant="h6" gutterBottom>快速开始</Typography>

          <Typography variant="subtitle1" sx={{ mt: 1 }}>1. 配置归属人</Typography>
          <Typography variant="body2" color="text.secondary">规则管理 → 归属人 → 添加家庭成员（如 joey、vila）</Typography>

          <Typography variant="subtitle1" sx={{ mt: 1 }}>2. 导入账单</Typography>
          <Typography variant="body2" color="text.secondary">导入账单 → 选择微信 XLSX 或支付宝 CSV → 选归属人 → 确认</Typography>

          <Typography variant="subtitle1" sx={{ mt: 1 }}>3. 数据清洗</Typography>
          <Typography variant="body2" color="text.secondary">数据清洗 → 开始清洗 → 查看待过滤/修改 → 确认执行</Typography>

          <Typography variant="subtitle1" sx={{ mt: 1 }}>4. 分类</Typography>
          <Typography variant="body2" color="text.secondary">人工复核 → ① 自动分类 → ② AI 分类（需配置 API Key）→ 逐条采纳/修改/跳过</Typography>

          <Typography variant="subtitle1" sx={{ mt: 1 }}>5. 交易明细</Typography>
          <Typography variant="body2" color="text.secondary">双击编辑标签/归属人/刚需/商品/金额 | 选中多行批量操作 | 右键菜单标记/添加规则 | 支持导出 CSV</Typography>

          <Typography variant="subtitle1" sx={{ mt: 1 }}>6. 月度汇总</Typography>
          <Typography variant="body2" color="text.secondary">查看全部月份收支透视表 | 点击金额下钻明细 | 底部填写手动数据（支持子项拆分）</Typography>

          <Divider sx={{ my: 1.5 }} />

          <Typography variant="h6" gutterBottom>AI 功能（可选）</Typography>
          <Typography variant="body2" color="text.secondary">设置 → 填入 DeepSeek API Key → 启用 → 测试连接</Typography>
          <Typography variant="body2" color="text.secondary">人工复核：AI 辅助分类 | 报表中心：AI 分析 / HTML 报表 / 图表生成</Typography>

          <Typography variant="h6" sx={{ mt: 1.5 }} gutterBottom>规则管理</Typography>
          <Typography variant="body2" color="text.secondary">分类规则 / AI学习规则 / 标签管理 / 汇总映射 / 规则测试 / 归属人</Typography>

          <Typography variant="h6" sx={{ mt: 1.5 }} gutterBottom>数据库位置</Typography>
          <Typography variant="body2" color="text.secondary" sx={{ fontFamily: "monospace", fontSize: 12 }}>
            ~/Library/Application Support/family-accounting-app/database.db
          </Typography>
        </DialogContent>
      </Dialog>
    </Box>
  );
}

export default Layout;
