import { useEffect, useMemo, useState } from "react";
import { Link } from "react-router-dom";
import { History as HistoryIcon, Search, Trash2 } from "lucide-react";
import { PageBody, PageHeader } from "@/components/layout/PageHeader";
import { Panel } from "@/components/ui/panel";
import { Button } from "@/components/ui/button";
import { Modal } from "@/components/ui/modal";
import { Empty, Pill } from "@/components/ui/states";
import { ipc } from "@/lib/ipc";
import { dateTime, duration, fps } from "@/lib/format";
import { useApp } from "@/stores/app";
import type { SessionSummary } from "@/lib/types";

export default function History() {
  const sessions = useApp((state) => state.sessions);
  const refreshSessions = useApp((state) => state.refreshSessions);
  const [query, setQuery] = useState("");
  const [pending, setPending] = useState<SessionSummary | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void refreshSessions();
  }, [refreshSessions]);

  const filtered = useMemo(() => {
    const needle = query.trim().toLowerCase();
    if (!needle) return sessions;
    return sessions.filter(
      (session) =>
        session.gameName.toLowerCase().includes(needle) ||
        session.status.toLowerCase().includes(needle) ||
        dateTime(session.startedAtMs).toLowerCase().includes(needle),
    );
  }, [query, sessions]);

  const remove = async () => {
    if (!pending) return;
    try {
      await ipc.deleteSession(pending.id);
      setPending(null);
      await refreshSessions();
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    }
  };

  return (
    <>
      <PageHeader
        eyebrow={`${sessions.length} stored locally`}
        title="History"
        description="Every session lives in a SQLite file in your user data folder. Nothing leaves this machine."
        actions={
          <div className="relative">
            <Search className="pointer-events-none absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2 text-text-3" />
            <input
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder="Filter sessions"
              aria-label="Filter sessions"
              className="h-8 w-[220px] rounded-[8px] border border-line bg-elevated pl-8 pr-2.5 text-[12.5px] text-text-1 placeholder:text-text-3 focus-visible:border-line-strong"
            />
          </div>
        }
      />
      <PageBody>
        {error ? <div className="text-[12px] text-err">{error}</div> : null}
        <Panel>
          {filtered.length === 0 ? (
            <Empty
              icon={<HistoryIcon />}
              title={sessions.length === 0 ? "No sessions stored yet" : "No session matches that filter"}
              description={
                sessions.length === 0
                  ? "Record a benchmark and it will be listed here, with the ability to re-open, analyse or export it."
                  : "Clear the filter to see every stored session."
              }
            />
          ) : (
            <table className="w-full border-collapse text-[12px]">
              <thead>
                <tr className="border-b border-line text-left text-[10.5px] tracking-[0.05em] text-text-3 uppercase">
                  <th className="px-4 py-2 font-medium">Game</th>
                  <th className="px-3 py-2 font-medium">Recorded</th>
                  <th className="px-3 py-2 text-right font-medium">Length</th>
                  <th className="px-3 py-2 text-right font-medium">Displayed avg</th>
                  <th className="px-3 py-2 text-right font-medium">1% low</th>
                  <th className="px-3 py-2 text-right font-medium">Frames</th>
                  <th className="px-3 py-2 font-medium">Status</th>
                  <th className="px-3 py-2" />
                </tr>
              </thead>
              <tbody>
                {filtered.map((session) => (
                  <tr key={session.id} className="border-b border-line last:border-0 hover:bg-[color-mix(in_oklab,var(--text-1)_3%,transparent)]">
                    <td className="max-w-[220px] truncate px-4 py-2">
                      <Link to={`/results/${session.id}`} className="text-text-1 hover:text-accent">
                        {session.gameName}
                      </Link>
                    </td>
                    <td className="px-3 py-2 whitespace-nowrap text-text-3">{dateTime(session.startedAtMs)}</td>
                    <td className="px-3 py-2 text-right tabular-nums text-text-2">{duration(session.durationMs)}</td>
                    <td className="px-3 py-2 text-right tabular-nums text-text-1">{fps(session.avgFpsDisplayed, 1)}</td>
                    <td className="px-3 py-2 text-right tabular-nums text-text-2">{fps(session.p1LowDisplayed, 1)}</td>
                    <td className="px-3 py-2 text-right tabular-nums text-text-3">
                      {(session.framesDisplayed ?? 0).toLocaleString()}
                    </td>
                    <td className="px-3 py-2">
                      {session.recovered ? (
                        <Pill tone="warn">recovered</Pill>
                      ) : session.status === "completed" ? (
                        <Pill tone="ok">complete</Pill>
                      ) : (
                        <Pill tone="neutral">{session.status}</Pill>
                      )}
                    </td>
                    <td className="px-3 py-2 text-right">
                      <Button
                        variant="ghost"
                        size="iconSm"
                        aria-label={`Delete ${session.gameName}`}
                        onClick={() => setPending(session)}
                      >
                        <Trash2 />
                      </Button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </Panel>
      </PageBody>

      <Modal
        open={pending !== null}
        onOpenChange={(openState) => (openState ? undefined : setPending(null))}
        title="Delete this session?"
        description={
          pending
            ? `${pending.gameName} from ${dateTime(pending.startedAtMs)} and its stored frames will be removed from this machine.`
            : undefined
        }
        footer={
          <>
            <Button variant="ghost" size="sm" onClick={() => setPending(null)}>
              Keep
            </Button>
            <Button variant="danger" size="sm" onClick={() => void remove()}>
              Delete session
            </Button>
          </>
        }
      />
    </>
  );
}
