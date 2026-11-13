# AgentControl · 多智能体控制器

Windows 桌面端的「智能体管家」：统一盘点本机已安装的各类 AI 智能体（命令行 CLI 与桌面客户端），
一键完成安装 / 升级 / 卸载，并以「项目」为单位集中管理 API Key、MCP 配置、工作区路径与历史任务记录。

界面为 WorkBuddy 式工作台骨架：左侧导航（智能体与项目）、中部主工作区、右侧详情与实时安装日志抽屉。

```
┌──────────────┬────────────────────────────────────────┬───────────────────┐
│ 侧边导航      │ 主工作区（总览/智能体/项目/凭据/历史/设置）│ 详情 + 实时日志抽屉 │
│ 品牌/探测状态 │ 工具条 + 卡片式分区                      │ 阶段进度 / 安装日志 │
└──────────────┴────────────────────────────────────────┴───────────────────┘
```

---

## 一、功能概览

| 模块 | 能力 |
|---|---|
| 智能体探测 | 多源并发扫描：PATH 可执行文件、npm 全局包清单、Windows 卸载注册表、常见安装目录；单个目标超时降级为「已安装（版本未知）」，不阻塞整体扫描 |
| 安装 / 升级 / 卸载 | npm 型走国内镜像（默认 `registry.npmmirror.com`）；官方包型走可配置直链下载 + SHA-256 校验 + 静默安装；安装任务串行排队，实时流式日志 |
| 项目管理 | 项目名称、工作区根目录、绑定智能体、默认模型、附加启动参数、备注 |
| 凭据管理 | API Key / Token 写入 Windows 凭据管理器（DPAPI），界面仅显示掩码，支持多智能体多 Key |
| MCP 配置 | 直接读写各智能体真实配置文件（如 `~/.codebuddy/mcp.json`），JSON 格式校验后原子写回 |
| 历史任务 | SQLite 持久化每一次探测 / 安装 / 升级 / 卸载：时间、目标、来源、结果、耗时、错误摘要 |
| 全局设置 | npm 镜像源、HTTP 代理、下载目录、探测超时、单个智能体的安装源覆盖 |

内置 11 个智能体条目：`opencode`、`codex`、`claude-code`、`gemini-cli`、`qwen-code`、`workbuddy`（WorkBuddy）、
`doubao`（豆包）、`opencode-desktop`、`codebuddy`、`trae`、`cursor`；新增智能体 = 在清单中追加一条定义，无需改动业务代码。

---

## 二、技术栈

| 层 | 选型 |
|---|---|
| 桌面框架 | Tauri v2（Rust 后端 + 系统 WebView2） |
| 后端 | Rust 1.96 · tokio · serde · reqwest(rustls) · rusqlite(bundled) · keyring(Windows 凭据管理器) · sha2 · winreg |
| 前端 | Vue 3 + TypeScript + Vite + Pinia + Vue Router + TailwindCSS + lucide-vue-next + vue-echarts |
| 数据落盘 | 配置 `%APPDATA%\AgentControl\config.json`；历史 `%APPDATA%\AgentControl\history.db`；密钥进 Windows 凭据管理器 |

---

## 三、快速开始

### 环境要求
- Node.js ≥ 20（开发环境实测 v24.16.0）、npm ≥ 10
- Rust stable（实测 1.96.0，`stable-x86_64-pc-windows-msvc`）+ Visual Studio 2022 生成工具
- WebView2 Runtime（Windows 10/11 已内置）

### 安装依赖与运行

```bash
npm install          # 前端依赖
npm run tauri:dev    # 启动桌面端（自动拉起 vite dev server）
```

其他脚本：

```bash
npm run dev          # 仅启动前端（浏览器预览）
npm run build        # 前端类型检查 + 生产构建
npm run tauri:build  # 打包 Windows 安装包（NSIS）
```

### 命令行自检

