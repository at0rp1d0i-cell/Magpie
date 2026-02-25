import { BrowserRouter, Routes, Route } from "react-router-dom";
import Sidebar from "./components/Sidebar";
import ChatPage from "./pages/ChatPage";
import DashboardPage from "./pages/DashboardPage";
import PlansPage from "./pages/PlansPage";
import AlertsPage from "./pages/AlertsPage";
import SettingsPage from "./pages/SettingsPage";

export default function App() {
  return (
    <BrowserRouter>
      {/* Light slate background canvas */}
      <div className="flex h-screen w-screen overflow-hidden bg-slate-100 text-zinc-900">
        
        {/* Sidebar wrapper with padding for floating effect */}
        <div className="relative z-10 py-4 pl-4 pr-2">
          <Sidebar />
        </div>

        {/* Main Content wrapper */}
        <main className="flex-1 overflow-hidden py-4 pr-4 pl-2">
          {/* Main content floating container */}
          <div className="h-full w-full overflow-hidden rounded-[24px] bg-white shadow-glass ring-1 ring-zinc-200/50">
            <Routes>
              <Route path="/" element={<ChatPage />} />
              <Route path="/dashboard" element={<DashboardPage />} />
              <Route path="/plans" element={<PlansPage />} />
              <Route path="/alerts" element={<AlertsPage />} />
              <Route path="/settings" element={<SettingsPage />} />
            </Routes>
          </div>
        </main>

      </div>
    </BrowserRouter>
  );
}
