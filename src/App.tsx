import { Routes, Route, Navigate } from "react-router-dom";
import Layout from "./components/common/Layout";
import ImportPage from "./pages/ImportPage";
import CleaningPage from "./pages/CleaningPage";
import TransactionsPage from "./pages/TransactionsPage";
import ReviewPage from "./pages/ReviewPage";
import SummaryPage from "./pages/SummaryPage";
import ReportsPage from "./pages/ReportsPage";
import RulesPage from "./pages/RulesPage";
import SettingsPage from "./pages/SettingsPage";

/**
 * 根组件 — 路由配置
 * 所有页面共享 Layout（侧边栏 + 主区域）
 */
function App() {
  return (
    <Layout>
      <Routes>
        <Route path="/" element={<Navigate to="/import" replace />} />
        <Route path="/import" element={<ImportPage />} />
        <Route path="/cleaning" element={<CleaningPage />} />
        <Route path="/transactions" element={<TransactionsPage />} />
        <Route path="/review" element={<ReviewPage />} />
        <Route path="/summary" element={<SummaryPage />} />
        <Route path="/reports" element={<ReportsPage />} />
        <Route path="/rules" element={<RulesPage />} />
        <Route path="/owners" element={<Navigate to="/rules" replace />} />
        <Route path="/settings" element={<SettingsPage />} />
      </Routes>
    </Layout>
  );
}

export default App;
