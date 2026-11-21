export type Metric = {
  value: number | null;
  available: boolean;
  source: string;
  method: string | null;
  reason: string | null;
};

export type Snapshot = {
  tsMs: number;
  samplingMs: number;
  cpuPercent: Metric;
  cpuClockMhz: Metric;
  cpuPerCore: number[];
  ramPercent: Metric;
  ramUsedBytes: Metric;
  ramTotalBytes: Metric;
  gpuPercent: Metric;
  vramUsedBytes: Metric;
  vramTotalBytes: Metric;
  diskBytesPerSec: Metric;
  netBytesPerSec: Metric;
  cpuTempC: Metric;
  gpuTempC: Metric;
  frameSourceReady: boolean;
};

export type DiskInfo = { mount: string; totalBytes: number; freeBytes: number };
export type DisplayInfo = { width: number; height: number; refreshHz: number };

export type HardwareInfo = {
  cpuName: string;
  cpuCores: number;
  cpuThreads: number;
  gpuName: string;
  vramTotalBytes: number | null;
  ramTotalBytes: number;
  osName: string;
  osVersion: string;
  osBuild: string;
  disks: DiskInfo[];
  display: DisplayInfo | null;
  adapterLuid: string | null;
  notes: string[];
  collectedAtMs: number;
};

export type ProcessInfo = {
  pid: number;
  name: string;
  exePath: string | null;
  memoryBytes: number;
  cpuPercent: number;
  isForeground: boolean;
};

export type FrameTick = {
  tsMs: number;
  displayedFps: number | null;
  presentedFps: number | null;
  displayedFrameMs: number | null;
  presentedFrameMs: number | null;
  pid: number;
  process: string;
  presentMode: string | null;
  runtime: string | null;
  syncInterval: number | null;
  capturedDisplayed: number;
  capturedPresented: number;
};

export type FrameCaptureStatus = {
  state: string;
  pid: number | null;
  process: string | null;
  message: string | null;
  elevation: string;
  sidecarPresent: boolean;
  sidecarPath: string | null;
};

export type SessionSummary = {
  id: number;
  gameId: number | null;
  gameName: string;
  startedAtMs: number;
  endedAtMs: number | null;
  durationMs: number;
  status: string;
  primarySource: string;
  framesDisplayed: number | null;
  framesPresented: number | null;
  avgFpsDisplayed: number | null;
  avgFpsPresented: number | null;
  medianFpsDisplayed: number | null;
  p1LowDisplayed: number | null;
  p01LowDisplayed: number | null;
  p1LowPresented: number | null;
  stutterCount: number | null;
  recovered: boolean;
  hardwareJson: string | null;
  metaJson: string | null;
  notes: string | null;
};

export type FrameStats = {
  frames: number;
  durationMs: number;
  avgFps: number | null;
  medianFps: number | null;
  p1Low: number | null;
  p01Low: number | null;
  avgFrameMs: number | null;
  medianFrameMs: number | null;
  p95FrameMs: number | null;
  p99FrameMs: number | null;
  minFrameMs: number | null;
  maxFrameMs: number | null;
  cv: number | null;
  stutterCount: number;
  stutterMsTotal: number;
  insufficientSamples: boolean;
};

export type HistogramBin = { fromMs: number; toMs: number; count: number };
export type SampleSeries = {
  tMs: number[];
  cpu: number[];
  gpu: number[];
  ram: number[];
  vram: number[];
  cpuTemp: number[];
  gpuTemp: number[];
};

export type SessionDetail = {
  summary: SessionSummary;
  stats: { displayed: FrameStats; presented: FrameStats };
  histogram: HistogramBin[];
  sampleSeries: SampleSeries;
  frames: number[];
};

export type RecorderState = {
  active: boolean;
  sessionId: number | null;
  pid: number | null;
  process: string | null;
  startedAtMs: number | null;
  displayedFrames: number;
  presentedFrames: number;
  persistedFrames: number;
  backpressure: number;
  message: string | null;
};

export type Finding = {
  code: string;
  severity: string;
  title: string;
  explanation: string;
  nextStep: string;
  confidence: string;
  evidence: string[];
  heuristic: boolean;
};

export type Thresholds = {
  pacingCv: number;
  pacingP99Ratio: number;
  spikeMultiplier: number;
  spikeFloorMs: number;
  spikeClusterWindowMs: number;
  spikeClusterMin: number;
  ramPressurePct: number;
  vramPressurePct: number;
  gpuDipPoints: number;
  singleCorePct: number;
  gpuUnderutilizedPct: number;
  backgroundCpuPct: number;
  thermalLimitC: number;
};

export type AnalysisReport = {
  findings: Finding[];
  notEvaluated: string[];
  thresholds: Thresholds;
};

export type GameEntry = {
  id: number;
  name: string;
  exePath: string | null;
  source: string;
  launcherId: string | null;
  iconPath: string | null;
  createdAtMs: number;
  sessionCount: number;
  bestAvgFps: number | null;
};

export type OverlaySettings = {
  enabled: boolean;
  position: string;
  opacity: number;
  compact: boolean;
  showFps: boolean;
  showFrameTime: boolean;
  showCpu: boolean;
  showGpu: boolean;
  showGpuTemp: boolean;
  showRam: boolean;
};

export type Settings = {
  theme: string;
  accent: string;
  compact: boolean;
  reduceMotion: boolean;
  samplingMs: number;
  backgroundMonitoring: boolean;
  primarySource: string;
  enabledMetrics: {
    cpu: boolean; gpu: boolean; ram: boolean; vram: boolean;
    disk: boolean; network: boolean; temperatures: boolean;
  };
  exportFormat: string;
  retentionDays: number;
  diagnosticsEnabled: boolean;
  launchOnStartup: boolean;
  minimizeToTray: boolean;
  nativeFrame: boolean;
  overlay: OverlaySettings;
  thresholds: Thresholds;
  sidebarCollapsed: boolean;
  legalAcceptedVersion: string;
};

export type Bootstrap = {
  settings: Settings;
  hardware: HardwareInfo;
  capture: FrameCaptureStatus;
  recorder: RecorderState;
  sessions: SessionSummary[];
  recovered: number[];
  version: string;
  dataDir: string;
  sensorNotes: string[];
  sidecar: string;
  startedAtMs: number;
};

export type LiveFrames = {
  displayed: number[];
  presented: number[];
  frame: FrameTick | null;
  status: FrameCaptureStatus;
  recorder: RecorderState;
  messages: string[];
};

export type TickPayload = { snapshot: Snapshot; frame: FrameTick | null };

export type SelfMetrics = { cpuPercent: number; memoryBytes: number; uptimeMs: number };
export type DataStats = { sessions: number; games: number; databaseBytes: number; dataDir: string };
export type DetectedGame = {
  name: string;
  source: string;
  launcherId: string;
  exePath: string | null;
  installDir: string | null;
};
