import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import type {
  AgentCredentialReport,
  AgentDefinition,
  AgentStatus,
  Provider,
} from "../types/agent";
import type {
  AgentOverride,
  AppConfig,
  InstallLogLine,
  InstallStatus,
  McpDoc,
  Project,
  TaskRecord,
} from "../types/config";
import type {
  WorkBuddyAccount,
  PlatformCatalog,
  WorkBuddyCatalog,
  WorkBuddyCliInfo,
  WorkBuddyCliModel,
  WorkBuddyCliRun,
  WorkBuddyCredentialOverview,
  WorkBuddyOverview,
  WorkBuddySessionOverview,
  WorkBuddyStatus,
} from "../types/workbuddy";

export type DetectProgress = {
  agentId: string;
  agentName: string;
  phase: string;
  done: number;
  total: number;
  finished: boolean;
};

export const api = {
  listAgents: () => invoke<AgentDefinition[]>("list_agents"),
  listProviders: () => invoke<Provider[]>("list_providers"),
  agentCredentials: (agentId: string) =>
    invoke<AgentCredentialReport>("agent_credentials", { agentId }),
  detectAll: () => invoke<AgentStatus[]>("detect_all"),
  installStart: (agentId: string, action: string) =>
    invoke<string>("install_start", { agentId, action }),
  installCancel: (taskId: string) => invoke<void>("install_cancel", { taskId }),
  getConfig: () => invoke<AppConfig>("get_config"),
  saveConfig: (config: AppConfig) => invoke<AppConfig>("save_config", { config }),
  saveAgentOverride: (item: AgentOverride) =>
    invoke<AppConfig>("save_agent_override", { item }),
  saveProject: (project: Project) => invoke<AppConfig>("save_project", { project }),
  deleteProject: (projectId: string) =>
    invoke<AppConfig>("delete_project", { projectId }),
  saveCredential: (provider: string, label: string, secret: string) =>
    invoke<AppConfig>("save_credential", { provider, label, secret }),
  deleteCredential: (credentialId: string) =>
    invoke<AppConfig>("delete_credential", { credentialId }),
  loadMcp: (agentId: string) => invoke<McpDoc>("load_mcp", { agentId }),
  saveMcp: (agentId: string, content: string) =>
    invoke<McpDoc>("save_mcp", { agentId, content }),
  listWorkspaceRoots: () => invoke<string[]>("list_workspace_roots"),
  listTasks: (limit?: number, kind?: string) =>
    invoke<TaskRecord[]>("list_tasks", { limit: limit ?? null, kind: kind ?? null }),
  clearTasks: () => invoke<void>("clear_tasks"),
  workbuddyStatus: () => invoke<WorkBuddyStatus>("workbuddy_status"),
  workbuddyOverview: () => invoke<WorkBuddyOverview>("workbuddy_overview"),
  workbuddyPaths: () => invoke<Record<string, string>>("workbuddy_paths"),
  workbuddyOpen: (target: string) => invoke<void>("workbuddy_open", { target }),
  workbuddyAccount: () => invoke<WorkBuddyAccount>("workbuddy_account"),
  workbuddySessions: (limit?: number) =>
    invoke<WorkBuddySessionOverview>("workbuddy_sessions", { limit: limit ?? null }),
  workbuddyCredentials: () =>
    invoke<WorkBuddyCredentialOverview>("workbuddy_credentials"),
  workbuddyCatalog: () => invoke<WorkBuddyCatalog>("workbuddy_catalog"),
  platformCatalogs: () => invoke<PlatformCatalog[]>("platform_catalogs"),
  workbuddyCliInfo: () => invoke<WorkBuddyCliInfo>("workbuddy_cli_info"),
  workbuddyCliRun: (prompt: string, cwd?: string, timeoutSecs?: number) =>
    invoke<WorkBuddyCliRun>("workbuddy_cli_run", {
      prompt,
      cwd: cwd ?? null,
      timeoutSecs: timeoutSecs ?? null,
    }),
  workbuddyCliModels: () => invoke<WorkBuddyCliModel[]>("workbuddy_cli_models"),
  workbuddyDispatchStart: (prompt: string, cwd?: string, model?: string) =>
    invoke<string>("workbuddy_dispatch_start", {
      prompt,
      cwd: cwd ?? null,
      model: model ?? null,
    }),
  workbuddyDispatchCancel: (taskId: string) =>
    invoke<void>("workbuddy_dispatch_cancel", { taskId }),
};

export const onDetectProgress = (cb: (p: DetectProgress) => void) =>
  listen<DetectProgress>("detect://progress", (e) => cb(e.payload));

export const onInstallLog = (cb: (l: InstallLogLine) => void) =>
  listen<InstallLogLine>("install://log", (e) => cb(e.payload));

export const onInstallStatus = (cb: (s: InstallStatus) => void) =>
  listen<InstallStatus>("install://status", (e) => cb(e.payload));