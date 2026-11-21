import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, Search, Square, X } from "lucide-react";
import { Pill } from "@/components/ui/states";
import { Button } from "@/components/ui/button";
import { Tooltip } from "@/components/ui/tooltip";
import { isDesktop } from "@/lib/ipc";
import { pct } from "@/lib/format";
import { useApp } from "@/stores/app";

function Mark() {
  return (
    <span className="flex size-[22px] items-center justify-center rounded-[6px] border border-line-strong bg-elevated">
      <svg viewBox="0 0 16 16" className="size-3.5" aria-hidden>
        <rect x="1.5" y="8" width="2.6" height="6" rx="1" fill="var(--accent)" opacity=".55" />
        <rect x="6.2" y="4" width="2.6" height="10" rx="1" fill="var(--accent)" />
        <rect x="10.9" y="1.5" width="2.6" height="12.5" rx="1" fill="var(--accent)" opacity=".7" />
      </svg>
    </span>
  );
}

function RailItem({ label, value }: { label: string; value: string }) {
  return (
    <span className="flex items-baseline gap-1.5">
      <span className="text-[9.5px] tracking-[0.08em] text-text-3 uppercase">{label}</span>
      <span className="text-[11.5px] tabular-nums text-text-2">{value}</span>
    </span>
  );
}

export default function TitleBar({ onOpenPalette }: { onOpenPalette: () => void }) {
  const snapshot = useApp((state) => state.snapshot);
  const frame = useApp((state) => state.frame);
  const capture = useApp((state) => state.capture);
  const recorder = useApp((state) => state.recorder);
  const version = useApp((state) => state.bootstrap?.version);
  const win = isDesktop ? getCurrentWindow() : null;

  const gpu = snapshot?.gpuPercent;
  const cpu = snapshot?.cpuPercent;
  const ram = snapshot?.ramPercent;
  const fpsNow = frame?.displayedFps ?? frame?.presentedFps ?? null;

  return (
    <header className="app-drag flex h-[46px] shrink-0 items-center gap-3 border-b border-line bg-surface pr-0 pl-3.5">
      <div className="flex items-center gap-2">
        <Mark />
        <span className="text-[12.5px] font-semibold tracking-[-0.012em]">FrameLab</span>
        {version ? <span className="text-[10.5px] text-text-3">v{version}</span> : null}
      </div>

      <span className="h-4 w-px bg-line" />

      {capture.state === "running" ? (
        <Pill tone="ok" dot>
          {recorder.active ? "Recording" : "Capturing"} · {capture.process ?? "process"}
        </Pill>
      ) : (
        <Pill tone={capture.state === "error" ? "err" : "neutral"} dot>
          {capture.state === "error" ? "Capture error" : "Idle"}
        </Pill>
      )}

      <div className="app-no-drag ml-auto hidden items-center gap-4 xl:flex">
        <div className="flex items-center gap-3.5 border-r border-line pr-4">
          <RailItem label="CPU" value={pct(cpu?.value)} />
          <RailItem label="GPU" value={pct(gpu?.value)} />
          <RailItem label="RAM" value={pct(ram?.value)} />
          <RailItem label="FPS" value={fpsNow ? fpsNow.toFixed(0) : "—"} />
        </div>
        <Tooltip content="Command palette · Ctrl K">
          <Button variant="ghost" size="iconSm" onClick={onOpenPalette} aria-label="Open command palette">
            <Search />
          </Button>
        </Tooltip>
      </div>

      <div className="app-no-drag ml-2 flex items-stretch">
        <button
          type="button"
          aria-label="Minimize"
          onClick={() => void win?.minimize()}
          className="grid h-[46px] w-[46px] place-items-center text-text-2 hover:bg-[color-mix(in_oklab,var(--text-1)_7%,transparent)] hover:text-text-1"
        >
          <Minus className="size-3.5" />
        </button>
        <button
          type="button"
          aria-label="Maximize"
          onClick={() => void win?.toggleMaximize()}
          className="grid h-[46px] w-[46px] place-items-center text-text-2 hover:bg-[color-mix(in_oklab,var(--text-1)_7%,transparent)] hover:text-text-1"
        >
          <Square className="size-3" />
        </button>
        <button
          type="button"
          aria-label="Close"
          onClick={() => void win?.close()}
          className="grid h-[46px] w-[46px] place-items-center text-text-2 hover:bg-err hover:text-white"
        >
          <X className="size-4" />
        </button>
      </div>
    </header>
  );
}
