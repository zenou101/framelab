import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { CircleDot, Square, Timer } from "lucide-react";
import { PageBody, PageHeader } from "@/components/layout/PageHeader";
import { Panel, PanelHeader, SectionLabel } from "@/components/ui/panel";
import { Button } from "@/components/ui/button";
import { Select } from "@/components/ui/select";
import { Empty, Spinner } from "@/components/ui/states";
import { ipc } from "@/lib/ipc";
import { fps } from "@/lib/format";
import { useApp } from "@/stores/app";
import type { ProcessInfo } from "@/lib/types";

export default function Benchmarks() {
  const games = useApp((state) => state.games);
  const sessions = useApp((state) => state.sessions);
  const capture = useApp((state) => state.capture);
  const recorder = useApp((state) => state.recorder);
  const primarySource = useApp((state) => state.settings?.primarySource ?? "displayed");
  const setCapture = useApp((state) => state.setCapture);
  const setRecorder = useApp((state) => state.setRecorder);
  const refreshSessions = useApp((state) => state.refreshSessions);
  const [processes, setProcesses] = useState<ProcessInfo[]>([]);
  const [pid, setPid] = useState<number | null>(null);
  const [gameId, setGameId] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const navigate = useNavigate();

  useEffect(() => {
    ipc
      .processes()
      .then((list) => {
        setProcesses(list);
        setPid((current) => current ?? list.find((entry) => entry.isForeground)?.pid ?? list[0]?.pid ?? null);
      })
      .catch((reason: unknown) => setError(reason instanceof Error ? reason.message : String(reason)));
  }, []);

  const selected = processes.find((entry) => entry.pid === pid) ?? null;
  const game = games.find((entry) => String(entry.id) === gameId) ?? null;

  const attach = async () => {
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

  const record = async () => {
    if (!selected) return;
    setBusy(true);
    setError(null);
    try {
      const name = game?.name ?? selected.name.replace(/\.exe$/i, "");
      setRecorder(await ipc.startRecording(name, game?.id ?? null));
      navigate("/live");
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  };

  const stop = async () => {
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

  return (
    <>
      <PageHeader
        eyebrow="Capture a session"
        title="Benchmarks"
        description="Attach to a running game, record, then stop to store the session. Every number below is computed from the frames that were actually captured."
      />
      <PageBody>
        <div className="grid gap-3 lg:grid-cols-[1.2fr_1fr]">
          <Panel>
            <PanelHeader title="New session" hint="The game must already be running." />
            <div className="space-y-2.5 border-t border-line px-4 py-3">
              <label className="block">
                <span className="mb-1 block text-[11px] text-text-3">Game (optional, links the run to your library)</span>
                <Select
                  value={gameId}
                  onValueChange={setGameId}
                  options={games.map((entry) => ({
                    value: String(entry.id),
                    label: entry.name,
                    hint: `${entry.sessionCount} sessions`,
                  }))}
                  placeholder="No game selected"
                  size="sm"
                />
              </label>
              <label className="block">
                <span className="mb-1 block text-[11px] text-text-3">Running process</span>
                <Select
                  value={pid === null ? null : String(pid)}
                  onValueChange={(value) => setPid(Number(value))}
                  options={processes.slice(0, 160).map((entry) => ({
                    value: String(entry.pid),
                    label: `${entry.name} · ${entry.pid}`,
                    hint: entry.isForeground ? "foreground" : `${entry.cpuPercent.toFixed(1)}% CPU`,
                  }))}
                  placeholder="Select a process"
                  size="sm"
                />
              </label>
              <div className="flex items-center gap-2 pt-1">
                {recorder.active ? (
                  <Button variant="danger" size="sm" onClick={() => void stop()} disabled={busy}>
                    {busy ? <Spinner /> : <Square />} Stop and save
                  </Button>
                ) : capture.state === "running" ? (
                  <Button variant="primary" size="sm" onClick={() => void record()} disabled={busy}>
                    {busy ? <Spinner /> : <CircleDot />} Record benchmark
                  </Button>
                ) : (
                  <Button variant="primary" size="sm" onClick={() => void attach()} disabled={busy || !selected}>
                    {busy ? <Spinner /> : <Timer />} Attach capture
                  </Button>
                )}
                <span className="text-[11px] text-text-3">
                  {capture.state === "running" ? `Attached to ${capture.process}` : "Capture not attached"}
                </span>
              </div>
              {error ? <p className="text-[11.5px] text-err">{error}</p> : null}
            </div>
          </Panel>

          <Panel>
            <PanelHeader title="How the numbers are produced" hint="Documented, not inferred." />
            <ul className="space-y-2 border-t border-line px-4 py-3 text-[11.5px] leading-relaxed text-text-2">
              <li><span className="text-text-1">Average FPS</span> — captured frames ÷ elapsed capture time.</li>
              <li><span className="text-text-1">Median FPS</span> — 1000 ÷ median frame time.</li>
              <li><span className="text-text-1">1% low</span> — 1000 ÷ mean of the slowest 1% of frames; needs at least 300 frames.</li>
              <li><span className="text-text-1">0.1% low</span> — same method on the slowest 0.1%; needs at least 1000 frames.</li>
              <li><span className="text-text-1">Stutter</span> — a frame longer than max(2 × median, 33.3 ms).</li>
              <li className="text-text-3">
                This release reports <span className="text-text-2">{primarySource}</span> as the headline source; both displayed and
                presented series are always stored and shown separately.
              </li>
            </ul>
          </Panel>
        </div>

        <SectionLabel>Previous results</SectionLabel>
        <Panel>
          {sessions.length === 0 ? (
            <Empty icon={<Timer />} title="Nothing recorded yet" description="Your first finished session will appear here." />
          ) : (
            <ul className="divide-y divide-line">
              {sessions.slice(0, 6).map((session) => (
                <li key={session.id}>
                  <button
                    type="button"
                    onClick={() => navigate(`/results/${session.id}`)}
                    className="flex w-full items-center gap-4 px-4 py-2.5 text-left hover:bg-[color-mix(in_oklab,var(--text-1)_4%,transparent)]"
                  >
                    <span className="min-w-0 flex-1 truncate text-[12.5px]">{session.gameName}</span>
                    <span className="w-16 text-right text-[12px] tabular-nums">{fps(session.avgFpsDisplayed, 1)} fps</span>
                  </button>
                </li>
              ))}
            </ul>
          )}
        </Panel>
      </PageBody>
    </>
  );
}
