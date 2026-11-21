import { useEffect, useState, type ReactNode } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { FolderOpen } from "lucide-react";
import { PageBody, PageHeader } from "@/components/layout/PageHeader";
import { Panel, PanelHeader, SectionLabel } from "@/components/ui/panel";
import { Button } from "@/components/ui/button";
import { Modal } from "@/components/ui/modal";
import { Segmented } from "@/components/ui/segmented";
import { Select } from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { ipc, isDesktop } from "@/lib/ipc";
import { bytes, pct } from "@/lib/format";
import { useApp } from "@/stores/app";
import type { DataStats, SelfMetrics, Thresholds } from "@/lib/types";

function Row({ label, hint, children }: { label: string; hint?: ReactNode; children: ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-6 border-b border-line px-4 py-2.5 last:border-0">
      <div className="min-w-0">
        <div className="text-[12.5px] text-text-1">{label}</div>
        {hint ? <div className="mt-0.5 text-[11px] leading-relaxed text-text-3">{hint}</div> : null}
      </div>
      <div className="shrink-0">{children}</div>
    </div>
  );
}

const ACCENTS = ["blue", "violet", "teal", "amber", "rose", "lime"] as const;

const SWATCH: Record<(typeof ACCENTS)[number], string> = {
  blue: "#6b9dff",
  violet: "#a78bfa",
  teal: "#46c9b8",
  amber: "#e0a95c",
  rose: "#f08d9a",
  lime: "#a3d977",
};

const THRESHOLD_FIELDS: { key: keyof Thresholds; label: string; step: number }[] = [
  { key: "spikeMultiplier", label: "Spike multiple of median", step: 0.1 },
  { key: "spikeFloorMs", label: "Spike floor (ms)", step: 1 },
  { key: "pacingCv", label: "Pacing CV limit", step: 0.01 },
  { key: "ramPressurePct", label: "RAM pressure (%)", step: 1 },
  { key: "vramPressurePct", label: "VRAM pressure (%)", step: 1 },
  { key: "gpuDipPoints", label: "GPU dip (points)", step: 1 },
  { key: "singleCorePct", label: "Single-core limit (%)", step: 1 },
  { key: "backgroundCpuPct", label: "Background CPU (%)", step: 1 },
];