```bash
cd src-tauri
cargo test                              # 14 个单元 / 实机测试
cargo run --example detect              # 无 GUI 探测本机智能体，打印清单
cargo build --release --features custom-protocol   # 产出可独立运行的 exe
```

> ⚠️ `cargo build --release` **不要**直接用：未启用 Tauri 的 `custom-protocol` 时，产物仍指向
> `localhost:1420`，双击会显示「无法访问此页面」。请带 `--features custom-protocol`，
> 或直接用 `npm run tauri:build`（CLI 会自动带上该 feature）。

### 界面风格

界面参照 WorkBuddy 工作台改造为**浅色两栏布局**：

- **第 1 列（248px 导航侧栏）**：应用名与版本、发现应用、导航（新建任务 / 助理 BETA / 专家·技能·连接器 /
  **WorkBuddy** / 定时任务 / 资料库 / 更多）、任务列表、空间列表、底部用户条（含通知与对话按钮）。
  右边缘为隐形拖拽手柄：悬停出现绿色散射光晕且光标变左右拉伸，拖动可调宽度（200–420px，双击复位默认 248px）。

### WorkBuddy 集成

侧栏「WorkBuddy」页会读取本机 WorkBuddy（腾讯桌面 Agent）的运行状态与能力清单：

- **状态**：是否安装 / 是否运行中、版本、数据目录 `~/.workbuddy`、安装目录。
- **能力**：会话（`workspace/sessions`）、技能（`.skill-list-cache.json`，SkillHub 市场）、
  MCP 服务（`mcp.json`）、连接器（`connectors/`）、自定义模型（`models.json`）。
- **操作**：一键打开数据目录 / 会话目录 / 官网。

> WorkBuddy 的本地 gateway（`Authorization: Bearer <CODEBUDDY_GATEWAY_PASSWORD>`）密钥进程内随机生成、不落盘，
> guest SDK 仅供其内部 webview 使用，因此本集成走**只读数据目录**这条稳定路径；
> `models.json` 中的明文 API Key 不会被读取或回传。

#### CLI 引擎

WorkBuddy 安装包已把 CodeBuddy CLI 完整落到磁盘，**无需解包 app.asar**：
`<安装目录>/resources/app.asar.unpacked/cli`（`bin/codebuddy` 为 Node 脚本，主 bundle 是
`dist/codebuddy-lite-wb.mjs`，需 `CODEBUDDY_FORCE_LITE_WB_BUNDLE=1` 命中）。
「WorkBuddy」页的 **CLI 引擎** 卡片会自动定位 CLI 与 Node（WorkBuddy 自带
`~/.workbuddy/binaries/node/versions/<v>/node.exe`），并支持用 `-p` 非交互模式试跑一句话。

- 定位顺序：`WORKBUDDY_CLI_ROOT` 环境变量 → 运行中的 WorkBuddy 进程 → 注册表 Uninstall → 常见安装路径。
- 支持 `--output-format stream-json`、`--model`、`--permission-mode` 等原生参数语义。
- CLI 需单独登录一次：在终端运行 `codebuddy` 后执行 `/login`，即可复用 WorkBuddy 账号与模型配置。

#### 智能体详情右栏里的 WorkBuddy 数据

点击「助理」页任意智能体卡片，右侧会滑出**详情右栏**（`InstallDrawer`，420px）：
智能体概要 → 安装状态 → 安装源 → 安装进度日志，底部常驻「**WorkBuddy 本机数据**」：

- **登录账号**：`~/.workbuddy/storage/skeleton/account-snapshot.json`（昵称 / UID / 套餐 / 管理员 / 语言 / 主题）。
  昵称里的 Unicode 私有区占位符（U+E000–U+F8FF）会被清洗，否则显示成方块。
- **会话数据**：`~/.workbuddy/workbuddy.db`（只读）——`sessions` / `session_usage` / `workspaces`，
  展示会话总数、未结束数、累计 token、累计额度、最近会话与最近工作区。
