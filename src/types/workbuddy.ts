export type WorkBuddyStatus = {
  installed: boolean;
  running: boolean;
  version: string | null;
  installDir: string | null;
  dataDir: string | null;
  lastLaunch: string | null;
  sessionCount: number;
  skillCount: number;
  mcpCount: number;
  connectorCount: number;
  modelCount: number;
};

export type WorkBuddySession = {
  id: string;
  updatedAt: number;
};

export type WorkBuddyMcp = {
  name: string;
  kind: string | null;
  url: string | null;
  command: string | null;
  disabled: boolean;
};

export type WorkBuddySkill = {
  name: string;
  description: string | null;
  source: string | null;
  canonicalName: string | null;
  version: string | null;
  installedAt: number | null;
};

export type WorkBuddyModel = {
  id: string;
  name: string | null;
  vendor: string | null;
  url: string | null;
};

export type WorkBuddyOverview = {
  status: WorkBuddyStatus;
  sessions: WorkBuddySession[];
  mcps: WorkBuddyMcp[];
  skills: WorkBuddySkill[];
  connectors: string[];
  models: WorkBuddyModel[];
};

export type WorkBuddyCliInfo = {
  available: boolean;
  cliPath: string | null;
  cliRoot: string | null;
  nodePath: string | null;
  version: string | null;
  bundle: string;
  source: string | null;
  reason: string | null;
};

export type WorkBuddyCliRun = {
  ok: boolean;
  stdout: string;
  stderr: string;
  exitCode: number | null;
  elapsedMs: number;
  authRequired: boolean;
};

export type WorkBuddySessionRow = {
  id: string;
  title: string | null;
  cwd: string | null;
  status: string | null;
  model: string | null;
  expertId: string | null;
  sourceMode: string | null;
  permissionMode: string | null;
  background: boolean;
  createdAt: number | null;
  updatedAt: number | null;
  /** 已消耗 token */
  used: number | null;
  /** 上下文窗口 */
  contextSize: number | null;
  /** 额度消耗 */
  credit: number;
};

export type WorkBuddyWorkspaceRow = {
  path: string;
  lastOpenedAt: number | null;
  sessions: number;
};

export type WorkBuddySessionOverview = {
  totalSessions: number;
  activeSessions: number;
  totalUsed: number;
  totalCredit: number;
  sessions: WorkBuddySessionRow[];
  workspaces: WorkBuddyWorkspaceRow[];
  dbPath: string | null;
  note: string | null;
};

export type WorkBuddyLoginState = {
  uid: string | null;
  accountType: string | null;
  domain: string | null;
  apiPath: string | null;
  host: string | null;
  enabled: boolean;
  updatedAt: number | null;
};

export type WorkBuddyCookie = {
  host: string;
  name: string;
  path: string | null;
  secure: boolean;
  httpOnly: boolean;
  expiresAt: number | null;
  valueLen: number;
  encrypted: boolean;
  /** 脱敏预览 */
  preview: string | null;
};

export type WorkBuddyTokenHint = {
  entry: string;
  path: string;
  length: number;
  preview: string | null;
};

export type WorkBuddyKeyblobInfo = {
  version: number;
  keyId: string | null;
  slots: number;
  slotType: string | null;
  protectorKeyId: string | null;
  updatedAt: number | null;
};

export type WorkBuddyAuthToken = {
  field: string;
  encrypted: boolean;
  envelopeLen: number;
  expiresAt: number | null;
  expiresIn: number | null;
};

export type WorkBuddyAuthAccount = {
  uid: string | null;
  uin: string | null;
  accountType: string | null;
  lastLogin: boolean;
  isAdmin: boolean;
  uniqueUserId: string | null;
  mpOpenId: string | null;
  oneidAccountId: string | null;
};

export type WorkBuddyAuthSession = {
  file: string | null;
  loggedIn: boolean;
  tokenType: string | null;
  domain: string | null;
  sessionState: string | null;
  accessToken: WorkBuddyAuthToken | null;
  refreshToken: WorkBuddyAuthToken | null;
  lastRefreshAt: number | null;
  refreshExpiresAt: number | null;
  account: WorkBuddyAuthAccount | null;
  encryptedFields: string[];
  observation: unknown;
  note: string | null;
};

export type WorkBuddyCredentialOverview = {
  loggedIn: boolean;
  loginStates: WorkBuddyLoginState[];
  cookies: WorkBuddyCookie[];
  cookiesError: string | null;
  keyblob: WorkBuddyKeyblobInfo | null;
  tokenHints: WorkBuddyTokenHint[];
  cliDir: string | null;
  cliLoggedIn: boolean;
  /** 共享产品目录下的认证会话（令牌所在） */
  authSession: WorkBuddyAuthSession | null;
  note: string | null;
};

