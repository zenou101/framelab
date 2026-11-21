import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { save } from "@tauri-apps/plugin-dialog";
import { Download, History, Stethoscope } from "lucide-react";
import { PageBody, PageHeader } from "@/components/layout/PageHeader";
import { Panel, PanelHeader, SectionLabel } from "@/components/ui/panel";
import { Button } from "@/components/ui/button";
import { Empty, Pill, Skeleton } from "@/components/ui/states";
import FrameChart from "@/components/charts/FrameChart";
import HistogramChart from "@/components/charts/HistogramChart";
import SampleChart from "@/components/charts/SampleChart";
import { ipc } from "@/lib/ipc";
import { dateTime, duration, fps, ms, shortName } from "@/lib/format";
import { useApp } from "@/stores/app";
import type { FrameStats, SessionDetail } from "@/lib/types";

function StatRows({ stats }: { stats: FrameStats }) {
  const rows: [string, string][] = [
    ["Frames", stats.frames.toLocaleString()],
    ["Average FPS", fps(stats.avgFps, 1)],
    ["Median FPS", fps(stats.medianFps, 1)],
    ["1% low", stats.insufficientSamples ? `${fps(stats.p1Low, 1)} (fewer than 300 frames)` : fps(stats.p1Low, 1)],
    ["0.1% low", fps(stats.p01Low, 1)],
    ["Average frame", ms(stats.avgFrameMs)],
    ["p95 frame", ms(stats.p95FrameMs)],
    ["p99 frame", ms(stats.p99FrameMs)],
    ["Longest frame", ms(stats.maxFrameMs)],
    ["Stutters", `${stats.stutterCount} · ${ms(stats.stutterMsTotal, 0)} lost`],
    ["Variation (CV)", stats.cv === null ? "—" : stats.cv.toFixed(2)],
  ];
  return (
    <dl className="divide-y divide-line text-[12px]">
      {rows.map(([label, value]) => (
        <div key={label} className="flex items-baseline justify-between gap-4 px-4 py-1.5">
          <dt className="text-text-3">{label}</dt>
          <dd className="tabular-nums text-text-1">{value}</dd>
        </div>
      ))}
    </dl>
  );
}

export default function Results() {
  const { id } = useParams();
  const sessionId = id ? Number(id) : null;
  const sessions = useApp((state) => state.sessions);
  const exportFormat = useApp((state) => state.settings?.exportFormat ?? "csv");
  const [detail, setDetail] = useState<SessionDetail | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const target = sessionId ?? sessions[0]?.id ?? null;

  useEffect(() => {
    if (!target) return;
    setLoading(true);
    setError(null);
    ipc
      .session(target)
      .then(setDetail)
      .catch((reason: unknown) => setError(reason instanceof Error ? reason.message : String(reason)))
      .finally(() => setLoading(false));
  }, [target]);

  const exportSession = async () => {
    if (!target) return;
    const extension = exportFormat === "json" ? "json" : "csv";
    const path = await save({
      title: "Export session",
      defaultPath: `framelab-session-${target}.${extension}`,
      filters: [{ name: extension.toUpperCase(), extensions: [extension] }],
    });
    if (!path) return;
    try {
      const written = await ipc.exportSession(target, path, extension);
      setNotice(`Exported to ${written}`);
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    }
  };

  if (!target) {
    return (
      <>
        <PageHeader eyebrow="Results" title="No session selected" />
        <PageBody>
          <Panel>
            <Empty
              icon={<History />}
              title="Nothing to show yet"
              description="Record a benchmark and its result opens here automatically."
              action={
                <Button variant="secondary" size="sm" asChild>
                  <Link to="/live">Open Live Monitor</Link>
                </Button>
              }
            />
          </Panel>
        </PageBody>
      </>
    );
  }

  const summary = detail?.summary;

  return (
    <>
      <PageHeader
        eyebrow={summary ? dateTime(summary.startedAtMs) : "Results"}
        title={summary ? shortName(summary.gameName, 48) : "Loading session"}
        description={
          summary
            ? `${duration(summary.durationMs)} recorded · headline source: ${summary.primarySource}`
            : "Reading the stored frames and samples from the local database."
        }
        actions={
          <>
            <Button variant="secondary" size="sm" asChild>
              <Link to={`/analysis?s=${target}`}>
                <Stethoscope /> Analyse
              </Link>
            </Button>
            <Button variant="primary" size="sm" onClick={() => void exportSession()}>
              <Download /> Export {exportFormat.toUpperCase()}
            </Button>
          </>
        }
      />

      <PageBody>
        {summary?.recovered ? (
          <div className="rounded-[10px] border border-[color-mix(in_oklab,var(--warn)_32%,transparent)] bg-[color-mix(in_oklab,var(--warn)_8%,transparent)] px-3 py-2 text-[12px] text-warn">
            This session was recovered after an unexpected exit. It contains the chunks that had already been flushed to disk.
          </div>
        ) : null}
        {error ? <div className="text-[12px] text-err">{error}</div> : null}
        {notice ? <div className="text-[12px] text-ok">{notice}</div> : null}

        {loading && !detail ? (
          <div className="grid gap-3 lg:grid-cols-2">
            <Skeleton className="h-[280px]" />
            <Skeleton className="h-[280px]" />
          </div>
        ) : null}

        {detail ? (
          <>
            <div className="grid gap-3 lg:grid-cols-2">
              <Panel>
                <PanelHeader
                  title="Displayed frames"
                  hint="Measured from MsBetweenDisplayChange — what reached the screen."
                  action={<Pill tone="accent">headline</Pill>}
                />
                <div className="border-t border-line">
                  <StatRows stats={detail.stats.displayed} />
                </div>
              </Panel>
              <Panel>
                <PanelHeader
                  title="Presented frames"
                  hint="Measured from MsBetweenPresents — the CPU present cadence."
                />
                <div className="border-t border-line">
                  <StatRows stats={detail.stats.presented} />
                </div>
              </Panel>
            </div>

            <SectionLabel>Frame time trace</SectionLabel>
            <Panel>
              <PanelHeader
                title="Frame times"
                hint="Step trace of the recorded displayed frames; dashed lines are p95 and p99."
              />
              <div className="border-t border-line px-2 py-2">
                <FrameChart
                  frames={detail.frames}
                  p95={detail.stats.displayed.p95FrameMs}
                  p99={detail.stats.displayed.p99FrameMs}
                  height={240}
                />
              </div>
            </Panel>

            <div className="grid gap-3 lg:grid-cols-2">
              <Panel>
                <PanelHeader title="Frame time distribution" hint="Binned displayed frame times." />
                <div className="border-t border-line px-2 py-2">
                  <HistogramChart bins={detail.histogram} height={200} />
                </div>
              </Panel>
              <Panel>
                <PanelHeader title="System during the run" hint="Sampled with the same clock as the frames." />
                <div className="border-t border-line px-2 py-2">
                  <SampleChart series={detail.sampleSeries} height={200} />
                </div>
              </Panel>
            </div>
          </>
        ) : null}
      </PageBody>
    </>
  );
}
