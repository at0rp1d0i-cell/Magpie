import { useState, useEffect } from "react";
import { motion } from "framer-motion";
import { invoke } from "../utils/tauri";
import { RefreshCw, Plane, Train, Activity } from "lucide-react";

// Mock data — will be replaced by Tauri IPC `get_latest_tickets()`
const MOCK_TICKETS = [
  { vehicle_code: "MU5131", vehicle_type: "flight", price_info: "￥320", start_time: "07:20", arrive_time: "09:35", duration: "2h15m", from_station_name: "HGH", to_station_name: "BJS", booking_status: "Y" },
];

function extractPrice(priceInfo: string): number {
  const match = priceInfo.match(/￥(\d+)/);
  return match ? parseInt(match[1]) : 0;
}

export default function DashboardPage() {
  const [pulse, setPulse] = useState(false);
  const [tickets, setTickets] = useState<any[]>([]);
  const [daemonStatus, setDaemonStatus] = useState("checking...");

  useEffect(() => {
    const fetchRealData = async () => {
      try {
        const dbTickets = await invoke<any[]>("get_latest_tickets");
        if (dbTickets.length > 0) setTickets(dbTickets);
        
        const st = await invoke<string>("get_daemon_status");
        setDaemonStatus(st);
      } catch (e) {
        console.error("IPC fetch error:", e);
      }
    };
    
    // Initial fetch
    fetchRealData();

    // Pulse interval & polling
    const iv = setInterval(() => {
      setPulse((p) => !p);
      fetchRealData();
    }, 2000);
    return () => clearInterval(iv);
  }, []);

  const displayTickets = tickets.length > 0 ? tickets : MOCK_TICKETS;
  const flights = displayTickets.filter((t: any) => t.vehicle_type === "flight");
  const trains = displayTickets.filter((t: any) => t.vehicle_type === "train");
  const cheapest = [...displayTickets].sort((a: any, b: any) => extractPrice(a.price_info) - extractPrice(b.price_info))[0];

  return (
    <div className="h-full overflow-y-auto bg-white">
      {/* Header */}
      <header className="flex items-center justify-between border-b border-zinc-100 px-8 py-5">
        <div>
          <h1 className="text-lg font-bold tracking-tight text-zinc-900">实时监控大盘</h1>
          <p className="mt-1 text-xs text-zinc-500">{new Date().toLocaleDateString("zh-CN")} · 后台 daemon 运行中 ({daemonStatus})</p>
        </div>
        <button 
          onClick={() => setPulse((p) => !p)} 
          className="flex items-center gap-2 rounded-xl border border-zinc-200 bg-white px-4 py-2 text-xs font-medium text-zinc-600 shadow-sm transition-all hover:bg-slate-50 hover:text-zinc-900 active:scale-95"
        >
          <RefreshCw className="h-3.5 w-3.5 stroke-[2.5px]" />
          手动抓取
        </button>
      </header>
      <div className="space-y-6 p-8">
        {/* Stats cards */}
        <div className="grid grid-cols-3 gap-4">
          {[
            { label: "当前最低价", value: `￥${extractPrice(cheapest.price_info)}`, sub: cheapest.vehicle_code, color: "text-zinc-900" },
            { label: "监控航班", value: flights.length.toString(), sub: "条可用航线", color: "text-zinc-800" },
            { label: "监控高铁", value: trains.length.toString(), sub: "有效组次", color: "text-zinc-800" },
          ].map((stat) => (
            <motion.div
              key={stat.label}
              whileHover={{ scale: 1.01 }}
              className="rounded-2xl border border-zinc-200/60 bg-slate-50 p-6 shadow-sm"
            >
              <p className="text-[11px] font-semibold uppercase tracking-widest text-zinc-400">{stat.label}</p>
              <p className={`mt-2 font-mono text-3xl font-bold ${stat.color}`}>{stat.value}</p>
              <p className="mt-1 text-xs font-medium text-zinc-500">{stat.sub}</p>
            </motion.div>
          ))}
        </div>

        {/* Ticket matrix */}
        <div>
          <h2 className="mb-4 text-[11px] font-bold uppercase tracking-widest text-zinc-400">
            Price Matrix · 全网快照
          </h2>
          <div className="space-y-2.5">
            {displayTickets.slice(0, 8).map((t, i) => (
              <motion.div
                key={t.vehicle_code + i}
                initial={{ opacity: 0, x: -12 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ delay: i * 0.06 }}
                className="flex items-center justify-between rounded-xl border border-zinc-100 bg-white px-5 py-3.5 shadow-sm transition-colors hover:border-zinc-300"
              >
                <div className="flex items-center gap-4">
                  <span className="flex h-10 w-10 items-center justify-center rounded-[10px] bg-slate-100 text-zinc-600">
                    {t.vehicle_type === "flight" ? <Plane className="h-5 w-5" /> : <Train className="h-5 w-5" />}
                  </span>
                  <div>
                    <p className="font-mono text-sm font-bold text-zinc-900">{t.vehicle_code}</p>
                    <p className="mt-0.5 text-[11px] font-medium text-zinc-500">
                      {t.from_station_name} → {t.to_station_name} · {t.duration}
                    </p>
                  </div>
                </div>
                <div className="flex items-center gap-8">
                  <div className="text-right">
                    <p className="text-xs font-medium text-zinc-400">{t.start_time} <span className="text-zinc-300">→</span> {t.arrive_time}</p>
                  </div>
                  <div className="min-w-[80px] text-right">
                    <p className="font-mono text-[15px] font-black text-zinc-900">{t.price_info.split("|")[0]}</p>
                  </div>
                </div>
              </motion.div>
            ))}
          </div>
        </div>

        {/* Daemon heartbeat */}
        <div className="rounded-2xl border border-zinc-200/60 bg-slate-50 p-5 shadow-sm">
          <div className="flex items-center gap-3.5">
            <span className="relative flex h-3.5 w-3.5 items-center justify-center">
              <span className={`absolute inline-flex h-full w-full rounded-full opacity-75 ${pulse ? "animate-ping bg-emerald-400" : "bg-emerald-500"}`} />
              <span className="relative flex h-3.5 w-3.5 items-center justify-center rounded-full bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.5)]">
                 <Activity className="h-2 w-2 text-white" />
              </span>
            </span>
            <div>
              <p className="text-sm font-bold text-zinc-800">Tokio Native 线程巡视中</p>
              <p className="mt-0.5 text-xs font-medium text-zinc-500">Strategy: VIP Leisure · 智能间隙抓取 · 上次: {new Date().toLocaleTimeString("zh-CN")}</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
