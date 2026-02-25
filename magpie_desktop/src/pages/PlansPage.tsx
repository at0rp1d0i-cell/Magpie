import { motion } from "framer-motion";

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
    <div className="h-full overflow-y-auto">
      <header className="border-b border-white/[0.06] px-8 py-5">
        <h1 className="text-lg font-bold tracking-tight text-white/90">出行计划</h1>
        <p className="text-xs text-white/40">管理你的监控任务</p>
      </header>

      <div className="space-y-6 p-8">
        {/* Active plan */}
        <div>
          <h2 className="mb-3 text-[11px] font-semibold uppercase tracking-widest text-white/35">当前活跃计划</h2>
          <motion.div
            whileHover={{ scale: 1.01 }}
            className="relative overflow-hidden rounded-2xl border border-violet-500/20 bg-gradient-to-br from-violet-500/[0.08] to-fuchsia-500/[0.04] p-6 backdrop-blur-md"
          >
            <div className="absolute -right-6 -top-6 h-24 w-24 rounded-full bg-violet-500/10 blur-2xl" />
            <div className="relative z-10">
              <div className="mb-4 flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <span className="flex h-10 w-10 items-center justify-center rounded-xl bg-violet-500/20 text-xl">🐦</span>
                  <div>
                    <p className="font-semibold text-white/90">{plan.departure.city} → {plan.destination.city}</p>
                    <p className="text-xs text-white/40">{plan.time_window}</p>
                  </div>
                </div>
                <span className="rounded-full bg-emerald-500/20 px-3 py-1 text-[10px] font-bold uppercase tracking-widest text-emerald-400">
                  Monitoring
                </span>
              </div>
              <div className="grid grid-cols-3 gap-4">
                {[
                  { label: "预算上限", value: `￥${plan.budget_cap}` },
                  { label: "画像", value: plan.persona === "leisure" ? "🍹 休闲" : "🧑‍💼 商务" },
                  { label: "监控模式", value: plan.persona === "leisure" ? "3h/次" : "60s/次" },
                ].map((item) => (
                  <div key={item.label} className="rounded-xl bg-black/20 px-4 py-3">
                    <p className="text-[10px] font-medium uppercase tracking-widest text-white/30">{item.label}</p>
                    <p className="mt-1 text-sm font-semibold text-white/80">{item.value}</p>
                  </div>
                ))}
              </div>
            </div>
          </motion.div>
        </div>

        {/* Create new plan hint */}
        <button className="flex w-full items-center justify-center gap-2 rounded-2xl border border-dashed border-white/10 bg-white/[0.02] py-8 text-sm text-white/30 transition-colors hover:border-violet-500/30 hover:bg-white/[0.04] hover:text-white/50">
          <span className="text-xl">+</span>
          前往 Chat 页面，和 AI 对话创建新计划
        </button>
      </div>
    </div>
  );
}