export default function SettingsPage() {
  const settings = useApp((state) => state.settings);
  const snapshot = useApp((state) => state.snapshot);
  const sidecar = useApp((state) => state.bootstrap?.sidecar);
  const dataDir = useApp((state) => state.bootstrap?.dataDir);
  const version = useApp((state) => state.bootstrap?.version);
  const patchSettings = useApp((state) => state.patchSettings);
  const refreshSessions = useApp((state) => state.refreshSessions);
  const [stats, setStats] = useState<DataStats | null>(null);
  const [overhead, setOverhead] = useState<SelfMetrics | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [clearing, setClearing] = useState(false);

  useEffect(() => {
    void ipc.dataStats().then(setStats).catch(() => undefined);
    const poll = () => void ipc.selfMetrics().then(setOverhead).catch(() => undefined);
    poll();
    const id = window.setInterval(poll, 3000);
    return () => window.clearInterval(id);
  }, []);

  if (!settings) return null;

  const updateThreshold = (key: keyof Thresholds, value: number) => {
    if (!Number.isFinite(value)) return;
    void patchSettings({ thresholds: { ...settings.thresholds, [key]: value } }).catch((reason: unknown) =>
      setError(reason instanceof Error ? reason.message : String(reason)),
    );
  };

  const toggleAutostart = async (enabled: boolean) => {
    try {
      await ipc.setAutostart(enabled);
      await patchSettings({ launchOnStartup: enabled });
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    }
  };

  const toggleNativeFrame = async (enabled: boolean) => {
    try {
      if (isDesktop) await getCurrentWindow().setDecorations(enabled);
      await patchSettings({ nativeFrame: enabled });
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    }
  };

  const clearData = async () => {
    try {
      await ipc.clearSessions();
      setClearing(false);
      await refreshSessions();
      setStats(await ipc.dataStats());
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    }
  };

  const sensors = [
    { name: "CPU utilisation", metric: snapshot?.cpuPercent },
    { name: "CPU clock", metric: snapshot?.cpuClockMhz },
    { name: "GPU utilisation", metric: snapshot?.gpuPercent },
    { name: "VRAM used", metric: snapshot?.vramUsedBytes },
    { name: "VRAM total", metric: snapshot?.vramTotalBytes },
    { name: "Memory", metric: snapshot?.ramPercent },
    { name: "Disk throughput", metric: snapshot?.diskBytesPerSec },
    { name: "Network throughput", metric: snapshot?.netBytesPerSec },
    { name: "CPU temperature", metric: snapshot?.cpuTempC },
    { name: "GPU temperature", metric: snapshot?.gpuTempC },
  ];

  return (
    <>
      <PageHeader
        eyebrow={`FrameLab ${version ?? ""}`}
        title="Settings"
        description="Everything here is stored in a JSON file next to your database. No account, no sync, no telemetry."
      />
      <PageBody>
        {error ? <div className="text-[12px] text-err">{error}</div> : null}

        <SectionLabel>Appearance</SectionLabel>
        <Panel>
          <Row label="Theme" hint="Both themes are hand-tuned; contrast is checked per surface.">
            <Segmented
              value={settings.theme === "light" ? "light" : "dark"}
              onChange={(value) => void patchSettings({ theme: value })}
              options={[
                { value: "dark", label: "Dark" },
                { value: "light", label: "Light" },
              ]}
              ariaLabel="Theme"
            />
          </Row>
          <Row label="Accent" hint="Derives the interaction colour and the first chart series.">
            <div className="flex items-center gap-1.5">
              {ACCENTS.map((accent) => (
                <button
                  key={accent}
                  type="button"
                  aria-label={`${accent} accent`}
                  aria-pressed={settings.accent === accent}
                  onClick={() => void patchSettings({ accent })}
                  className={`size-5 rounded-full border ${
                    settings.accent === accent ? "border-text-1" : "border-transparent"
                  }`}
                  style={{ background: SWATCH[accent] }}
                >
                  <span className="sr-only">{accent}</span>
                </button>
              ))}
            </div>
          </Row>
          <Row label="Compact density" hint="Tighter rows for smaller displays.">
            <Switch checked={settings.compact} onCheckedChange={(value) => void patchSettings({ compact: value })} />
          </Row>
          <Row label="Reduce motion" hint="Honours the Windows animation setting by default.">
            <Switch checked={settings.reduceMotion} onCheckedChange={(value) => void patchSettings({ reduceMotion: value })} />
          </Row>
          <Row label="Native window frame" hint="Needed for Windows Snap Layouts; FrameLab's own title bar is hidden while this is on.">
            <Switch checked={settings.nativeFrame} onCheckedChange={(value) => void toggleNativeFrame(value)} />
          </Row>
        </Panel>

        <SectionLabel>Monitoring</SectionLabel>
        <Panel>
          <Row label="Sample interval" hint="How often sensor counters are read and charted.">
            <div className="w-[140px]">
              <Select
                value={String(settings.samplingMs)}
                onValueChange={(value) => void patchSettings({ samplingMs: Number(value) })}
                options={[
                  { value: "250", label: "250 ms" },
                  { value: "500", label: "500 ms" },
                  { value: "1000", label: "1 second" },
                  { value: "2000", label: "2 seconds" },
                ]}
                size="sm"
              />
            </div>
          </Row>
          <Row label="Headline frame source" hint="Both series are always stored; this only changes which one leads a result.">
            <Segmented
              value={settings.primarySource === "presented" ? "presented" : "displayed"}
              onChange={(value) => void patchSettings({ primarySource: value })}
              options={[
                { value: "displayed", label: "Displayed" },
                { value: "presented", label: "Presented" },
              ]}
            />
          </Row>
          <Row label="Keep sampling in background" hint="Off means counters pause when the window is hidden.">
            <Switch
              checked={settings.backgroundMonitoring}
              onCheckedChange={(value) => void patchSettings({ backgroundMonitoring: value })}
            />
          </Row>
          <Row label="Launch at sign-in" hint="Uses the Windows startup registry entry through the autostart plugin.">
            <Switch checked={settings.launchOnStartup} onCheckedChange={(value) => void toggleAutostart(value)} />
          </Row>
        </Panel>

        <SectionLabel>Diagnostic thresholds</SectionLabel>
        <Panel>
          <PanelHeader
            title="Heuristic limits"
            hint="Analysis compares measurements against these numbers. Changing them changes what gets reported, never what was measured."
          />
          <div className="grid grid-cols-2 gap-x-6 border-t border-line px-4 py-3 md:grid-cols-4">
            {THRESHOLD_FIELDS.map((field) => (
              <label key={field.key} className="flex flex-col gap-1 py-1">
                <span className="text-[11px] text-text-3">{field.label}</span>
                <input
                  type="number"
                  step={field.step}
                  value={settings.thresholds[field.key]}
                  onChange={(event) => updateThreshold(field.key, Number(event.target.value))}
                  className="h-7 w-full rounded-[6px] border border-line bg-elevated px-2 text-[12px] tabular-nums text-text-1"
                />
              </label>
            ))}
          </div>
        </Panel>

        <SectionLabel>Data</SectionLabel>
        <Panel>
          <Row
            label="Stored sessions"
            hint={stats ? `${stats.sessions} sessions · ${bytes(stats.databaseBytes)} on disk` : "Reading database size…"}
          >
            <div className="flex items-center gap-1.5">
              <Button variant="secondary" size="xs" onClick={() => void ipc.openDataDir().catch(() => undefined)}>
                <FolderOpen /> Folder
              </Button>
              <Button variant="danger" size="xs" onClick={() => setClearing(true)}>
                Delete all
              </Button>
            </div>
          </Row>
          <Row label="Retention" hint="0 keeps everything until you delete it.">
            <div className="w-[140px]">
              <Select
                value={String(settings.retentionDays)}
                onValueChange={(value) => void patchSettings({ retentionDays: Number(value) })}
                options={[
                  { value: "0", label: "Keep all" },
                  { value: "30", label: "30 days" },
                  { value: "90", label: "90 days" },
                  { value: "365", label: "1 year" },
                ]}
                size="sm"
              />
            </div>
          </Row>
          <Row label="Export format" hint="CSV keeps frame times per row; JSON also carries metadata and samples.">
            <Segmented
              value={settings.exportFormat === "json" ? "json" : "csv"}
              onChange={(value) => void patchSettings({ exportFormat: value })}
              options={[
                { value: "csv", label: "CSV" },
                { value: "json", label: "JSON" },
              ]}
            />
          </Row>
          <Row
            label="Apply retention now"
            hint="Runs the age filter immediately and reports how many sessions were removed."
          >
            <Button
              variant="secondary"
              size="xs"
              onClick={() =>
                void ipc
                  .applyRetention()
                  .then(async (removed) => {
                    setError(removed > 0 ? `Removed ${removed} old sessions.` : "Nothing was older than the retention window.");
                    await refreshSessions();
                    setStats(await ipc.dataStats());
                  })
                  .catch((reason: unknown) => setError(reason instanceof Error ? reason.message : String(reason)))
              }
            >
              Apply
            </Button>
          </Row>
          {dataDir ? <div className="px-4 py-2 font-mono text-[10.5px] break-all text-text-3">{dataDir}</div> : null}
        </Panel>

        <SectionLabel>Sensor availability</SectionLabel>
        <Panel>
          <PanelHeader
            title="What this machine exposes"
            hint="Unavailable values are reported with the reason instead of being shown as zero."
          />
          <table className="w-full border-collapse text-[12px]">
            <thead>
              <tr className="border-y border-line text-left text-[10.5px] tracking-[0.05em] text-text-3 uppercase">
                <th className="px-4 py-1.5 font-medium">Metric</th>
                <th className="px-3 py-1.5 font-medium">Source</th>
                <th className="px-3 py-1.5 font-medium">Status</th>
              </tr>
            </thead>
            <tbody>
              {sensors.map(({ name, metric }) => (
                <tr key={name} className="border-b border-line last:border-0">
                  <td className="px-4 py-1.5 text-text-1">{name}</td>
                  <td className="px-3 py-1.5 text-text-3">{metric?.source ?? "not sampled yet"}</td>
                  <td className="px-3 py-1.5">
                    {metric?.available ? (
                      <span className="text-ok">available</span>
                    ) : (
                      <span className="text-text-3">{metric?.reason ?? "waiting for the first sample"}</span>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </Panel>

        <SectionLabel>FrameLab's own overhead</SectionLabel>
        <Panel>
          <div className="grid grid-cols-2 gap-4 px-4 py-3 md:grid-cols-4">
            <div>
              <div className="text-[10.5px] tracking-[0.05em] text-text-3 uppercase">CPU now</div>
              <div className="text-[18px] tabular-nums">{pct(overhead?.cpuPercent ?? null, 1)}</div>
            </div>
            <div>
              <div className="text-[10.5px] tracking-[0.05em] text-text-3 uppercase">Resident memory</div>
              <div className="text-[18px] tabular-nums">{bytes(overhead?.memoryBytes ?? null)}</div>
            </div>
            <div>
              <div className="text-[10.5px] tracking-[0.05em] text-text-3 uppercase">Uptime</div>
              <div className="text-[18px] tabular-nums">
                {overhead ? `${Math.round(overhead.uptimeMs / 1000)}s` : "—"}
              </div>
            </div>
            <div>
              <div className="text-[10.5px] tracking-[0.05em] text-text-3 uppercase">PresentMon</div>
              <div className="text-[13px] text-text-2">{sidecar ?? "unknown"}</div>
            </div>
          </div>
          <div className="border-t border-line px-4 py-2 text-[11px] text-text-3">
            Sampled from the process itself. Targets: under 2% CPU while charting, under 250 MB resident.
          </div>
        </Panel>
      </PageBody>

      <Modal
        open={clearing}
        onOpenChange={setClearing}
        title="Delete every stored session?"
        description="Frames, samples and summaries are removed from the local database. Your games library stays."
        footer={
          <>
            <Button variant="ghost" size="sm" onClick={() => setClearing(false)}>
              Cancel
            </Button>
            <Button variant="danger" size="sm" onClick={() => void clearData()}>
              Delete sessions
            </Button>
          </>
        }
      />
    </>
  );
}
