import { NavLink } from "react-router-dom";
import { motion } from "framer-motion";

const NAV_ITEMS = [
  { path: "/",          icon: "💬", label: "Chat" },
  { path: "/dashboard", icon: "📊", label: "Monitor" },
  { path: "/plans",     icon: "📋", label: "Plans" },
  { path: "/alerts",    icon: "🔔", label: "Alerts" },
  { path: "/settings",  icon: "⚙️", label: "Settings" },
];

export default function Sidebar() {
  return (
    <aside className="flex h-screen w-[72px] flex-col items-center border-r border-white/[0.06] bg-black/40 py-6 backdrop-blur-xl">
      {/* Brand logo */}
      <div className="mb-8 flex h-10 w-10 items-center justify-center rounded-xl bg-gradient-to-br from-violet-600 to-fuchsia-500 text-lg font-black text-white shadow-lg shadow-violet-500/30">
        M
      </div>

      {/* Navigation */}
      <nav className="flex flex-1 flex-col items-center gap-1">
        {NAV_ITEMS.map((item) => (
          <NavLink
            key={item.path}
            to={item.path}
            className={({ isActive }) =>
              [
                "group relative flex h-11 w-11 items-center justify-center rounded-xl transition-all duration-200",
                isActive
                  ? "bg-white/10 shadow-lg shadow-violet-500/10"
                  : "hover:bg-white/[0.06]",
              ].join(" ")
            }
          >
            {({ isActive }) => (
              <>
                {isActive && (
                  <motion.div
                    layoutId="activeTab"
                    className="absolute left-0 h-6 w-[3px] rounded-r-full bg-gradient-to-b from-violet-400 to-fuchsia-400"
                    transition={{ type: "spring", stiffness: 380, damping: 30 }}
                  />
                )}
                <span className="text-lg">{item.icon}</span>
                {/* Tooltip */}
                <span className="pointer-events-none absolute left-16 rounded-lg bg-zinc-800 px-2.5 py-1 text-xs font-medium text-white/80 opacity-0 shadow-xl transition-opacity group-hover:opacity-100">
                  {item.label}
                </span>
              </>
            )}
          </NavLink>
        ))}
      </nav>

      {/* Status indicator */}
      <div className="flex flex-col items-center gap-1">
        <span className="relative flex h-2 w-2">
          <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-emerald-400 opacity-75" />
          <span className="relative inline-flex h-2 w-2 rounded-full bg-emerald-400" />
        </span>
        <span className="text-[9px] font-medium uppercase tracking-widest text-white/30">
          Live
        </span>
      </div>
    </aside>
  );
}
