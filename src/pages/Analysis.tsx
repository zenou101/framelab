import { useEffect, useState } from "react";
import { useSearchParams } from "react-router-dom";
import { CheckCircle2, Stethoscope, TriangleAlert } from "lucide-react";
import { PageBody, PageHeader } from "@/components/layout/PageHeader";
import { Panel, PanelHeader, SectionLabel } from "@/components/ui/panel";
import { Select } from "@/components/ui/select";
import { Empty, Pill, Spinner } from "@/components/ui/states";
import { ipc } from "@/lib/ipc";
import { dateTime, duration } from "@/lib/format";
import { useApp } from "@/stores/app";
import type { AnalysisReport, Finding } from "@/lib/types";

const TONE: Record<string, "ok" | "warn" | "err" | "neutral"> = {
  info: "neutral",
  low: "neutral",
  medium: "warn",
  high: "err",
  critical: "err",
};

function FindingCard({ finding }: { finding: Finding }) {
  return (
    <li className="px-4 py-3">
      <div className="flex items-start gap-2.5">
        {finding.severity === "info" ? (
          <CheckCircle2 className="mt-0.5 size-3.5 shrink-0 text-ok" />
        ) : (
          <TriangleAlert className={`mt-0.5 size-3.5 shrink-0 ${finding.severity === "high" || finding.severity === "critical" ? "text-err" : "text-warn"}`} />
        )}
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-2">
            <span className="text-[12.5px] font-medium text-text-1">{finding.title}</span>
            <Pill tone={TONE[finding.severity] ?? "neutral"}>{finding.severity}</Pill>
            <Pill tone={finding.confidence === "measured" ? "accent" : "neutral"}>{finding.confidence}</Pill>
            {finding.heuristic ? <span className="text-[10px] text-text-3">heuristic (configurable)</span> : null}
          </div>
          <p className="mt-1 text-[12px] leading-relaxed text-text-2">{finding.explanation}</p>
          {finding.evidence.length ? (
            <ul className="mt-1.5 space-y-0.5">
              {finding.evidence.map((line) => (
                <li key={line} className="font-mono text-[11px] text-text-3">
                  {line}
                </li>
              ))}
            </ul>
          ) : null}
          <p className="mt-1.5 text-[11.5px] text-text-2">
            <span className="text-text-3">Next step: </span>
            {finding.nextStep}
          </p>
        </div>
      </div>
    </li>
  );
}

export default function Analysis() {
  const sessions = useApp((state) => state.sessions);
  const [params] = useSearchParams();
  const [sessionId, setSessionId] = useState<string | null>(params.get("s"));
  const [report, setReport] = useState<AnalysisReport | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!sessionId && sessions.length > 0) setSessionId(String(sessions[0].id));
  }, [sessionId, sessions]);

  useEffect(() => {
    if (!sessionId) return;
    setBusy(true);
    setError(null);
    ipc
      .analyze(Number(sessionId))
      .then(setReport)
      .catch((reason: unknown) => setError(reason instanceof Error ? reason.message : String(reason)))
      .finally(() => setBusy(false));
  }, [sessionId]);

  const selected = sessions.find((session) => String(session.id) === sessionId) ?? null;

  return (
    <>
      <PageHeader
        eyebrow="Heuristics, not verdicts"
        title="Analysis"
        description="Findings compare measured frames and system samples against configurable thresholds. They describe what was measured and never claim a definitive cause."
        actions={
          <div className="w-[240px]">
            <Select
              value={sessionId}
              onValueChange={setSessionId}
              options={sessions.map((session) => ({
                value: String(session.id),
                label: session.gameName,
                hint: dateTime(session.startedAtMs),
              }))}
              placeholder="Choose a session"
              size="sm"
            />
          </div>
        }
      />
      <PageBody>
        {sessions.length === 0 ? (
          <Panel>
            <Empty
              icon={<Stethoscope />}
              title="No sessions to analyse"
              description="Analysis runs on a recorded benchmark, because it needs both frame times and system samples."
            />
          </Panel>
        ) : null}

        {selected ? (
          <div className="text-[11.5px] text-text-3">
            {selected.gameName} · {dateTime(selected.startedAtMs)} · {duration(selected.durationMs)} ·{" "}
            {selected.framesDisplayed?.toLocaleString() ?? 0} displayed frames
          </div>
        ) : null}

        {busy ? (
          <div className="flex items-center gap-2 text-[12px] text-text-3">
            <Spinner /> Comparing frames and samples…
          </div>
        ) : null}
        {error ? <div className="text-[12px] text-err">{error}</div> : null}

        {report && !busy ? (
          <>
            <Panel>
              <PanelHeader
                title="Findings"
                hint={
                  report.findings.length
                    ? `${report.findings.length} observations ordered by how directly they are measured`
                    : "Nothing exceeded the configured thresholds in this session"
                }
              />
              {report.findings.length === 0 ? (
                <Empty
                  icon={<CheckCircle2 />}
                  title="No threshold was crossed"
                  description="That is not proof the run was perfect, only that no measured signal crossed the configured limits."
                />
              ) : (
                <ul className="divide-y divide-line border-t border-line">
                  {report.findings.map((finding) => (
                    <FindingCard key={finding.code} finding={finding} />
                  ))}
                </ul>
              )}
            </Panel>

            {report.notEvaluated.length ? (
              <>
                <SectionLabel>Could not be evaluated</SectionLabel>
                <Panel>
                  <ul className="space-y-1.5 px-4 py-3">
                    {report.notEvaluated.map((line) => (
                      <li key={line} className="flex items-start gap-2 text-[12px] text-text-2">
                        <span className="mt-[6px] size-1 shrink-0 rounded-full bg-text-3" />
                        {line}
                      </li>
                    ))}
                  </ul>
                </Panel>
              </>
            ) : null}

            <SectionLabel>Thresholds used</SectionLabel>
            <Panel>
              <dl className="grid grid-cols-2 gap-x-6 gap-y-1.5 px-4 py-3 text-[11.5px] md:grid-cols-3">
                {Object.entries(report.thresholds).map(([key, value]) => (
                  <div key={key} className="flex items-baseline justify-between gap-3">
                    <dt className="truncate text-text-3">{key}</dt>
                    <dd className="tabular-nums text-text-2">{String(value)}</dd>
                  </div>
                ))}
              </dl>
              <div className="border-t border-line px-4 py-2 text-[11px] text-text-3">
                Edit these in Settings → Diagnostics thresholds. Raising a limit makes findings rarer, not the run faster.
              </div>
            </Panel>
          </>
        ) : null}
      </PageBody>
    </>
  );
}
