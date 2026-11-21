import { useCallback, useEffect, useRef, useState } from "react";
import { Route, Routes, useLocation, useNavigate } from "react-router-dom";
import { Command, TriangleAlert } from "lucide-react";
import TitleBar from "@/app/TitleBar";
import Sidebar, { NAV_ITEMS } from "@/app/Sidebar";
import { Button } from "@/components/ui/button";
import { Kbd, Skeleton } from "@/components/ui/states";
import { Modal } from "@/components/ui/modal";
import { TooltipProvider } from "@/components/ui/tooltip";
import { useApp } from "@/stores/app";
import Overview from "@/pages/Overview";
import Live from "@/pages/Live";
import Benchmarks from "@/pages/Benchmarks";
import Results from "@/pages/Results";
import Analysis from "@/pages/Analysis";
import Games from "@/pages/Games";
import History from "@/pages/History";
import SettingsPage from "@/pages/Settings";
import Legal from "@/pages/Legal";
import LegalNotice from "@/app/LegalNotice";

function BootSkeleton() {
  return (
    <div className="flex h-full flex-col">
      <div className="h-[46px] shrink-0 border-b border-line bg-surface" />
      <div className="flex min-h-0 flex-1">
        <div className="w-[224px] shrink-0 space-y-2 border-r border-line bg-surface p-3">
          {Array.from({ length: 7 }).map((_, index) => (
            <Skeleton key={index} className="h-8 w-full" />
          ))}
        </div>
        <div className="flex-1 space-y-4 p-6">
          <Skeleton className="h-7 w-56" />
          <div className="grid grid-cols-3 gap-3">
            {Array.from({ length: 6 }).map((_, index) => (
              <Skeleton key={index} className="h-[104px]" />
            ))}
          </div>
          <Skeleton className="h-[260px] w-full" />
        </div>
      </div>
    </div>
  );
}

function FatalState({ message }: { message: string | null }) {
  const init = useApp((state) => state.init);
  return (
    <div className="grid h-full place-items-center bg-bg p-8">
      <div className="max-w-[480px] rounded-[14px] border border-line bg-card p-6">
        <div className="flex items-center gap-2 text-err">
          <TriangleAlert className="size-4" />
          <span className="text-[13px] font-medium">FrameLab could not start monitoring</span>
        </div>
        <p className="mt-2 text-[12.5px] leading-relaxed text-text-2">{message}</p>
        <p className="mt-2 text-[11.5px] leading-relaxed text-text-3">
          Sensors are read from Windows performance counters, so this only works inside the desktop app.
        </p>
        <Button className="mt-4" variant="secondary" size="sm" onClick={() => void init()}>
          Retry
        </Button>
      </div>
    </div>
  );
}

export default function AppShell() {
  const status = useApp((state) => state.status);
  const error = useApp((state) => state.error);
  const init = useApp((state) => state.init);
  const [paletteOpen, setPaletteOpen] = useState(false);
  const navigate = useNavigate();
  const location = useLocation();
  const mainRef = useRef<HTMLElement | null>(null);

  // Reset the scroll position when the route changes, so a long page never
  // leaves the next one starting halfway down.
  useEffect(() => {
    mainRef.current?.scrollTo({ top: 0 });
  }, [location.pathname]);

  useEffect(() => {
    void init();
  }, [init]);

  const go = useCallback((path: string) => {
    navigate(path);
    setPaletteOpen(false);
  }, [navigate]);

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (!(event.ctrlKey || event.metaKey)) return;
      if (event.key.toLowerCase() === "k") {
        event.preventDefault();
        setPaletteOpen((open) => !open);
        return;
      }
      const index = Number(event.key);
      if (index >= 1 && index <= NAV_ITEMS.length) {
        event.preventDefault();
        navigate(NAV_ITEMS[index - 1].path);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [navigate]);

  if (status === "error" || status === "idle") {
    return status === "idle" ? <BootSkeleton /> : <FatalState message={error} />;
  }
  if (status === "loading") return <BootSkeleton />;

  return (
    <TooltipProvider delayDuration={280}>
      <div className="flex h-full flex-col overflow-hidden bg-bg text-text-1">
        <TitleBar onOpenPalette={() => setPaletteOpen(true)} />
        <div className="flex min-h-0 flex-1">
          <Sidebar />
          <main id="app-main" ref={mainRef} className="min-w-0 flex-1 overflow-y-auto">
            <Routes>
              <Route path="/" element={<Overview />} />
              <Route path="/live" element={<Live />} />
              <Route path="/benchmarks" element={<Benchmarks />} />
              <Route path="/results" element={<Results />} />
              <Route path="/results/:id" element={<Results />} />
              <Route path="/analysis" element={<Analysis />} />
              <Route path="/games" element={<Games />} />
              <Route path="/history" element={<History />} />
              <Route path="/settings" element={<SettingsPage />} />
              <Route path="/legal" element={<Legal />} />
            </Routes>
          </main>
        </div>
      </div>
      <Modal
        open={paletteOpen}
        onOpenChange={setPaletteOpen}
        title="Jump to"
        description="Every page is one keystroke away."
        width={420}
      >
        <div className="space-y-0.5">
          {NAV_ITEMS.map((item, index) => (
            <button
              key={item.path}
              type="button"
              onClick={() => go(item.path)}
              className="flex w-full items-center gap-2.5 rounded-[8px] px-2 py-2 text-left text-[12.5px] text-text-2 hover:bg-[color-mix(in_oklab,var(--text-1)_6%,transparent)] hover:text-text-1"
            >
              <item.icon className="size-3.5 text-text-3" />
              {item.label}
              <span className="ml-auto flex items-center gap-1 text-text-3">
                <Command className="size-3" /> <Kbd>{index + 1}</Kbd>
              </span>
            </button>
          ))}
        </div>
      </Modal>
      <LegalNotice />
    </TooltipProvider>
  );
}