- **登录令牌与凭据**：
  - **认证会话（令牌真正所在）**：`%LOCALAPPDATA%\CodeBuddyExtension\Data\Public\auth\workbuddy-desktop.info`
    —— WorkBuddy 桌面端与 CodeBuddy IDE 扩展**共用**的产品目录下，路径规则见
    `main/file-authentication-storage.js` 的 `getAuthSavePath()`：
    `join(sharedDataPath, "auth", `${authentication.id}.info`)`。
    文件里 `auth.accessToken` / `auth.refreshToken` 为字段级密文（`$wbEncrypted` + envelope），
    `tokenType` / `domain` / `sessionState` / `expiresAt` / `refreshExpiresAt` 为明文。
  - 密钥链：`protector`（运行时注入，未落盘）→ 解 `~/.workbuddy/keyblob`（static-v1 槽）→ 静态密钥
    → HKDF-SHA256(`workbuddy-auth-session`, `auth/<文件名>.info`) → AES-256-GCM。
    因 protector 需主进程注入，本项目**只读元信息与有效期，不解密令牌**。
  - 登录态索引另存于 `~/.workbuddy/local_storage/*.info`（JSON 字符串 → base64 → gzip → JSON），
    其 `enabled` 字段可能为 false，**不能据此判断是否登录**。
  - Cookie 来自 `~/.workbuddy/app/session/Network/Cookies`；**WorkBuddy 运行期间该库被独占锁定**，
    退出 WorkBuddy 后才能读到，界面会明确提示。
  - `~/.workbuddy/keyblob` 是本地文件加密的主密钥容器，这里**只读取元信息，不导出密钥**。
  - 所有凭据展示均为脱敏形式（前 8 位 + 后 4 位 + 长度），完整令牌不会出现在界面或 IPC 返回值里。

#### 派发任务给 WorkBuddy

「WorkBuddy」页可直接把任务派给内置 CLI（`Read` / `Write` / `Bash` 等工具权限齐备）：

- **模型**：读取 CLI 配置目录 `~/.codebuddy/models.json` 的自定义模型（`custom-local:<id>`），
  远程端点优先，本机端点标注「（本机）」。用自定义模型可**免官方登录**直接工作。
- **执行**：`-p <prompt> --output-format stream-json`，逐行解析 NDJSON：
  助手文本实时回显，工具调用显示为 `[调用工具 X]`，错误单独标记。
- **控制**：可随时取消（杀子进程）；完成后写入历史任务（`kind=workbuddy`），在「定时任务」页可筛。
- **第 2 列（主工作区）**：顶部 40px 菜单行（文件 / 搜索 / 窗口 / 帮助 + 已安装计数 + 一键扫描）。

首页为居中大标题「AgentControl，我帮你」+ 模式胶囊（办公 / 开发 / 设计）+ 快捷标签 + 大输入框
（`@` 添加上下文、`/` 调用技能）+ 状态条 + 推荐卡片 + 统计卡与厂商分布图。
配色：主工作区 `#FFFFFF`（白底）、侧栏与标题栏 `#FAFAFB`、卡片 `#FFFFFF` + 描边 `#E6E8EC`、
文字 `#1D2129`、品牌蓝 `#3B6EF5`、日志区深色终端。

**无边框窗口**：已去掉系统标题栏（`decorations: false`），顶部为 36px 自绘标题栏：
左侧 AC 标识 + 标题 + **文件 / 搜索 / 窗口 / 帮助** 菜单，右侧「已安装 N / M + 一键扫描」与最小化 / 最大化 / 关闭按钮；
标题栏可拖动窗口、双击最大化。对应权限在 `src-tauri/capabilities/default.json` 中声明。

**用户面板**：点击侧栏左下角「AC AgentControl」用户条，向上弹出账号小面板——套餐（本地版）与已装智能体、
「打开助理市场 / 检查更新」、助理市场 / 工作区与空间 / 凭据与 MCP / 定时任务记录 / 设置（`Ctrl+,`）/
记忆与进化，以及「退出应用」；点击外部或按 `Esc` 关闭。

