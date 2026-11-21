import { create } from "zustand";
import { ipc, isDesktop, onTick } from "@/lib/ipc";
import type {
  Bootstrap, FrameCaptureStatus, FrameTick, GameEntry, RecorderState, SessionSummary,
  Settings, Snapshot,
} from "@/lib/types";

type Status = "idle" | "loading" | "ready" | "error";

const EMPTY_CAPTURE: FrameCaptureStatus = {
  state: "idle", pid: null, process: null, message: null,
  elevation: "unknown", sidecarPresent: false, sidecarPath: null,
};

const EMPTY_RECORDER: RecorderState = {
  active: false, sessionId: null, pid: null, process: null, startedAtMs: null,
  displayedFrames: 0, presentedFrames: 0, persistedFrames: 0, backpressure: 0, message: null,
};

export function applyTheme(settings: Settings) {
  const root = document.documentElement;
  root.classList.toggle("light", settings.theme === "light");
  root.dataset.accent = settings.accent;
  root.dataset.motion = settings.reduceMotion ? "reduced" : "full";
  root.dataset.density = settings.compact ? "compact" : "comfortable";
}

type AppStore = {
  status: Status;
  error: string | null;
  bootstrap: Bootstrap | null;
  settings: Settings | null;
  snapshot: Snapshot | null;
  frame: FrameTick | null;
  capture: FrameCaptureStatus;
  recorder: RecorderState;
  series: { displayed: number[]; presented: number[] };
  sessions: SessionSummary[];
  games: GameEntry[];
  init: () => Promise<void>;
  refreshSessions: () => Promise<void>;
  refreshGames: () => Promise<void>;
  pollFrames: () => Promise<void>;
  saveSettings: (next: Settings) => Promise<void>;
  patchSettings: (patch: Partial<Settings>) => Promise<void>;
  setCapture: (status: FrameCaptureStatus) => void;
  setRecorder: (state: RecorderState) => void;
};

let subscribed = false;

export const useApp = create<AppStore>((set, get) => ({
  status: "idle",
  error: null,
  bootstrap: null,
  settings: null,
  snapshot: null,
  frame: null,
  capture: EMPTY_CAPTURE,
  recorder: EMPTY_RECORDER,
  series: { displayed: [], presented: [] },
  sessions: [],
  games: [],

  init: async () => {
    if (get().status === "loading" || get().status === "ready") return;
    set({ status: "loading", error: null });
    try {
      const bootstrap = await ipc.bootstrap();
      set({
        status: "ready",
        bootstrap,
        settings: bootstrap.settings,
        capture: bootstrap.capture,
        recorder: bootstrap.recorder,
        sessions: bootstrap.sessions,
      });
      applyTheme(bootstrap.settings);
      if (!subscribed) {
        subscribed = true;
        await onTick((payload) => {
          set((state) => ({
            snapshot: payload.snapshot,
            frame: payload.frame ?? state.frame,
          }));
        });
      }
      void get().refreshGames();
      void get().refreshSessions();
    } catch (error) {
      set({ status: "error", error: error instanceof Error ? error.message : String(error) });
    }
  },

  refreshSessions: async () => {
    try {
      set({ sessions: await ipc.sessions(60, 0) });
    } catch {
      /* the list keeps its previous value when the read fails */
    }
  },

  refreshGames: async () => {
    try {
      set({ games: await ipc.games() });
    } catch {
      /* same: no destructive fallback */
    }
  },

  pollFrames: async () => {
    try {
      const live = await ipc.liveFrames(1600);
      set({
        series: { displayed: live.displayed, presented: live.presented },
        capture: live.status,
        recorder: live.recorder,
        frame: live.frame ?? get().frame,
      });
    } catch {
      /* transient: the next poll retries */
    }
  },

  saveSettings: async (next) => {
    const applied = await ipc.updateSettings(next);
    set({ settings: applied });
    applyTheme(applied);
  },

  patchSettings: async (patch) => {
    const current = get().settings;
    if (!current) return;
    await get().saveSettings({ ...current, ...patch });
  },

  setCapture: (capture) => set({ capture }),
  setRecorder: (recorder) => set({ recorder }),
}));

export const desktopAvailable = isDesktop;
