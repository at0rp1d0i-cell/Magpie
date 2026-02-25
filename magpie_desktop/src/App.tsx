import { useState, useEffect } from "react";

const MagpieDashboard = () => {
  // 模拟从 SQLite 获取的飞行/高铁价格矩阵
  const SNAPSHOTS = [
    { type: "flight", route: "HGH ✈ BJS", price: 310, trend: -45, timestamp: "2026-03-02 18:55" },
    { type: "train", route: "HZH 🚄 BJP", price: 610, trend: 0, timestamp: "2026-03-02 06:50" },
    { type: "flight", route: "HGH ✈ BJS", price: 420, trend: 15, timestamp: "2026-03-02 11:50" }
  ];

  // 模拟 DeepSeek 生成的决策建议
  const DECISION = {
    status: "recommend", // recommend, watch, wait
    confidence: 0.92,
    message: "探测到破价航班 JD5907，比同日高铁便宜约50%。由于您是休闲出游 (leisure)，建议立即出票锁定 310 元特价档位。"
  };

  const statusColor = {
    recommend: "bg-emerald-400 shadow-[0_0_15px_rgba(52,211,153,0.5)]",
    watch: "bg-amber-400 shadow-[0_0_15px_rgba(251,191,36,0.5)]",
    wait: "bg-rose-400 shadow-[0_0_15px_rgba(251,113,133,0.5)]",
  };

  const delta = (s: any) => s.trend;
  const pct = (s: any) => ((Math.abs(s.trend) / (s.price - s.trend)) * 100).toFixed(1);

  const [pulse, setPulse] = useState(false);
  useEffect(() => {
    const interval = setInterval(() => setPulse((p) => !p), 2000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="flex min-h-screen items-center justify-center bg-zinc-950 bg-[url('https://images.unsplash.com/photo-1557683316-973673baf926?q=80&w=2029')] bg-cover bg-center p-4">
      {/* ── 主容器：极光毛玻璃效 ────────── */}
      <div className="relative w-full max-w-md overflow-hidden rounded-3xl border border-white/20 bg-white/10 shadow-[0_8px_32px_rgba(0,0,0,0.37)] backdrop-blur-2xl backdrop-saturate-150">
        
        {/* 背景光晕点缀 */}
        <div className="absolute -left-10 -top-10 h-40 w-40 rounded-full bg-violet-500/30 blur-3xl" />
        <div className="absolute -bottom-10 -right-10 h-40 w-40 rounded-full bg-fuchsia-500/20 blur-3xl" />

        {/* ── Header (Phase 2 Rust Tokio 心跳映射) ────────── */}
        <div className="relative z-10 flex items-center justify-between border-b border-white/10 px-6 py-5">
          <div>
            <h1 className="text-lg font-bold tracking-tight text-white/90">
              Magpie Omni-Tracker
            </h1>
            <p className="text-xs font-medium tracking-wider text-white/50">
              {new Date().toISOString().split("T")[0]} · SYSTEM ACTIVE
            </p>
          </div>
          
          <div className="flex items-center gap-2 rounded-full border border-white/10 bg-black/20 px-3 py-1.5 backdrop-blur-md">
            <span className="text-[10px] font-bold uppercase tracking-widest text-white/70">
              Tokio
            </span>
            <span className="relative flex h-2.5 w-2.5">
              <span
                className={[
                  "absolute inline-flex h-full w-full rounded-full opacity-75 transition-all duration-1000",
                  pulse ? "animate-ping bg-emerald-400" : "bg-emerald-500",
                ].join(" ")}
              />
              <span className="relative inline-flex h-2.5 w-2.5 rounded-full bg-emerald-400" />
            </span>
          </div>
        </div>

        {/* ── Price Matrix (Phase 2 SQLite snapshots) ─────────── */}
        <div className="relative z-10 space-y-2 px-6 py-4">
          <h2 className="mb-1 text-[11px] font-semibold uppercase tracking-widest text-white/40">
            Price Matrix · 实时快照
          </h2>

          {SNAPSHOTS.map((s) => {
            const d = delta(s);
            const falling = d < 0;
            return (
              <div
                key={s.route + s.timestamp}
                className="flex items-center justify-between rounded-xl border border-white/10 bg-white/5 px-4 py-2.5 transition-colors hover:bg-white/10"
              >
                <div className="flex items-center gap-3">
                  <span className="text-base">
                    {s.type === "flight" ? "✈️" : "🚄"}
                  </span>
                  <div>
                    <p className="text-sm font-medium text-white">{s.route}</p>
                    <p className="text-[10px] text-white/30">{s.timestamp}</p>
                  </div>
                </div>

                <div className="text-right">
                  <p className="font-mono text-sm font-semibold text-white">
                    ¥{s.price}
                  </p>
                  <p
                    className={[
                      "font-mono text-[11px] font-medium",
                      falling ? "text-emerald-400" : d > 0 ? "text-rose-400" : "text-white/30",
                    ].join(" ")}
                  >
                    {falling ? "↓" : d > 0 ? "↑" : "—"} {Math.abs(d)} ({pct(s)}%)
                  </p>
                </div>
              </div>
            );
          })}
        </div>

        {/* ── Agent Decision (Phase 3 DeepSeek verdict) ───────── */}
        <div className="relative z-10 mx-6 mb-6 rounded-2xl border border-white/15 bg-white/5 p-4 backdrop-blur-md">
          <div className="mb-2 flex items-center gap-2">
            <span className={`h-2 w-2 rounded-full ${(statusColor as any)[DECISION.status]}`} />
            <span className="text-[11px] font-semibold uppercase tracking-widest text-white/50">
              DeepSeek 决策 · Confidence {(DECISION.confidence * 100).toFixed(0)}%
            </span>
          </div>

          <p className="text-[13px] leading-relaxed text-white/80">
            {DECISION.message}
          </p>

          {/* confidence bar */}
          <div className="mt-3 h-1 w-full overflow-hidden rounded-full bg-white/10">
            <div
              className="h-full rounded-full bg-gradient-to-r from-violet-400 to-fuchsia-400 transition-all duration-700"
              style={{ width: `${DECISION.confidence * 100}%` }}
            />
          </div>
        </div>
      </div>
    </div>
  );
};

export default MagpieDashboard;