完整规范见 [.loop/ui-spec.md](.loop/ui-spec.md) 第六、八、九节。

### 一键脚本 `run.bat`

仓库根目录提供 `run.bat`，**双击即可运行**，内置菜单：

| 选项 | 作用 | 实际执行 |
|---|---|---|
| 1 | 开发模式启动 | `npm run tauri:dev` |
| 2 | 仅构建前端 | `npm run build` |
| 3 | 打包 Windows 安装包 | `npm run tauri:build` |
| 4 | 运行后端测试 | `cargo test` |
| 5 | 命令行探测本机智能体 | `cargo run --example detect` |
| 6 | 重新生成应用图标 | `scripts\generate-icons.ps1` |
| 7 | 一键发布版 | 前端构建 → `cargo build --release` → 启动应用 |
| 0 | 退出 | — |

> 脚本菜单文案使用英文：Windows `cmd` 解析包含多字节中文的 `.bat` 时按字节偏移定位 `goto` 标签，
> 会出现行错位（如 `'gentControl' is not recognized`）。纯 ASCII 的批处理在 UTF-8 / ANSI 两种代码页下行为一致，最稳。

---

## 四、目录结构

```
agentControl/
├── PLAN.md                     # 产品计划（需求 / 架构 / 验收）
├── scripts/generate-icons.ps1  # 自定义应用图标生成脚本
├── src/                        # Vue3 前端
│   ├── api/ipc.ts              # invoke 封装 + 事件订阅
│   ├── stores/                 # agents / config / tasks 三个 Pinia store
│   ├── views/                  # 总览、智能体、项目、凭据&MCP、历史、设置
│   ├── components/             # AgentCard、InstallDrawer、StatusTag、EmptyState
│   └── index.css               # 深色主题变量与基础样式
└── src-tauri/
    ├── build.rs                # 资源与应用程序清单嵌入
    ├── capabilities/           # 最小权限声明
    ├── icons/                  # 应用图标
    └── src/
        ├── agent/              # 智能体清单（model + registry）
        ├── detect/             # 并发探测（scanners：PATH / npm / 注册表 / 安装目录）
        ├── install/            # 安装队列、npm 安装、官方包下载、校验
        ├── config/             # 配置存储、凭据库、MCP、工作区
        ├── tasks/              # SQLite 历史任务
        ├── runtime/            # 子进程封装（流式输出 / 超时 / 取消）
        └── commands/           # Tauri 命令（17 个）
```

---

## 五、关键设计决策

1. **清单驱动**：所有探测 / 安装逻辑只面向 `AgentDefinition` 编程，新增智能体仅追加清单条目；
   豆包、WorkBuddy 的官方下载地址未证实，故默认只指向官网入口，下载地址可在「设置 → 智能体安装源」中覆盖。
2. **探测并行、安装串行**：探测用 `join_all` 并发，npm 全局安装存在目录锁，故安装进入单消费者队列，两者互不阻塞。
3. **O(1) 覆盖 npm 智能体**：`npm ls -g --depth=0 --json` 一次调用解析全部 npm 型智能体版本，避免逐个 `--version`。
4. **凭据不落盘明文**：API Key 存 Windows 凭据管理器，配置文件仅存引用 ID，日志与回显一律掩码（`sk-****abcd`）。
5. **命令注入防护**：外部命令一律以参数数组传参，不拼接 shell 字符串。
6. **原子写入**：配置与 MCP 文件先写 `.tmp` 再 rename，失败保留原文件。
7. **清单嵌入由链接器完成**：`tauri-build` 生成的资源只链接到 bin 目标，`cargo test` 的测试可执行文件会缺少
   Common-Controls v6 清单，导致进程以 `0xC0000139` 启动失败；因此改为由链接器统一嵌入清单（见 `src-tauri/build.rs`）。

---

## 六、实机验证结果

在本机（Windows + Node v24.16.0 + rustc 1.96.0）验证：

