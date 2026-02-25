import { motion } from "framer-motion";
import { Coffee, Plus, Briefcase, Navigation, Radar } from "lucide-react";

export default function PlansPage() {
  const plan = {
    persona: "leisure",
    departure: { city: "杭州", train_code: "HZH", flight_code: "HGH" },
    destination: { city: "北京", train_code: "BJP", flight_code: "BJS" },
    time_window: "2026-03-01 ~ 2026-03-05",
    budget_cap: 1000,
    status: "active",
  };

  return (
    <div className="h-full overflow-y-auto bg-white">
      <header className="border-b border-zinc-100 px-8 py-5">
        <h1 className="text-lg font-bold tracking-tight text-zinc-900">出行计划</h1>
        <p className="mt-1 text-xs text-zinc-500">管理你的监控任务</p>
      </header>

      <div className="space-y-6 p-8">
        {/* Active plan */}
        <div>
          <h2 className="mb-4 text-[11px] font-semibold uppercase tracking-widest text-zinc-400">当前活跃计划</h2>
          <motion.div
            whileHover={{ scale: 1.01 }}
            className="relative overflow-hidden rounded-2xl border border-violet-100 bg-white p-6 shadow-sm ring-1 ring-zinc-200/50"
          >
            <div className="absolute -right-6 -top-6 h-24 w-24 rounded-full bg-violet-100/50 blur-2xl" />
            <div className="relative z-10">
              <div className="mb-5 flex items-center justify-between">
                <div className="flex items-center gap-4">
                  <span className="flex h-11 w-11 items-center justify-center rounded-[12px] bg-violet-50 text-violet-600 shadow-sm border border-violet-100">
                    <Navigation className="h-5 w-5" strokeWidth={2.5} />
                  </span>
                  <div>
                    <h3 className="text-base font-bold text-zinc-900">{plan.departure.city} 到 {plan.destination.city}</h3>
                    <p className="mt-0.5 text-xs font-medium text-zinc-500">{plan.time_window}</p>
                  </div>
                </div>
                <div className="flex items-center gap-1.5 rounded-full border border-emerald-200 bg-emerald-50 px-3 py-1.5 shadow-sm">
                  <Radar className="h-3 w-3 animate-spin text-emerald-600 [animation-duration:3s]" />
                  <span className="text-[10px] font-bold uppercase tracking-widest text-emerald-600">
                    Monitoring
                  </span>
                </div>
              </div>
              
              <div className="grid grid-cols-3 gap-4 border-t border-zinc-100 pt-5">
                {[
                  { label: "预算上限", value: `￥${plan.budget_cap}` },
                  { label: "出游画像", value: plan.persona === "leisure" ? "Leisure 休闲" : "Business 差旅", icon: plan.persona === "leisure" ? <Coffee className="h-4 w-4 text-zinc-400 mr-1.5" /> : <Briefcase className="h-4 w-4 text-zinc-400 mr-1.5" /> },
                  { label: "轮询策略", value: plan.persona === "leisure" ? "3h / 周期" : "60s / 周期" },
                ].map((item, i) => (
                  <div key={i} className="flex flex-col justify-center rounded-xl border border-zinc-100 bg-slate-50 px-4 py-3.5">
                    <p className="text-[10px] font-semibold uppercase tracking-widest text-zinc-400">{item.label}</p>
                    <div className="mt-1 flex items-center text-[13px] font-bold text-zinc-900">
                      {item.icon}
                      {item.value}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </motion.div>
        </div>

        {/* Create new plan hint */}
        <button className="group mt-8 flex w-full flex-col items-center justify-center gap-2 rounded-2xl border-2 border-dashed border-zinc-200 bg-slate-50 py-10 transition-colors hover:border-violet-300 hover:bg-violet-50 focus:outline-none">
          <div className="flex h-10 w-10 items-center justify-center rounded-full bg-zinc-100 text-zinc-400 transition-colors group-hover:bg-violet-100 group-hover:text-violet-600">
            <Plus className="h-5 w-5 stroke-[2.5px]" />
          </div>
          <span className="text-sm font-medium text-zinc-500 group-hover:text-violet-700">
            前往 Chat 页面，和 AI 对话创建新计划
          </span>
        </button>
      </div>
    </div>
  );
}
