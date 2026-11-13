export type Project = {
  id: string;
  name: string;
  workspace: string;
  agents: string[];
  defaultModel: string | null;
  extraArgs: string | null;
  notes: string | null;
  createdAt: number;
};

export type Credential = {
  id: string;
  provider: string;
  label: string;
  masked: string;
  createdAt: number;
};

export type AgentOverride = {
  agentId: string;
  officialDownload: string | null;
  officialPage: string | null;
  npmPackage: string | null;
  npmRegistry: string | null;
  silentArgs: string[] | null;
  enabled: boolean | null;
};

export type AppConfig = {
  npmRegistry: string;
  proxy: string | null;
  downloadDir: string | null;
  detectTimeoutSecs: number;
  installTimeoutMinutes: number;
  projects: Project[];
  agentOverrides: AgentOverride[];
  credentials: Credential[];
};

export type TaskRecord = {
  id: string;
  ts: number;
  kind: string;
  agentId: string;
  agentName: string;
  target: string;
  source: string;
  result: string;
  durationMs: number;
  message: string | null;
};

export type McpDoc = {
  path: string;
  exists: boolean;
  content: string;
  valid: boolean;
  error: string | null;
};

export type InstallPhase =
  | "queued"
  | "resolving"
  | "downloading"
  | "verifying"
  | "installing"
  | "uninstalling"
  | "finished"
  | "failed"
  | "cancelled";

export type InstallStatus = {
  taskId: string;
  agentId: string;
  agentName: string;
  action: string;
  phase: InstallPhase;
  progress: number | null;
  message: string | null;
  ts: number;
};

export type InstallLogLine = {
  taskId: string;
  agentId: string;
  stream: string;
  text: string;
  ts: number;
};