| 验证项 | 结果 |
|---|---|
| `cargo test` | 14 passed / 0 failed（含 npm 探测与全量探测实机测试） |
| `npm run build` | `vue-tsc --noEmit` 通过，vite build 成功 |
| `cargo build --release` | 成功（3m26s），可执行文件正常启动并保持运行 |
| 探测实机结果 | 识别出 4/11：`codex 0.138.0`（npm + PATH）、`WorkBuddy 5.9.2`（注册表）、`豆包 2.25.18`（注册表）、`OpenCode 桌面版`（安装目录） |
| `npm run tauri:build` | 未完成：打包器需从 GitHub 下载 wix/NSIS 工具链，当前网络无法访问 GitHub（npmmirror 可达）；已验证到「release 编译通过 + 应用可正常启动」 |

---

## 七、已知限制

- 豆包 / WorkBuddy 官方安装包直链未获官方证实，清单默认只给出官网入口，需在「设置 → 智能体安装源」手动填入直链。
- 桌面客户端的版本号依赖卸载注册表 `DisplayVersion`，便携版安装目录只能识别为「版本未知」。
- 本期不包含多智能体统一会话编排，仅在 `runtime/process.rs` 预留了子进程流式封装能力。
- 未打包 macOS / Linux 安装包（`bundle.icon` 已移除 `.icns`）。
- `npm run tauri:build` 需要联网下载 wix / NSIS 工具链；离线或无法访问 GitHub 时会报 `failed to bundle project: timeout: global`，
  此时可用 `cargo build --release` 产出可执行文件。

---

## 九、按智能体切换的登录态（2026-11-13）

### 结论：登录态按**账号体系**隔离
每个智能体属于不同厂商的账号体系，凭据存储与格式互不相同，令牌无法互换：

| 智能体 | 账号体系 | 凭据位置 |
|---|---|---|
| WorkBuddy / CodeBuddy / CodeBuddy CLI | 腾讯 CodeBuddy（**三者共用同一份**） | `%LOCALAPPDATA%\CodeBuddyExtension\Data\Public\auth\<id>.info` + `~\.workbuddy\storage\skeleton\account-snapshot.json` |
| Codex CLI | OpenAI | `~\.codex\auth.json` |
| Claude Code | Anthropic | `~\.claude\.credentials.json`、`~\.claude.json` |
| Gemini CLI | Google | `~\.gemini\oauth_creds.json` |
| Qwen Code | 阿里云百炼 | `~\.qwen\oauth_creds.json` |
| OpenCode | 各模型厂商 API Key | `~\.local\share\opencode\auth.json`、`~\.config\opencode\opencode.json` |
| 豆包 | 火山方舟 API Key | 不落盘，走环境变量 `ARK_API_KEY` |
| TRAE / Cursor | 各自应用内登录 | 应用自行管理 |

### 改动
- 后端：新增 `src-tauri/src/commands/agent_credentials.rs`
  - 命令 `agent_credentials(agent_id)` → `AgentCredentialReport`（账号体系 / 是否共用 / 登录方式 /
    登录命令 / 凭据来源 / 脱敏字段 / 说明）。
  - 通用 JSON 扫描器递归提取关键字段，敏感值只留前 4 + 后 4 + 长度；令牌一律不导出明文。
  - `read_auth_session()` 改为 `pub(crate)` 复用 CodeBuddy 系认证会话读取。
- 前端：新增 `src/components/AgentCredentialPanel.vue`；`InstallDrawer.vue` 原
  「WorkBuddy 本机数据（常驻）」改为「登录态」并随选中智能体切换；
  会话数据拆到 `src/components/WorkBuddySessionsPanel.vue`（仅 CodeBuddy 系渲染）；
  删除 `src/components/WorkBuddyPanels.vue`。

---

## 十、「技能与连接」新增 WorkBuddy 页（2026-11-13）

