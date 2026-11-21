import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { ArrowRight, Gauge, PlayCircle } from "lucide-react";
import { PageBody, PageHeader } from "@/components/layout/PageHeader";
import { MetricCard } from "@/components/metrics/MetricCard";
import { Panel, PanelHeader, SectionLabel } from "@/components/ui/panel";
import { Button } from "@/components/ui/button";
import { Empty, Pill } from "@/components/ui/states";
import { useApp } from "@/stores/app";
import { bytes, dateTime, duration, fps, pct, rate } from "@/lib/format";
import type { Snapshot } from "@/lib/types";

const HISTORY = 120;

function useMetricHistory(snapshot: Snapshot | null) {
  const [history, setHistory] = useState<{ cpu: number[]; gpu: number[]; ram: number[] }>({
    cpu: [],
    gpu: [],
    ram: [],
  });
  useEffect(() => {
    if (!snapshot) return;
    const push = (list: number[], value: number | null) =>
      value === null ? list : [...list, value].slice(-HISTORY);
    setHistory((previous) => ({
      cpu: push(previous.cpu, snapshot.cpuPercent.available ? snapshot.cpuPercent.value : null),
      gpu: push(previous.gpu, snapshot.gpuPercent.available ? snapshot.gpuPercent.value : null),
      ram: push(previous.ram, snapshot.ramPercent.available ? snapshot.ramPercent.value : null),
    }));
  }, [snapshot]);
  return history;
}

