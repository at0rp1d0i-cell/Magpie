import { motion } from "framer-motion";
import { CheckCircle2, Eye, ShieldAlert } from "lucide-react";

const MOCK_ALERTS = [
  {
    id: "1",
    timestamp: "2026-02-25 15:30",
    confidence: 0.92,
    status: "recommend" as const,
    message: "探测到破价航班 MU5131，杭州→北京仅 ￥320，比同日高铁便宜约47%。由于您是休闲出游 (leisure)，建议立即出票锁定特价档位。",
  },
  {
    id: "2",
    timestamp: "2026-02-25 12:00",
    confidence: 0.65,
    status: "watch" as const,
    message: "当前高铁价格平稳 (￥610)，飞机票价格处于中位。建议持续观望，等待更低价航班释放。",
  },
  {
    id: "3",
    timestamp: "2026-02-24 18:45",
    confidence: 0.45,
    status: "wait" as const,
    message: "全网扫描均显示高价态势。北京目的地周末天气预报显示有小雪，可考虑推迟出行。",
  },
];

const STATUS_CONFIG = {
  recommend: { label: "推荐出手", icon: CheckCircle2, color: "text-emerald-600", dot: "bg-emerald-500", border: "border-emerald-200", bg: "bg-emerald-50" },
  watch:     { label: "持续观望", icon: Eye,          color: "text-amber-600",   dot: "bg-amber-500",   border: "border-amber-200",   bg: "bg-amber-50" },
  wait:      { label: "暂缓出行", icon: ShieldAlert,  color: "text-rose-600",    dot: "bg-rose-500",    border: "border-rose-200",    bg: "bg-rose-50" },
};

export default function AlertsPage() {
  return (
    <div className="h-full overflow-y-auto bg-white">
      <header className="border-b border-zinc-100 px-8 py-5">
        <h1 className="text-lg font-bold tracking-tight text-zinc-900">通知中心</h1>
        <p className="mt-1 text-xs text-zinc-500">AI 决策推送历史</p>
      </header>

      <div className="space-y-4 p-8">
        {MOCK_ALERTS.map((alert, i) => {
          const cfg = STATUS_CONFIG[alert.status];
          return (
            <motion.div
              key={alert.id}
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: i * 0.08 }}
              className={`rounded-2xl border ${cfg.border} ${cfg.bg} p-6 shadow-sm`}
            >
              <div className="mb-3.5 flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <cfg.icon className={`h-4 w-4 ${cfg.color}`} strokeWidth={2.5} />
                  <span className={`text-[11px] font-bold uppercase tracking-widest ${cfg.color}`}>{cfg.label}</span>
                  <span className="text-[11px] font-medium text-zinc-400">· 决策置信度 {(alert.confidence * 100).toFixed(0)}%</span>
                </div>
                <span className="text-[11px] font-medium text-zinc-400">{alert.timestamp}</span>
              </div>
              <p className="text-[14px] leading-relaxed text-zinc-700">{alert.message}</p>
              
              {/* Confidence bar */}
              <div className="mt-4 h-1.5 w-full overflow-hidden rounded-full bg-black/[0.04]">
                <div
                  className="h-full rounded-full bg-zinc-800 transition-all duration-700"
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