### 数据来源（三类混合，界面上以 `API` / `本机` 标记区分）
| 数据 | 来源 | 说明 |
|---|---|---|
| 专家团市场 | **公网 API** `https://acc-1258344699.cos.accelerate.myqcloud.com/workbuddy/expert-marketplace/expert_center.json` | 429 个专家 / 15 个分类，腾讯 COS 静态资源，公开可访问 |
| 专家团精选场景 | **公网 API** `.../expert-marketplace/featuredScenes.json` | 10 个场景 |
| 技能 | WorkBuddy 的 **API 缓存** `~\.workbuddy\.skill-list-cache.json` + 本机 `~\.workbuddy\skills\*` | 技能正式接口 `https://api.skillhub.cn/console/agent-gateway/internal_knot-skill/apigw/api/v1/skills` 需要登录态（Authorization + X-User-Id），本机令牌是密文无法还原，故直接复用 WorkBuddy 自己拉取的缓存 |
| 连接器 | 本机 `~\.workbuddy\connectors-marketplace\connectors\*\mcp.json` + 接入状态 `~\.workbuddy\connectors\<uid>\connector-states.json` | 市场清单远端为 `https://static.workbuddy.cn/connectors-config-v2/connectors-config.zip`（42 MB，本地已有解压缓存） |
| 本机专家团 | 本机 `~\.workbuddy\teams\<id>\config.json` | AgentTeams，含 lead 与成员列表 |

### 改动
- 后端：新增 `src-tauri/src/commands/workbuddy_catalog.rs`
  - 命令 `workbuddy_catalog()` → `WorkBuddyCatalog`（skills / connectors / experts / teams / sources）。
  - `sources` 逐项标注来源类型（`api` / `local`）、端点或路径、条目数与异常信息。
  - 专家团市场用 `reqwest` 直连公网接口，30 s 超时，失败时降级为空并给出原因。
- 前端：新增 `src/components/WorkBuddyCatalogPanel.vue`（技能 / 连接 / 专家团 / 本机专家团四个可折叠区块，
  顶部搜索过滤与来源徽标）；`src/views/ProjectsView.vue` 增加「项目 / WorkBuddy」两个页签。
- 类型：`src/types/workbuddy.ts` 增加 `WorkBuddyCatalog` 等 6 个类型；`src/api/ipc.ts` 增加 `workbuddyCatalog()`。

### 能力目录面板改版（2026-11-13 晚，对齐 WorkBuddy 官方「专家」页）

「专家 · 技能 · 连接器」页顶部内嵌 WorkBuddy 能力面板（三个页签：专家 / 技能 / 连接器），
卡片与布局对齐 WorkBuddy 官方界面：一级分类大卡（封面用该分类首位专家的头像）、二级分类 chips、
排序、4 列网格卡片（圆形头像 + 名称 + 定位 + 描述 + 标签 + 底部统计）。

| 数据 | 来源 | 说明 |
|---|---|---|
| 专家（官方） | **公网 API** `https://acc-1258344699.cos.accelerate.myqcloud.com/workbuddy/experts/expert_center.json` | 151 位专家 / 12 分类；`avatar` 为相对路径，需拼 `/workbuddy/experts` 基址 |
| 专家（扩展 / 专家团） | **公网 API** `.../workbuddy/expert-marketplace/expert_center.json` | 429 位，取其中团队型专家（`members`）补齐「专家团」品类 |
| 专家（我的） | 本机 `~/.workbuddy/storage/user-<uid>-personal/global/expert.json` 的 `recents` | 腾讯真实专家（吴八哥 / 卖得好 等），头像在 `openplatform-cdn.codebuddy.cn` |
| 技能 | 本机 `~/.workbuddy/storage/user-<uid>-personal/scoped/<scope>/skill-market.json`（WorkBuddy 自己落盘的市场缓存）+ `~/.workbuddy/.skill-list-cache.json` | 166 条，含图标、分类（OPC·一人公司 / 办公协同 / 双十一 …）、使用与安装次数 |
| 连接器 | 本机 `~/.workbuddy/connectors-marketplace/connectors/*/mcp.json` + `connectors/<uid>/connector-states.json` | 312 条 |