export default function Overview() {
  const snapshot = useApp((state) => state.snapshot);
  const hardware = useApp((state) => state.bootstrap?.hardware);
  const sessions = useApp((state) => state.sessions);
  const capture = useApp((state) => state.capture);
  const recorder = useApp((state) => state.recorder);
  const history = useMetricHistory(snapshot);
  const navigate = useNavigate();

  const vramUsed = snapshot?.vramUsedBytes;
  const vramTotal = snapshot?.vramTotalBytes;
  const vramFooter =
    vramUsed?.available && vramTotal?.available
      ? `${bytes(vramUsed.value)} of ${bytes(vramTotal.value)}`
      : undefined;

  return (
    <>
      <PageHeader
        eyebrow="This machine"
        title="Overview"
        description={
          hardware
            ? `${hardware.cpuName} · ${hardware.gpuName} · ${bytes(hardware.ramTotalBytes)} RAM`
            : "Reading hardware and performance counters from Windows."
        }
        actions={
          capture.state === "running" ? (
            <Pill tone="ok" dot>{recorder.active ? "Recording" : "Capturing"}</Pill>
          ) : (
            <Button variant="primary" size="sm" onClick={() => navigate("/live")}>
              <PlayCircle /> Start monitoring
            </Button>
          )
        }
      />

      <PageBody>
        <div className="grid grid-cols-2 gap-3 xl:grid-cols-4">
          <MetricCard
            label="CPU"
            value={pct(snapshot?.cpuPercent.value)}
            series={history.cpu}
            max={100}
            available={snapshot?.cpuPercent.available ?? false}
            reason={snapshot?.cpuPercent.reason}
            source={snapshot?.cpuPercent.source}
            method={snapshot?.cpuPercent.method}
            footer={hardware ? `${hardware.cpuCores} cores / ${hardware.cpuThreads} threads` : undefined}
          />
          <MetricCard
            label="GPU"
            value={pct(snapshot?.gpuPercent.value)}
            series={history.gpu}
            max={100}
            available={snapshot?.gpuPercent.available ?? false}
            reason={snapshot?.gpuPercent.reason}
            source={snapshot?.gpuPercent.source}
            method={snapshot?.gpuPercent.method}
            footer={snapshot?.gpuPercent.method ?? undefined}
          />
          <MetricCard
            label="Memory"
            value={pct(snapshot?.ramPercent.value)}
            series={history.ram}
            max={100}
            available={snapshot?.ramPercent.available ?? false}
            reason={snapshot?.ramPercent.reason}
            source={snapshot?.ramPercent.source}
            footer={
              snapshot?.ramUsedBytes.available ? `${bytes(snapshot.ramUsedBytes.value)} in use` : undefined
            }
          />
          <MetricCard
            label="VRAM"
            value={bytes(vramUsed?.value)}
            available={vramUsed?.available ?? false}
            reason={vramUsed?.reason}
            source={vramUsed?.source}
            footer={vramFooter}
          />
        </div>

        <div className="grid grid-cols-2 gap-3 xl:grid-cols-4">
          <MetricCard
            label="CPU clock"
            value={snapshot?.cpuClockMhz.value ? Math.round(snapshot.cpuClockMhz.value).toLocaleString() : "—"}
            unit="MHz"
            available={snapshot?.cpuClockMhz.available ?? false}
            reason={snapshot?.cpuClockMhz.reason}
            source={snapshot?.cpuClockMhz.source}
            method={snapshot?.cpuClockMhz.method}
          />
          <MetricCard
            label="Disk I/O"
            value={rate(snapshot?.diskBytesPerSec.value)}
            available={snapshot?.diskBytesPerSec.available ?? false}
            reason={snapshot?.diskBytesPerSec.reason}
            source={snapshot?.diskBytesPerSec.source}
          />
          <MetricCard
            label="Network"
            value={rate(snapshot?.netBytesPerSec.value)}
            available={snapshot?.netBytesPerSec.available ?? false}
            reason={snapshot?.netBytesPerSec.reason}
            source={snapshot?.netBytesPerSec.source}
            method={snapshot?.netBytesPerSec.method}
          />
          <MetricCard
            label="Temperature"
            value="—"
            available={snapshot?.cpuTempC.available ?? false}
            reason={snapshot?.cpuTempC.reason}
            source={snapshot?.cpuTempC.source}
          />
        </div>

        <SectionLabel className="pt-1">Recent benchmarks</SectionLabel>
        <div className="grid gap-3 lg:grid-cols-[1.6fr_1fr]">
          <Panel>
            <PanelHeader
              title="Latest sessions"
              hint="Displayed FPS is measured on screen; Presented FPS is the present cadence."
              action={
                sessions.length > 0 ? (
                  <Button variant="ghost" size="xs" onClick={() => navigate("/history")}>
                    All history <ArrowRight />
                  </Button>
                ) : null
              }
            />
            {sessions.length === 0 ? (
              <Empty
                icon={<Gauge />}
                title="No benchmark recorded yet"
                description="Attach FrameLab to a running game, record for a few minutes, and the session appears here with averages, lows and frame times."
                action={
                  <Button variant="secondary" size="sm" onClick={() => navigate("/live")}>
                    Capture a game
                  </Button>
                }
              />
            ) : (
              <ul className="divide-y divide-line border-t border-line">
                {sessions.slice(0, 5).map((session) => (
                  <li key={session.id}>
                    <button
                      type="button"
                      onClick={() => navigate(`/results/${session.id}`)}
                      className="flex w-full items-center gap-4 px-4 py-2.5 text-left hover:bg-[color-mix(in_oklab,var(--text-1)_4%,transparent)]"
                    >
                      <span className="min-w-0 flex-1">
                        <span className="block truncate text-[12.5px] text-text-1">{session.gameName}</span>
                        <span className="block text-[11px] text-text-3">
                          {dateTime(session.startedAtMs)} · {duration(session.durationMs)}
                        </span>
                      </span>
                      <span className="text-right">
                        <span className="block text-[13px] tabular-nums text-text-1">
                          {fps(session.avgFpsDisplayed, 1)} fps
                        </span>
                        <span className="block text-[10.5px] text-text-3">
                          1% low {fps(session.p1LowDisplayed, 1)}
                        </span>
                      </span>
                    </button>
                  </li>
                ))}
              </ul>
            )}
          </Panel>

          <Panel>
            <PanelHeader title="Hardware" hint="Detected from Windows, no cloud lookup." />
            <dl className="space-y-2 border-t border-line px-4 py-3 text-[12px]">
              {([
                ["CPU", hardware?.cpuName],
                ["GPU", hardware?.gpuName],
                ["Memory", hardware ? bytes(hardware.ramTotalBytes) : null],
                ["Display", hardware?.display ? `${hardware.display.width}×${hardware.display.height} @ ${hardware.display.refreshHz} Hz` : null],
                ["System", hardware ? `${hardware.osName} ${hardware.osVersion} (${hardware.osBuild})` : null],
                ["Adapter LUID", hardware?.adapterLuid],
              ] as const).map(([label, value]) => (
                <div key={label} className="flex items-baseline justify-between gap-4">
                  <dt className="text-text-3">{label}</dt>
                  <dd className="truncate text-right text-text-2">{value ?? "—"}</dd>
                </div>
              ))}
            </dl>
            {hardware?.notes.length ? (
              <div className="border-t border-line px-4 py-2.5 text-[10.5px] leading-relaxed text-text-3">
                {hardware.notes.map((note) => (
                  <div key={note}>{note}</div>
                ))}
              </div>
            ) : null}
          </Panel>
        </div>
      </PageBody>
    </>
  );
}
