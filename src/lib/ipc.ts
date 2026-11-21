import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AnalysisReport, Bootstrap, DataStats, DetectedGame, FrameCaptureStatus, GameEntry,
  HardwareInfo, LiveFrames, ProcessInfo, RecorderState, SelfMetrics, SessionDetail,
  SessionSummary, Settings, TickPayload,
} from "./types";

export const isDesktop = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!isDesktop) {
    throw new Error(
      "This is a browser preview. FrameLab reads real sensors and frame timings only inside the desktop window.",
    );
  }
  return invoke<T>(command, args);
}

export const ipc = {
  bootstrap: () => call<Bootstrap>("bootstrap"),
  liveFrames: (max = 1600) => call<LiveFrames>("live_frames", { max }),
  hardware: () => call<HardwareInfo>("hardware_info"),
  processes: () => call<ProcessInfo[]>("list_processes"),
  startCapture: (pid: number, process: string) =>
    call<FrameCaptureStatus>("start_capture", { pid, process }),
  stopCapture: () => call<FrameCaptureStatus>("stop_capture"),
  captureStatus: () => call<FrameCaptureStatus>("capture_status"),
  startRecording: (gameName: string, gameId: number | null) =>
    call<RecorderState>("start_recording", { gameName, gameId }),
  stopRecording: () => call<number | null>("stop_recording"),
  sessions: (limit = 50, offset = 0, search?: string) =>
    call<SessionSummary[]>("list_sessions", { limit, offset, search: search ?? null }),
  session: (id: number) => call<SessionDetail | null>("session_detail", { id }),
  deleteSession: (id: number) => call<void>("delete_session", { id }),
  clearSessions: () => call<void>("clear_sessions"),
  analyze: (id: number) => call<AnalysisReport | null>("analyze", { id }),
  exportSession: (id: number, path: string, format: string) =>
    call<string>("export_session", { id, path, format }),
  games: () => call<GameEntry[]>("list_games"),
  detectGames: () => call<DetectedGame[]>("detect_games"),
  importGames: () => call<GameEntry[]>("import_games"),
  addGame: (path: string) => call<GameEntry[]>("add_game_manual", { path }),
  removeGame: (id: number) => call<GameEntry[]>("remove_game", { id }),
  launchGame: (id: number) => call<void>("launch_game", { id }),
  settings: () => call<Settings>("get_settings"),
  updateSettings: (settings: Settings) => call<Settings>("update_settings", { settings }),
  dataStats: () => call<DataStats>("data_stats"),
  applyRetention: () => call<number>("apply_retention"),
  selfMetrics: () => call<SelfMetrics>("self_metrics"),
  setAutostart: (enabled: boolean) => call<boolean>("set_autostart", { enabled }),
  restartElevated: () => call<void>("restart_elevated"),
  openDataDir: () => call<void>("open_data_dir"),
};

export function onTick(handler: (payload: TickPayload) => void): Promise<UnlistenFn> {
  if (!isDesktop) return Promise.resolve(() => undefined);
  return listen<TickPayload>("framelab://tick", (event) => handler(event.payload));
}
