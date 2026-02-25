import { NavLink } from "react-router-dom";
import {
  MessageSquareQuote,
  LayoutDashboard,
  ClipboardList,
  BellRing,
  Settings,
  PocketKnife
} from "lucide-react";

const NAV_ITEMS = [
  { path: "/",          icon: MessageSquareQuote, label: "Chat" },
  { path: "/dashboard", icon: LayoutDashboard,     label: "Monitor" },
  { path: "/plans",     icon: ClipboardList,       label: "Plans" },
  { path: "/alerts",    icon: BellRing,            label: "Alerts" },
  { path: "/settings",  icon: Settings,            label: "Settings" },
];

export default function Sidebar() {
  return (
    <aside className="flex h-full w-[72px] flex-col items-center justify-between rounded-[24px] bg-white py-6 shadow-glass ring-1 ring-zinc-200/50">
      
      {/* Top Branding */}
      <div className="flex flex-col items-center">
        {/* Brand logo */}
        <div className="mb-8 flex h-10 w-10 items-center justify-center rounded-xl bg-zinc-900 text-white shadow-pop">
          <PocketKnife className="h-5 w-5" />
        </div>

        {/* Navigation */}
        <nav className="flex flex-col items-center gap-2">
          {NAV_ITEMS.map((item) => (
            <NavLink
              key={item.path}
              to={item.path}
              className={({ isActive }) =>
                [
                  "group relative flex h-[46px] w-[46px] items-center justify-center rounded-full transition-all duration-300",
                  isActive
                    ? "bg-zinc-900 text-white shadow-pop"
                    : "text-zinc-400 hover:bg-zinc-100 hover:text-zinc-800",
                ].join(" ")
              }
            >
              <item.icon className="relative z-10 h-5 w-5 stroke-[2px]" />
              
              {/* Tooltip */}
              <span className="pointer-events-none absolute left-[60px] whitespace-nowrap rounded-lg bg-zinc-900 px-3 py-1.5 text-[11px] font-medium text-white opacity-0 shadow-lg shadow-zinc-900/10 transition-all duration-200 group-hover:translate-x-1 group-hover:opacity-100">
                {item.label}
              </span>
            </NavLink>
          ))}
        </nav>
      </div>

      {/* Status indicator */}
      <div className="flex flex-col items-center gap-1.5">
        <span className="relative flex h-2.5 w-2.5">
          <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-emerald-500 opacity-75" />
          <span className="relative inline-flex h-2.5 w-2.5 rounded-full bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.5)]" />
        </span>
      </div>
    </aside>
  );
}
