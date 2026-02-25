import { motion } from "framer-motion";

const MOCK_ALERTS = [
  {
    id: "1",
    timestamp: "2026-02-25 15:30",
    confidence: 0.92,
    status: "recommend" as const,
    message: "探测到破价航班 MU5131，杭州→北京仅 ￥320，比同日高铁便宜约47%。由于您是休闲出游 (leisure)，建议立即出票锁定特价档位。🎯",
  },
  {
    id: "2",
    timestamp: "2026-02-25 12:00",
    confidence: 0.65,
    status: "watch" as const,
    message: "当前高铁价格平稳 (￥610)，飞机票价格处于中位。建议持续观望，等待更低价航班释放。📡",
  },
  {
    id: "3",
    timestamp: "2026-02-24 18:45",
    confidence: 0.45,
    status: "wait" as const,
    message: "全网扫描均显示高价态势。北京目的地周末天气预报显示有小雪，可考虑推迟出行。❄️",
  },
];

const STATUS_CONFIG = {
  recommend: { label: "推荐出手", color: "bg-emerald-400", border: "border-emerald-500/20", bg: "bg-emerald-500/[0.06]" },
  watch:     { label: "持续观望", color: "bg-amber-400",   border: "border-amber-500/20",   bg: "bg-amber-500/[0.06]" },
  wait:      { label: "暂缓出行", color: "bg-rose-400",    border: "border-rose-500/20",    bg: "bg-rose-500/[0.06]" },
};

export default function AlertsPage() {
  return (
    <div className="h-full overflow-y-auto">
      <header className="border-b border-white/[0.06] px-8 py-5">
        <h1 className="text-lg font-bold tracking-tight text-white/90">通知中心</h1>
        <p className="text-xs text-white/40">AI 决策推送历史</p>
      </header>

      <div className="space-y-3 p-8">
        {MOCK_ALERTS.map((alert, i) => {
          const cfg = STATUS_CONFIG[alert.status];
          return (
            <motion.div
              key={alert.id}
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: i * 0.08 }}
              className={`rounded-2xl border ${cfg.border} ${cfg.bg} p-5 backdrop-blur-md`}
            >
              <div className="mb-2.5 flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <span className={`h-2 w-2 rounded-full ${cfg.color}`} />
                  <span className="text-[10px] font-bold uppercase tracking-widest text-white/50">{cfg.label}</span>
                  <span className="text-[10px] text-white/25">· Confidence {(alert.confidence * 100).toFixed(0)}%</span>
                </div>
                <span className="text-[11px] text-white/25">{alert.timestamp}</span>
              </div>
              <p className="text-[13px] leading-relaxed text-white/75">{alert.message}</p>
              {/* Confidence bar */}
              <div className="mt-3 h-1 w-full overflow-hidden rounded-full bg-white/[0.06]">
                <div
                  className="h-full rounded-full bg-gradient-to-r from-violet-400 to-fuchsia-400 transition-all duration-700"
                  style={{ width: `${alert.confidence * 100}%` }}
                />
              </div>
            </motion.div>
          );
        })}
      </div>
    </div>
  );
}
