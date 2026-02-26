import { useState, useEffect } from "react";
import { motion } from "framer-motion";
import { Coffee, Plus, Briefcase, Navigation, Radar, AlertTriangle } from "lucide-react";
import { invoke } from "../utils/tauri";
import { useNavigate } from "react-router-dom";

interface PlanData {
  persona?: string;
  trip_type?: string;
  passenger_count?: number;
  departure?: { city?: string; train_code?: string; flight_code?: string };
  destinations?: { city?: string; train_code?: string; flight_code?: string }[];
  time_window_start?: string;
  time_window_end?: string;
  return_time_window_start?: string;
  return_time_window_end?: string;
  budget_cap?: number;
}

export default function PlansPage() {
  const [plan, setPlan] = useState<PlanData | null>(null);
  const [loading, setLoading] = useState(true);
  const navigate = useNavigate();

  useEffect(() => {
    invoke<any>("get_user_plan")
      .then((data) => {
        if (data && typeof data === "object" && data.departure) {
          setPlan(data as PlanData);
        } else {
          setPlan(null);
        }
        setLoading(false);
      })
      .catch((e) => {
        console.error("Failed to load plan:", e);
        setLoading(false);
      });
  }, []);

  if (loading) return <div className="p-8 text-zinc-500">正在读取监控任务...</div>;

  const dest = plan?.destinations?.[0];

  return (
    <div className="h-full overflow-y-auto bg-white">
      <header className="border-b border-zinc-100 px-8 py-5">
        <h1 className="text-lg font-bold tracking-tight text-zinc-900">出行计划</h1>
        <p className="mt-1 text-xs text-zinc-500">管理你的监控任务</p>
      </header>

      <div className="space-y-6 p-8">
        {plan && dest ? (
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
                      <h3 className="text-base font-bold text-zinc-900 flex items-center gap-2">
                        {plan.departure?.city} 
                        {plan.trip_type === "round_trip" ? <span className="text-violet-500">↔</span> : <span className="text-zinc-300">→</span>} 
                        {dest.city}
                      </h3>
                      <p className="mt-0.5 text-xs font-medium text-zinc-500 flex flex-col gap-0.5">
                        <span>去程: {plan.time_window_start} ~ {plan.time_window_end}</span>
                        {plan.trip_type === "round_trip" && plan.return_time_window_start && (
                          <span className="text-violet-600/80">返程: {plan.return_time_window_start} ~ {plan.return_time_window_end}</span>
                        )}
                      </p>
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
                    { label: "监控预算", value: `单价￥${plan.budget_cap ?? "未设定"}` },
                    {
                      label: "人员规模",
                      value: `${plan.passenger_count || 1} 人`,
                      icon: <Briefcase className="h-4 w-4 text-zinc-400 mr-1.5" />,
                    },
                    {
                      label: "出游画像",
                      value: plan.persona === "leisure" ? "Leisure 休闲" : "Business 差旅",
                      icon: plan.persona === "leisure"
                        ? <Coffee className="h-4 w-4 text-zinc-400 mr-1.5" />
                        : <Briefcase className="h-4 w-4 text-zinc-400 mr-1.5" />,
                    },
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
        ) : (
          /* Empty state when no plan exists */
          <div className="flex flex-col items-center justify-center py-16 text-center">
            <AlertTriangle className="h-12 w-12 text-zinc-300 mb-4" />
            <h3 className="text-base font-bold text-zinc-700">暂无活跃的监控计划</h3>
            <p className="mt-2 text-sm text-zinc-500">前往 AI 出行顾问，通过对话创建你的第一个监控任务。</p>
          </div>
        )}

        {/* Create new plan button - now with real navigation */}
        <button
          onClick={() => navigate("/")}
          className="group mt-8 flex w-full flex-col items-center justify-center gap-2 rounded-2xl border-2 border-dashed border-zinc-200 bg-slate-50 py-10 transition-colors hover:border-violet-300 hover:bg-violet-50 focus:outline-none"
        >
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
