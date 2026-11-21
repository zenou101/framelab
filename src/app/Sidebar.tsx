import { NavLink } from "react-router-dom";
import {
  Activity, ChartLine, ChevronLeft, ChevronRight, Gamepad2, Gauge,
  History as HistoryIcon, Settings as SettingsIcon, ShieldCheck, Stethoscope, Timer,
} from "lucide-react";
import { Tooltip } from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";
import { ipc } from "@/lib/ipc";
import { useApp } from "@/stores/app";

export const NAV_ITEMS = [
  { path: "/", label: "Overview", icon: Gauge },
  { path: "/live", label: "Live Monitor", icon: Activity },
  { path: "/benchmarks", label: "Benchmarks", icon: Timer },
  { path: "/results", label: "Results", icon: ChartLine },
  { path: "/analysis", label: "Analysis", icon: Stethoscope },
  { path: "/games", label: "Games", icon: Gamepad2 },
  { path: "/history", label: "History", icon: HistoryIcon },
  { path: "/settings", label: "Settings", icon: SettingsIcon },
  { path: "/legal", label: "Legal & About", icon: ShieldCheck },
] as const;

export default function Sidebar() {
  const settings = useApp((state) => state.settings);
  const recorder = useApp((state) => state.recorder);
  const patchSettings = useApp((state) => state.patchSettings);
  const collapsed = settings?.sidebarCollapsed ?? false;
  const dataDir = useApp((state) => state.bootstrap?.dataDir);

  return (
    <aside
      className={cn(
        "flex shrink-0 flex-col border-r border-line bg-surface transition-[width] duration-150",
        collapsed ? "w-[60px]" : "w-[224px]",
      )}
    >
      <nav className="flex-1 space-y-0.5 overflow-y-auto p-2" aria-label="Primary">
        {NAV_ITEMS.map((item, index) => {
          const button = (
            <NavLink
              key={item.path}
              to={item.path}
              end={item.path === "/"}
              className={({ isActive }) =>
                cn(
                  "group relative flex h-8 items-center gap-2.5 rounded-[8px] px-2 text-[12.5px] transition-colors",
                  isActive
                    ? "bg-[color-mix(in_oklab,var(--accent)_12%,transparent)] text-text-1"
                    : "text-text-2 hover:bg-[color-mix(in_oklab,var(--text-1)_5%,transparent)] hover:text-text-1",
                )
              }
            >
              {({ isActive }) => (
                <>
                  {isActive ? <span className="absolute left-0 h-4 w-[2px] rounded-r bg-accent" /> : null}
                  <item.icon className={cn("size-4 shrink-0", isActive ? "text-accent" : "text-text-3")} />
                  {collapsed ? null : (
                    <>
                      <span className="truncate">{item.label}</span>
                      <span className="ml-auto text-[10px] text-text-3 opacity-0 transition-opacity group-hover:opacity-100">
                        {index + 1}
                      </span>
                    </>
                  )}
                </>
              )}
            </NavLink>
          );
          return collapsed ? (
            <Tooltip key={item.path} content={item.label} side="right">
              {button}
            </Tooltip>
          ) : (
            button
          );
        })}
      </nav>

      <div className="border-t border-line p-2">
        {recorder.active && !collapsed ? (
          <div className="mb-1.5 rounded-[8px] border border-line bg-card px-2 py-1.5">
            <div className="flex items-center gap-1.5 text-[10.5px] text-text-3">
              <span className="size-1.5 animate-pulse rounded-full bg-err" />
              Recording
            </div>
            <div className="mt-1 truncate text-[11.5px] text-text-1">{recorder.process ?? "session"}</div>
            <div className="mt-0.5 text-[10.5px] tabular-nums text-text-3">
              {recorder.displayedFrames.toLocaleString()} frames
            </div>
          </div>
        ) : null}
        <div className="flex items-center gap-1">
          <button
            type="button"
            title="Open data folder"
            onClick={() => void ipc.openDataDir().catch(() => undefined)}
            disabled={!dataDir}
            className="flex h-7 min-w-0 flex-1 items-center gap-1.5 rounded-[6px] px-1.5 text-[10.5px] text-text-3 hover:text-text-2 disabled:opacity-40"
          >
            <span className="size-1.5 rounded-full bg-ok" />
            {collapsed ? null : <span className="truncate">Local data</span>}
          </button>
          <button
            type="button"
            aria-label={collapsed ? "Expand sidebar" : "Collapse sidebar"}
            onClick={() => void patchSettings({ sidebarCollapsed: !collapsed })}
            className="grid size-7 place-items-center rounded-[6px] text-text-3 hover:bg-[color-mix(in_oklab,var(--text-1)_6%,transparent)] hover:text-text-1"
          >
            {collapsed ? <ChevronRight className="size-3.5" /> : <ChevronLeft className="size-3.5" />}
          </button>
        </div>
      </div>
    </aside>
  );
}
