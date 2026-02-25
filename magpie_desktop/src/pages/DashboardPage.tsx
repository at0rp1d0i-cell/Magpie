import { useState, useEffect } from "react";
import { motion } from "framer-motion";

// Mock data — will be replaced by Tauri IPC `get_latest_tickets()`
const MOCK_TICKETS = [
  { vehicle_code: "MU5131", vehicle_type: "flight", price_info: "￥320", start_time: "07:20", arrive_time: "09:35", duration: "2h15m", from_station_name: "HGH", to_station_name: "BJS", booking_status: "Y" },
  { vehicle_code: "G7", vehicle_type: "train", price_info: "二等座:￥610|一等座:￥1010", start_time: "06:50", arrive_time: "11:39", duration: "4h49m", from_station_name: "HZH", to_station_name: "BJP", booking_status: "Y" },
  { vehicle_code: "HU7578", vehicle_type: "flight", price_info: "￥330", start_time: "07:30", arrive_time: "09:55", duration: "2h25m", from_station_name: "HGH", to_station_name: "BJS", booking_status: "Y" },
  { vehicle_code: "G39", vehicle_type: "train", price_info: "二等座:￥610|一等座:￥1010", start_time: "08:15", arrive_time: "12:57", duration: "4h42m", from_station_name: "HZH", to_station_name: "BJP", booking_status: "Y" },
  { vehicle_code: "CZ8856", vehicle_type: "flight", price_info: "￥400", start_time: "11:50", arrive_time: "14:15", duration: "2h25m", from_station_name: "HGH", to_station_name: "BJS", booking_status: "Y" },
];

function extractPrice(priceInfo: string): number {
  const match = priceInfo.match(/￥(\d+)/);
  return match ? parseInt(match[1]) : 0;
}

export default function DashboardPage() {
  const [pulse, setPulse] = useState(false);

  useEffect(() => {
    const iv = setInterval(() => setPulse((p) => !p), 2000);
    return () => clearInterval(iv);
  }, []);

  const flights = MOCK_TICKETS.filter((t) => t.vehicle_type === "flight");
  const trains = MOCK_TICKETS.filter((t) => t.vehicle_type === "train");
  const cheapest = [...MOCK_TICKETS].sort((a, b) => extractPrice(a.price_info) - extractPrice(b.price_info))[0];

  return (
    <div className="h-full overflow-y-auto">
      {/* Header */}
      <header className="flex items-center justify-between border-b border-white/[0.06] px-8 py-5">
        <div>
          <h1 className="text-lg font-bold tracking-tight text-white/90">实时监控仪表盘</h1>
          <p className="text-xs text-white/40">{new Date().toLocaleDateString("zh-CN")} · 后台 daemon 运行中</p>
        </div>
        <button className="flex items-center gap-2 rounded-xl border border-white/10 bg-white/[0.04] px-4 py-2 text-xs font-medium text-white/60 transition-all hover:border-violet-500/40 hover:bg-white/[0.08] hover:text-white/90">
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" fill="currentColor" className="h-3.5 w-3.5"><path fillRule="evenodd" d="M13.836 2.477a.75.75 0 0 1 .75.75v3.182a.75.75 0 0 1-.75.75h-3.182a.75.75 0 0 1 0-1.5h1.37l-.84-.841a4.5 4.5 0 0 0-7.08.932.75.75 0 0 1-1.3-.75 6 6 0 0 1 9.44-1.242l.842.84V3.227a.75.75 0 0 1 .75-.75Zm-.911 7.5A.75.75 0 0 1 13.199 11a6 6 0 0 1-9.44 1.241l-.84-.84v1.371a.75.75 0 0 1-1.5 0V9.591a.75.75 0 0 1 .75-.75H5.35a.75.75 0 0 1 0 1.5H3.98l.841.841a4.5 4.5 0 0 0 7.08-.932.75.75 0 0 1 1.025-.273Z" clipRule="evenodd" /></svg>
          手动抓取
        </button>
      </header>

      <div className="space-y-6 p-8">
        {/* Stats cards */}
        <div className="grid grid-cols-3 gap-4">
          {[
            { label: "当前最低价", value: `￥${extractPrice(cheapest.price_info)}`, sub: cheapest.vehicle_code, color: "text-emerald-400" },
            { label: "监控航班", value: flights.length.toString(), sub: "航线", color: "text-sky-400" },
            { label: "监控高铁", value: trains.length.toString(), sub: "车次", color: "text-amber-400" },
          ].map((stat) => (
            <motion.div
              key={stat.label}
              whileHover={{ scale: 1.02 }}
              className="rounded-2xl border border-white/[0.08] bg-white/[0.03] p-5 backdrop-blur-md"
            >
              <p className="text-[11px] font-medium uppercase tracking-widest text-white/35">{stat.label}</p>
              <p className={`mt-1 font-mono text-2xl font-bold ${stat.color}`}>{stat.value}</p>
              <p className="mt-0.5 text-xs text-white/30">{stat.sub}</p>
            </motion.div>
          ))}
        </div>

        {/* Ticket matrix */}
        <div>
          <h2 className="mb-3 text-[11px] font-semibold uppercase tracking-widest text-white/35">
            Price Matrix · 全网快照
          </h2>
          <div className="space-y-2">
            {MOCK_TICKETS.map((t, i) => (
              <motion.div
                key={t.vehicle_code + i}
                initial={{ opacity: 0, x: -12 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ delay: i * 0.06 }}
                className="flex items-center justify-between rounded-xl border border-white/[0.06] bg-white/[0.03] px-5 py-3 transition-colors hover:bg-white/[0.07]"
              >
                <div className="flex items-center gap-4">
                  <span className="flex h-9 w-9 items-center justify-center rounded-lg bg-white/[0.06] text-base">
                    {t.vehicle_type === "flight" ? "✈️" : "🚄"}
                  </span>
                  <div>
                    <p className="font-mono text-sm font-semibold text-white/90">{t.vehicle_code}</p>
                    <p className="text-[11px] text-white/30">
                      {t.from_station_name} → {t.to_station_name} · {t.duration}
                    </p>
                  </div>
                </div>
                <div className="flex items-center gap-6">
                  <div className="text-right">
                    <p className="text-xs text-white/40">{t.start_time} → {t.arrive_time}</p>
                  </div>
                  <div className="min-w-[80px] text-right">
                    <p className="font-mono text-sm font-bold text-white/90">{t.price_info.split("|")[0]}</p>
                  </div>
                </div>
              </motion.div>
            ))}
          </div>
        </div>

        {/* Daemon heartbeat */}
        <div className="rounded-2xl border border-white/[0.08] bg-white/[0.03] p-5">
          <div className="flex items-center gap-3">
            <span className="relative flex h-3 w-3">
              <span className={`absolute inline-flex h-full w-full rounded-full opacity-75 ${pulse ? "animate-ping bg-emerald-400" : "bg-emerald-500"}`} />
              <span className="relative inline-flex h-3 w-3 rounded-full bg-emerald-400" />
            </span>
            <div>
              <p className="text-sm font-medium text-white/80">Tokio Daemon 运行中</p>
              <p className="text-xs text-white/30">Strategy: Leisure · 每 3 小时抓取 · 上次: {new Date().toLocaleTimeString("zh-CN")}</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
