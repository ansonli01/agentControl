export type AgentKind = "cli" | "desktop";

export type InstallMethod = "npm" | "official" | "manual";

export type AgentDefinition = {
  id: string;
  name: string;
  vendor: string;
  kind: AgentKind;
  description: string;
  tags: string[];
  cliNames: string[];
  versionArgs: string[];
  npmPackage: string | null;
  officialDownload: string | null;
  officialPage: string | null;
  openPlatform: string | null;
  silentArgs: string[];
  mcpConfigPath: string | null;
  mcpConfigFormat: "json" | "toml" | null;
  configPath: string | null;
  credentialProvider: string | null;
  iconLetter: string;
  accent: string;
  installMethod: InstallMethod;
  builtin: boolean;
};

export type DetectSource = {
  kind: string;
  detail: string | null;
};

export type AgentStatus = {
  agentId: string;
  installed: boolean;
  version: string | null;
  path: string | null;
  sources: DetectSource[];
  note: string | null;
  checkedAt: number;
  versionUnknown: boolean;
};

export type Provider = {
  id: string;
  name: string;
  vendor: string;
  keyLabel: string;
  baseUrl: string | null;
  docs: string | null;
};

/** 凭据来源文件 */
export type CredentialSource = {
  label: string;
  path: string;
  exists: boolean;
  size: number;
  entries: number;
  kind: string;
};

/** 脱敏后的凭据字段 */
export type CredentialField = {
  label: string;
  value: string;
  secret: boolean;
};

/** 智能体登录态报告 */
export type AgentCredentialReport = {
  agentId: string;
  agentName: string;
  vendor: string;
  /** 所属账号体系 */
  accountSystem: string;
  /** 与哪些智能体共用同一份登录态 */
  sharedWith: string[];
  loggedIn: boolean;
  /** 登录方式 */
  method: string;
  /** 建议的登录命令 */
  loginCommand: string | null;
  sources: CredentialSource[];
  fields: CredentialField[];
  note: string | null;
};