后端 `src-tauri/src/commands/workbuddy_catalog.rs` 新增 `collect_skill_market()`、`collect_my_experts()`、
`absolute()`，并给技能/专家补 `icon`、`use_count`、`categories`、`updated_at` 等字段；
前端 `src/components/WorkBuddyCatalogPanel.vue` 重写为三页签布局，
`src/views/ProjectsView.vue` 去掉外层页签，改为「能力目录面板 + 项目列表」上下排布。

### 多平台能力目录（2026-11-13 晚追加：豆包工作 / 小米智电能）

「专家 · 技能 · 连接器」页改为「项目 / 能力目录」两个页签：**项目列表只出现在「项目」页签**，
能力目录页顶部新增平台切换（`src/components/PlatformCatalogPanel.vue`），三个平台内容互不混排：

| 平台 | 技能 | 插件 |
|---|---|---|
| WorkBuddy | 专家 / 技能 / 连接器（原有面板，含公网专家中心与本机市场缓存） | — |
| 豆包工作 | 41 个内置技能（`SKILL.md`） | 4 个本地工具包（lark-cli / mediakit-cli / design-cli / rpa-dev-local） |
| 小米智电能 | 11 个内置技能 | 11 个插件（内置 desktop-tools-plugin、运行时插件包、MCP 服务器） |

各平台本地数据源：

- 豆包工作技能：`%LOCALAPPDATA%\DoubaoWork\User Data\Default\.doubaowork\agent_mode\workspace\.skills\<skill>\SKILL.md`
  （`.user_skills` 为用户自建）；插件：技能 `metadata.requires.bins` + 沙箱
  `sandbox_envs_dir\.prepared_packages\*\dlcs\*\*.exe`。
- 小米智电能技能：`%APPDATA%\Xiaomi MiMo\engine-config\skills\<skill>\SKILL.md`（frontmatter 含 `icon: brand:xxx`）；
  插件：`engine-config\mimocode.json` 的 `plugin` 数组、`runtime-res\releases\main\plugin*.mjs`、
  `resources\runtimes\win32-x64\*`（MCP 服务器）。

后端新增 `src-tauri/src/commands/platform_catalogs.rs`（命令 `platform_catalogs`），含极简 YAML frontmatter 解析
与 3 个单元测试；前端新增 `PlatformCatalogPanel.vue`、`PlatformCatalog` / `PlatformSkill` / `PlatformPlugin` 类型。
卡片网格由 4 列改为 **3 列**（`panel p-4`），避免信息拥挤。

### 逆向过程中确认的 WorkBuddy 内部机制（备查）
- `wbipc` 是 **HTTP over 命名管道**（`\\.\pipe\wbipc-<hash>`），JSON-RPC 2.0 逐行分帧；
  握手为 `session_hello` → `session_challenge` → `session_hello_ack`，
  双方用 HMAC-SHA256(ticket, "wbipc-s"/"wbipc-c" + 协议|端点|client_nonce|server_nonce) 互证；
  ticket 落盘在 `~\.workbuddy\wbipc\endpoint.json`。实现见
  `resources/app.asar.unpacked/resources/plugins/workbuddy-builtin/skills/library/workbuddy_ipc/`。
- 本机 Gateway 需要进程内随机密码（`CODEBUDDY_GATEWAY_PASSWORD`），外部进程连不上，
  因此走 wbipc 或读本地缓存是现实路径。

---

## 十一、更多文档

- 产品计划：[PLAN.md](./PLAN.md)
- 架构与接口说明：[docs/architecture.md](./docs/architecture.md)
- 智能体清单与扩展：[docs/agents.md](./docs/agents.md)
- 配置、凭据与 MCP：[docs/configuration.md](./docs/configuration.md)
- 开发过程与决策记录：[.loop/](./.loop/)