export type WorkBuddyAccount = {
  loggedIn: boolean;
  uid: string | null;
  nickname: string | null;
  /** 昵称原文（可能含私有区占位符）。 */
  nicknameRaw: string | null;
  /** personal / team */
  accountType: string | null;
  /** free / pro ... */
  editionType: string | null;
  isPro: boolean;
  isAdmin: boolean;
  oneidAccountId: string | null;
  /** 账号快照写入时间（毫秒时间戳） */
  savedAt: number | null;
  language: string | null;
  theme: string | null;
  permissionMode: string | null;
  dataDir: string | null;
  reason: string | null;
};

export type WorkBuddyCliModel = {
  id: string;
  name: string;
  /** 传给 `--model` 的值，如 `custom-local:agnes-3.0-flash`。 */
  model: string;
  supportsToolCall: boolean;
  /** 是否为本机端点（服务未启动时不可用）。 */
  local: boolean;
};

export type WorkBuddyLogLine = {
  taskId: string;
  /** text / tool / error */
  stream: string;
  text: string;
  ts: number;
};

export type WorkBuddyTaskStatus = {
  taskId: string;
  /** running / finished / failed / cancelled */
  phase: string;
  message: string | null;
  durationMs: number | null;
  ts: number;
};

/** WorkBuddy 技能条目（能力目录） */
export type CatalogSkill = {
  id: string;
  name: string;
  description: string | null;
  source: string | null;
  marketplace: string | null;
  namespace: string | null;
  slug: string | null;
  version: string | null;
  installed: boolean;
  filePath: string | null;
  installedAt: number | null;
  tags: string[];
  /** 图标地址 */
  icon: string | null;
  /** 使用次数 */
  useCount: number | null;
  /** 安装次数 */
  installCount: number | null;
  /** 市场分类 */
  categories: string[];
  /** 更新时间（ISO 字符串） */
  updatedAt: string | null;
};

/** WorkBuddy 连接器条目（能力目录） */
export type CatalogConnector = {
  id: string;
  name: string;
  kind: string | null;
  url: string | null;
  command: string | null;
  enabled: boolean;
};

/** 专家团市场专家 */
export type CatalogExpert = {
  id: string;
  name: string;
  nameEn: string | null;
  profession: string | null;
  description: string | null;
  category: string | null;
  categoryId: string | null;
  expertType: string | null;
  tags: string[];
  /** 头像完整地址 */
  avatar: string | null;
  /** 数据来源（官方专家库 / 扩展专家库 / 本机常用） */
  source: string | null;
  /** 团队型专家的成员数 */
  memberCount: number | null;
  /** 使用次数（WorkBuddy「最热」排序字段，公开清单未带时为 null） */
  useCount: number | null;
  /** 更新时间（ISO 字符串） */
  updatedAt: string | null;
};

/** 精选场景（WorkBuddy 专家中心的推荐位） */
export type FeaturedScene = {
  id: string;
  name: string;
  description: string | null;
  image: string | null;
  expertIds: string[];
};

/** 分类（含封面图，用于顶部大卡） */
export type CatalogCategory = {
  id: string;
  name: string;
  count: number;
  cover: string | null;
  samples: string[];
};

/** 专家团成员 */
export type CatalogTeamMember = {
  agentId: string | null;
  name: string;
  role: string | null;
  agentType: string | null;
  model: string | null;
  workspace: string | null;
};

/** 本机专家团 */
export type CatalogTeam = {
  id: string;
  name: string;
  description: string | null;
  lead: string | null;
  members: CatalogTeamMember[];
  memberCount: number;
  createdAt: number | null;
};

/** 数据来源 */
export type CatalogSource = {
  id: string;
  label: string;
  /** api = 公网接口；local = 本机缓存 / 目录 */
  kind: string;
  endpoint: string;
  ok: boolean;
  count: number;
  note: string | null;
};

/** WorkBuddy 能力目录 */
export type WorkBuddyCatalog = {
  skills: CatalogSkill[];
  connectors: CatalogConnector[];
  experts: CatalogExpert[];
  teams: CatalogTeam[];
  sources: CatalogSource[];
  /** 专家分类 */
  expertCategories: CatalogCategory[];
  /** 技能分类 */
  skillCategories: CatalogCategory[];
  /** 本机常用专家 */
  myExperts: CatalogExpert[];
  /** 精选场景（推荐位） */
  featuredScenes: FeaturedScene[];
};

/** 平台下的技能条目（豆包工作 / 小米智电能） */
export type PlatformSkill = {
  id: string;
  name: string;
  description: string | null;
  icon: string | null;
  version: string | null;
  category: string | null;
  /** 内置 / 用户自建 */
  origin: string | null;
  path: string | null;
};

/** 平台下的插件条目 */
export type PlatformPlugin = {
  id: string;
  name: string;
  description: string | null;
  icon: string | null;
  /** cli / mcp / tool */
  kind: string | null;
  tools: string[];
  source: string | null;
  path: string | null;
};

/** 单个平台的技能与插件目录 */
export type PlatformCatalog = {
  id: string;
  name: string;
  skills: PlatformSkill[];
  plugins: PlatformPlugin[];
  sources: CatalogSource[];
};