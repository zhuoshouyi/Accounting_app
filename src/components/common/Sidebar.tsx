import { useState } from "react";
import { useNavigate, useLocation } from "react-router-dom";
import {
  Drawer,
  List,
  ListItem,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Toolbar,
  Typography,
  Box,
  Divider,
} from "@mui/material";
import UploadFileIcon from "@mui/icons-material/UploadFile";
import CleaningServicesIcon from "@mui/icons-material/CleaningServices";
import ListAltIcon from "@mui/icons-material/ListAlt";
import RateReviewIcon from "@mui/icons-material/RateReview";
import SummarizeIcon from "@mui/icons-material/Summarize";
import BarChartIcon from "@mui/icons-material/BarChart";
import RuleIcon from "@mui/icons-material/Rule";
import SettingsIcon from "@mui/icons-material/Settings";

/** 侧边栏宽度 */
const DRAWER_WIDTH = 240;

/** 导航菜单项定义 */
interface MenuItem {
  label: string;
  path: string;
  icon: React.ReactNode;
}

const menuItems: MenuItem[] = [
  { label: "导入账单", path: "/import", icon: <UploadFileIcon /> },
  { label: "数据清洗", path: "/cleaning", icon: <CleaningServicesIcon /> },
  { label: "交易明细", path: "/transactions", icon: <ListAltIcon /> },
  { label: "人工复核", path: "/review", icon: <RateReviewIcon /> },
  { label: "月度汇总", path: "/summary", icon: <SummarizeIcon /> },
  { label: "报表中心", path: "/reports", icon: <BarChartIcon /> },
  { label: "规则管理", path: "/rules", icon: <RuleIcon /> },
  { label: "设置", path: "/settings", icon: <SettingsIcon /> },
];

/**
 * 侧边导航组件
 * 固定左侧，高亮当前路由对应的菜单项
 */
function Sidebar() {
  const navigate = useNavigate();
  const location = useLocation();

  const selectedIndex = menuItems.findIndex(
    (item) => location.pathname === item.path
  );
  const [activeIndex, setActiveIndex] = useState(
    selectedIndex >= 0 ? selectedIndex : 0
  );

  const handleItemClick = (index: number, path: string) => {
    setActiveIndex(index);
    navigate(path);
  };

  return (
    <Drawer
      variant="permanent"
      sx={{
        width: DRAWER_WIDTH,
        flexShrink: 0,
        "& .MuiDrawer-paper": {
          width: DRAWER_WIDTH,
          boxSizing: "border-box",
        },
      }}
    >
      <Toolbar>
        <Box sx={{ display: "flex", alignItems: "center", gap: 1 }}>
          <Typography variant="h6" component="div" sx={{ fontWeight: "bold" }}>
            家庭记账
          </Typography>
        </Box>
      </Toolbar>
      <Divider />
      <List>
        {menuItems.map((item, index) => (
          <ListItem key={item.path} disablePadding>
            <ListItemButton
              selected={activeIndex === index}
              onClick={() => handleItemClick(index, item.path)}
            >
              <ListItemIcon>{item.icon}</ListItemIcon>
              <ListItemText primary={item.label} />
            </ListItemButton>
          </ListItem>
        ))}
      </List>
    </Drawer>
  );
}

export default Sidebar;
