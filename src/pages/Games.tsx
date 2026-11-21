import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { Gamepad2, Play, Plus, RefreshCw, Trash2 } from "lucide-react";
import { PageBody, PageHeader } from "@/components/layout/PageHeader";
import { Panel, PanelHeader, SectionLabel } from "@/components/ui/panel";
import { Button } from "@/components/ui/button";
import { Empty, Pill, Spinner } from "@/components/ui/states";
import { Tooltip } from "@/components/ui/tooltip";
import { ipc } from "@/lib/ipc";
import { fps } from "@/lib/format";
import { useApp } from "@/stores/app";
import type { DetectedGame } from "@/lib/types";

export default function Games() {
  const games = useApp((state) => state.games);
  const refreshGames = useApp((state) => state.refreshGames);
  const sessions = useApp((state) => state.sessions);
  const [detected, setDetected] = useState<DetectedGame[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const scan = async () => {
    setBusy(true);
    setError(null);
    try {
      setDetected(await ipc.detectGames());
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  };

  useEffect(() => {
    void scan();
  }, []);

  const importDetected = async () => {
    setBusy(true);
    try {
      await ipc.importGames();
      await refreshGames();
      setNotice("Detected games added to your library.");
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  };

  const addManually = async () => {
    const path = await open({
      title: "Choose a game executable",
      multiple: false,
      filters: [{ name: "Executables", extensions: ["exe"] }],
    });
    if (!path || Array.isArray(path)) return;
    setBusy(true);
    try {
      await ipc.addGame(path);
      await refreshGames();
      setNotice("Executable added.");
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  };

  const launch = async (id: number) => {
    setError(null);
    try {
      await ipc.launchGame(id);
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    }
  };

  const remove = async (id: number) => {
    try {
      await ipc.removeGame(id);
      await refreshGames();
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    }
  };

  const missing = detected.filter(
    (entry) => !games.some((game) => game.name.toLowerCase() === entry.name.toLowerCase()),
  );

  return (
    <>
      <PageHeader
        eyebrow="Local discovery"
        title="Games"
        description="Steam and Epic installs are read from their manifests on this machine. Launching only ever runs an executable that is already in this library."
        actions={
          <>
            <Button variant="secondary" size="sm" onClick={() => void addManually()} disabled={busy}>
              <Plus /> Add executable
            </Button>
            <Button variant="primary" size="sm" onClick={() => void scan()} disabled={busy}>
              {busy ? <Spinner /> : <RefreshCw />} Rescan
            </Button>
          </>
        }
      />
      <PageBody>
        {error ? <div className="text-[12px] text-err">{error}</div> : null}
        {notice ? <div className="text-[12px] text-ok">{notice}</div> : null}

        <SectionLabel>Library</SectionLabel>
        {games.length === 0 ? (
          <Panel>
            <Empty
              icon={<Gamepad2 />}
              title="Your library is empty"
              description="Import what FrameLab found on this machine, or add a single executable. Nothing is uploaded anywhere."
            />
          </Panel>
        ) : (
          <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
            {games.map((game) => {
              const runs = sessions.filter((session) => session.gameId === game.id).length;
              return (
                <Panel key={game.id} className="flex flex-col">
                  <div className="flex items-start gap-2.5 px-4 pt-3.5">
                    <span className="grid size-8 shrink-0 place-items-center rounded-[8px] border border-line bg-elevated text-[12px] font-medium text-text-2">
                      {game.name.slice(0, 1).toUpperCase()}
                    </span>
                    <div className="min-w-0 flex-1">
                      <div className="truncate text-[12.5px] text-text-1">{game.name}</div>
                      <div className="mt-0.5 flex items-center gap-1.5">
                        <Pill tone="neutral">{game.source}</Pill>
                        {game.launcherId ? <span className="text-[10px] text-text-3">{game.launcherId}</span> : null}
                      </div>
                    </div>
                  </div>
                  <div className="mt-2 flex items-baseline justify-between px-4 text-[11px] text-text-3">
                    <span>
                      {game.sessionCount || runs} sessions
                    </span>
                    <span className="tabular-nums">
                      best {fps(game.bestAvgFps, 1)} fps
                    </span>
                  </div>
                  <div className="mt-3 flex items-center gap-1.5 border-t border-line px-3 py-2">
                    <Button variant="secondary" size="xs" onClick={() => void launch(game.id)}>
                      <Play /> Launch
                    </Button>
                    <Tooltip content={game.exePath ?? "No executable recorded"}>
                      <span className="min-w-0 flex-1 truncate font-mono text-[10px] text-text-3">
                        {game.exePath ?? "launcher only"}
                      </span>
                    </Tooltip>
                    <Tooltip content="Remove from library">
                      <Button variant="ghost" size="iconSm" onClick={() => void remove(game.id)} aria-label={`Remove ${game.name}`}>
                        <Trash2 />
                      </Button>
                    </Tooltip>
                  </div>
                </Panel>
              );
            })}
          </div>
        )}

        <SectionLabel>Found on this machine</SectionLabel>
        <Panel>
          <PanelHeader
            title={`${detected.length} installed games detected`}
            hint="Read from Steam libraryfolders/ACF files and Epic launcher manifests."
            action={
              missing.length ? (
                <Button variant="secondary" size="sm" onClick={() => void importDetected()} disabled={busy}>
                  Import {missing.length}
                </Button>
              ) : undefined
            }
          />
          {detected.length === 0 ? (
            <Empty
              title={busy ? "Scanning launchers…" : "No launcher libraries found"}
              description="FrameLab looks in the default Steam and Epic locations. You can still add an executable directly."
            />
          ) : (
            <ul className="max-h-[280px] divide-y divide-line overflow-y-auto border-t border-line">
              {detected.map((entry) => (
                <li key={`${entry.source}-${entry.launcherId}`} className="flex items-center gap-3 px-4 py-2">
                  <span className="min-w-0 flex-1 truncate text-[12px] text-text-1">{entry.name}</span>
                  <span className="text-[10.5px] text-text-3">{entry.source}</span>
                  <span
                    className={`text-[10.5px] ${
                      games.some((game) => game.name.toLowerCase() === entry.name.toLowerCase())
                        ? "text-ok"
                        : "text-text-3"
                    }`}
                  >
                    {games.some((game) => game.name.toLowerCase() === entry.name.toLowerCase()) ? "in library" : "not imported"}
                  </span>
                </li>
              ))}
            </ul>
          )}
        </Panel>
      </PageBody>
    </>
  );
}
