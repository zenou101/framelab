import { useEffect, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import { AlertTriangle, CircleDot, Square, Target } from "lucide-react";
import { PageBody, PageHeader } from "@/components/layout/PageHeader";
import { Panel, PanelHeader } from "@/components/ui/panel";
import { Button } from "@/components/ui/button";
import { Select } from "@/components/ui/select";
import { Empty, Pill, Spinner } from "@/components/ui/states";
import { ipc } from "@/lib/ipc";
import { fps as fmtFps, ms as fmtMs } from "@/lib/format";
import { useApp } from "@/stores/app";
import { useLiveFrames } from "@/lib/useLive";
import type { ProcessInfo } from "@/lib/types";

function Oscilloscope({ frames }: { frames: number[] }) {
  const ref = useRef<HTMLCanvasElement | null>(null);

  useEffect(() => {
    const canvas = ref.current;
    const context = canvas?.getContext("2d");
    if (!canvas || !context) return;
    const ratio = window.devicePixelRatio || 1;
    const width = canvas.clientWidth;
    const height = canvas.clientHeight;
    canvas.width = Math.max(1, Math.round(width * ratio));
    canvas.height = Math.max(1, Math.round(height * ratio));
    context.setTransform(ratio, 0, 0, ratio, 0, 0);
    context.clearRect(0, 0, width, height);

    const styles = getComputedStyle(document.documentElement);
    const grid = styles.getPropertyValue("--grid").trim() || "#222";
    const accent = styles.getPropertyValue("--accent").trim() || "#6b9dff";
    const warn = styles.getPropertyValue("--warn").trim() || "#e8b45c";

    const slice = frames.slice(-600);
    const ceiling = Math.max(20, ...slice) * 1.1;
    const y = (value: number) => height - (value / ceiling) * height;

    context.strokeStyle = grid;
    context.lineWidth = 1;
    for (const step of [16.7, 33.3]) {
      if (step > ceiling) continue;
      context.beginPath();
      context.moveTo(0, y(step));
      context.lineTo(width, y(step));
      context.stroke();
    }

    if (slice.length < 2) return;
    context.strokeStyle = accent;
    context.lineWidth = 1.4;
    context.beginPath();
    const dx = width / (slice.length - 1);
    slice.forEach((value, index) => {
      const pointX = index * dx;
      if (index === 0) context.moveTo(pointX, y(value));
      else {
        context.lineTo(pointX, y(slice[index - 1]));
        context.lineTo(pointX, y(value));
      }
    });
    context.stroke();

    context.strokeStyle = warn;
    context.setLineDash([3, 4]);
    context.beginPath();
    context.moveTo(0, y(33.3));
    context.lineTo(width, y(33.3));
    context.stroke();
    context.setLineDash([]);
  }, [frames]);

  return <canvas ref={ref} className="h-[240px] w-full" aria-label="Frame time oscilloscope" />;
}

export default function Live() {
  const capture = useApp((state) => state.capture);
  const recorder = useApp((state) => state.recorder);
  const series = useApp((state) => state.series);
  const frame = useApp((state) => state.frame);
  const snapshot = useApp((state) => state.snapshot);
  const samplingMs = useApp((state) => state.settings?.samplingMs ?? 1000);
  const sidecar = useApp((state) => state.bootstrap?.sidecar ?? "unknown");
  const setCapture = useApp((state) => state.setCapture);
  const setRecorder = useApp((state) => state.setRecorder);
  const refreshSessions = useApp((state) => state.refreshSessions);
  const [processes, setProcesses] = useState<ProcessInfo[]>([]);
  const [pid, setPid] = useState<number | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const navigate = useNavigate();

  useLiveFrames(true, recorder.active ? 400 : 700);

  useEffect(() => {
    ipc
      .processes()
      .then((list) => {
        setProcesses(list);
        const foreground = list.find((entry) => entry.isForeground);
        setPid((current) => current ?? foreground?.pid ?? list[0]?.pid ?? null);
      })
      .catch((reason: unknown) => setError(reason instanceof Error ? reason.message : String(reason)));
  }, []);

  const selected = processes.find((entry) => entry.pid === pid) ?? null;
  const options = processes
    .slice()
    .sort((a, b) => Number(b.isForeground) - Number(a.isForeground) || b.cpuPercent - a.cpuPercent)
    .slice(0, 160)
    .map((entry) => ({
      value: String(entry.pid),
      label: `${entry.name} · ${entry.pid}`,
      hint: entry.isForeground ? "foreground window" : `${entry.cpuPercent.toFixed(1)}% CPU`,
    }));

  const startCapture = async () => {
    if (!selected) return;
    setBusy(true);
    setError(null);
    try {
      setCapture(await ipc.startCapture(selected.pid, selected.name));
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  };

  const stopCapture = async () => {
    setBusy(true);
    try {
      setCapture(await ipc.stopCapture());
    } finally {
      setBusy(false);
    }
  };

  const startRecording = async () => {
    if (!selected) return;
    setBusy(true);
    setError(null);
    try {
      setRecorder(await ipc.startRecording(selected.name.replace(/\.exe$/i, ""), null));
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  };

  const stopRecording = async () => {
    setBusy(true);
    try {
      const id = await ipc.stopRecording();
      setRecorder({ ...recorder, active: false });
      await refreshSessions();
      if (id) navigate(`/results/${id}`);
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  };

  const running = capture.state === "running";
  const fpsNow = frame?.displayedFps ?? null;
  const presentedNow = frame?.presentedFps ?? null;
  const syncInterval =
    frame?.syncInterval === null || frame?.syncInterval === undefined ? null : String(frame.syncInterval);

  return (
    <>
      <PageHeader
        eyebrow="PresentMon · ETW"
        title="Live Monitor"
        description="Frame times come from PresentMon's event stream. Displayed FPS is what reached the screen; Presented FPS is what the game submitted."
        actions={
          running ? (
            <Button variant="secondary" size="sm" onClick={() => void stopCapture()} disabled={busy}>
              <Square /> Stop capture
            </Button>
          ) : (
            <Button variant="primary" size="sm" onClick={() => void startCapture()} disabled={busy || !selected}>
              <CircleDot /> Start capture
            </Button>
          )
        }
      />

      <PageBody>
        <Panel>
          <PanelHeader
            title="Target process"
            hint="FrameLab attaches read-only to a game that is already running."
            action={running ? <Pill tone="ok" dot>{capture.process}</Pill> : undefined}
          />
          <div className="flex flex-wrap items-center gap-2 border-t border-line px-4 py-3">
            <div className="min-w-[260px] flex-1">
              <Select
                value={pid === null ? null : String(pid)}
                onValueChange={(value) => setPid(Number(value))}
                options={options}
                placeholder={processes.length ? "Select a running process" : "Reading process list…"}
                ariaLabel="Target process"
                size="sm"
              />
            </div>
            <Button
              variant="secondary"
              size="sm"
              onClick={() => void ipc.processes().then(setProcesses).catch(() => undefined)}
            >
              Refresh
            </Button>
            <span className="text-[11px] text-text-3">sensor sampling {samplingMs} ms</span>
          </div>
        </Panel>

        {error ? (
          <div className="flex items-start gap-2 rounded-[10px] border border-[color-mix(in_oklab,var(--err)_32%,transparent)] bg-[color-mix(in_oklab,var(--err)_8%,transparent)] px-3 py-2.5 text-[12px] text-err">
            <AlertTriangle className="mt-0.5 size-3.5 shrink-0" />
            <div className="min-w-0">
              <div className="leading-relaxed">{error}</div>
              {/privilege|elevat|access/i.test(error) ? (
                <Button
                  variant="danger"
                  size="xs"
                  className="mt-2"
                  onClick={() => void ipc.restartElevated().catch(() => undefined)}
                >
                  Restart as administrator
                </Button>
              ) : null}
            </div>
          </div>
        ) : null}

        {capture.message ? (
          <div className="rounded-[10px] border border-line bg-card px-3 py-2 text-[11.5px] text-text-2">
            {capture.message}
          </div>
        ) : null}

        <div className="grid gap-3 lg:grid-cols-2">
          <Panel>
            <PanelHeader
              title="Frame rate"
              hint={running ? "Live from the attached process" : "Attach a process to read frame timings"}
              action={<Target className="size-3.5 text-text-3" />}
            />
            <div className="grid grid-cols-2 divide-x divide-line border-t border-line">
              <div className="px-4 py-3">
                <div className="text-[10.5px] tracking-[0.055em] text-text-3 uppercase">Displayed FPS</div>
                <div className="mt-1 flex items-baseline gap-1">
                  <span className="text-[30px] leading-none font-medium tabular-nums text-text-1">
                    {fmtFps(fpsNow, 1)}
                  </span>
                  <span className="text-[11px] text-text-3">fps</span>
                </div>
                <div className="mt-1 text-[10.5px] text-text-3">
                  {fmtMs(frame?.displayedFrameMs ?? null)} per frame on screen
                </div>
              </div>
              <div className="px-4 py-3">
                <div className="text-[10.5px] tracking-[0.055em] text-text-3 uppercase">Presented FPS</div>
                <div className="mt-1 flex items-baseline gap-1">
                  <span className="text-[30px] leading-none font-medium tabular-nums text-text-2">
                    {fmtFps(presentedNow, 1)}
                  </span>
                  <span className="text-[11px] text-text-3">fps</span>
                </div>
                <div className="mt-1 text-[10.5px] text-text-3">
                  {fmtMs(frame?.presentedFrameMs ?? null)} between presents
                </div>
              </div>
            </div>
            <dl className="grid grid-cols-2 gap-x-4 gap-y-1.5 border-t border-line px-4 py-2.5 text-[11.5px]">
              {[
                ["Present mode", frame?.presentMode ?? null],
                ["Runtime", frame?.runtime ?? null],
                ["Sync interval", syncInterval],
                ["Frames on screen", frame ? frame.capturedDisplayed.toLocaleString() : null],
                ["Frames presented", frame ? frame.capturedPresented.toLocaleString() : null],
                ["PresentMon", sidecar],
              ].map(([label, value]) => (
                <div key={label} className="flex items-baseline justify-between gap-3">
                  <dt className="text-text-3">{label}</dt>
                  <dd className="truncate text-right tabular-nums text-text-2">{value ?? "—"}</dd>
                </div>
              ))}
            </dl>
          </Panel>

          <Panel>
            <PanelHeader
              title="Benchmark recording"
              hint="Frames and system samples are written to the local database in 10 s chunks."
              action={recorder.active ? <Pill tone="err" dot>Recording</Pill> : undefined}
            />
            <div className="border-t border-line px-4 py-3">
              {running ? (
                <div className="flex items-center gap-2">
                  <Button
                    variant={recorder.active ? "danger" : "primary"}
                    size="sm"
                    disabled={busy}
                    onClick={() => void (recorder.active ? stopRecording() : startRecording())}
                  >
                    {busy ? <Spinner /> : recorder.active ? <Square /> : <CircleDot />}
                    {recorder.active ? "Stop and save" : "Record benchmark"}
                  </Button>
                  <span className="text-[11px] text-text-3">
                    {recorder.active ? "The final chunk is flushed on stop" : "Runs until you stop it"}
                  </span>
                </div>
              ) : (
                <Empty
                  title="No capture attached"
                  description="Recording is available only while a process is attached, so every session contains real frame data."
                />
              )}
            </div>
            {recorder.active ? (
              <dl className="grid grid-cols-2 gap-x-4 gap-y-1.5 border-t border-line px-4 py-2.5 text-[11.5px]">
                <div className="flex justify-between">
                  <dt className="text-text-3">On screen</dt>
                  <dd className="tabular-nums">{recorder.displayedFrames.toLocaleString()}</dd>
                </div>
                <div className="flex justify-between">
                  <dt className="text-text-3">Presented</dt>
                  <dd className="tabular-nums">{recorder.presentedFrames.toLocaleString()}</dd>
                </div>
                <div className="flex justify-between">
                  <dt className="text-text-3">Persisted</dt>
                  <dd className="tabular-nums">{recorder.persistedFrames.toLocaleString()}</dd>
                </div>
                <div className="flex justify-between">
                  <dt className="text-text-3">Dropped</dt>
                  <dd className="tabular-nums">{recorder.backpressure.toLocaleString()}</dd>
                </div>
                {snapshot ? (
                  <div className="col-span-2 flex justify-between border-t border-line pt-1.5">
                    <dt className="text-text-3">System right now</dt>
                    <dd className="tabular-nums text-text-2">
                      CPU {snapshot.cpuPercent.value?.toFixed(0) ?? "—"}% · GPU{" "}
                      {snapshot.gpuPercent.value?.toFixed(0) ?? "—"}%
                    </dd>
                  </div>
                ) : null}
              </dl>
            ) : null}
            {recorder.message ? (
              <div className="border-t border-line px-4 py-2 text-[11px] text-warn">{recorder.message}</div>
            ) : null}
          </Panel>
        </div>

        <Panel>
          <PanelHeader
            title="Frame times"
            hint="Step trace of the last 600 displayed frames. The dashed line marks 33.3 ms (30 fps)."
            action={
              <span className="text-[10.5px] tabular-nums text-text-3">
                {series.displayed.length.toLocaleString()} buffered
              </span>
            }
          />
          <div className="border-t border-line px-2 py-2">
            <Oscilloscope frames={series.displayed} />
          </div>
        </Panel>
      </PageBody>
    </>
  );
}
