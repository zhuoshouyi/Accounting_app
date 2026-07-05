import { Box } from "@mui/material";
import Sidebar from "./Sidebar";

/**
 * 整体布局组件
 * 左侧固定宽度侧边栏（240px）+ 右侧主内容区域
 */
interface LayoutProps {
  children: React.ReactNode;
}

function Layout({ children }: LayoutProps) {
  return (
    <Box sx={{ display: "flex", minHeight: "100vh" }}>
      <Sidebar />
      <Box
        component="main"
        sx={{
          flexGrow: 1,
          p: 3,
          bgcolor: "background.default",
          overflow: "auto",
        }}
      >
        {children}
      </Box>
    </Box>
  );
}

export default Layout;
