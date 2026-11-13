# 架构说明

## 1. 总体结构

```
┌──────────────────────────── Vue 3 前端 (Tailwind 深色主题) ────────────────────────────┐
│  DashboardView  AgentsView  ProjectsView  CredentialsView  TasksView  SettingsView          │
│  stores: agents / config / tasks        api/ipc.ts (invoke 封装 + 事件订阅)                  │
└───────────────▲───────────────────────────────────────────────▲──────────────────────────────┘
                │ invoke(命令)                                │ listen(事件)
┌───────────────┴───────────────────────────────────────────────┴──────────────────────────────┐
│                            Tauri IPC 层 (src-tauri/src/commands)                              │
│  agent.rs   detect.rs   install.rs   config.rs   tasks.rs                                     │
└───────▲───────────────────────────────▲──────────────────────────────▲─────────────────────────┘
        │                               │                              │
┌───────┴────────┐  ┌──────────────────┴───────┐  ┌───────────────────┴──────────┐
│ agent/registry │  │ detect（多源并发探测）   │  │ install（串行队列 + 流式日志） │
│ 智能体清单     │  │  PATH/npm/注册表/目录   │  │  npm 镜像安装 / 官方包下载     │
└────────────────┘  └──────────────────────────┘  └──────────────────────────────┘
┌────────────────┐  ┌──────────────────────────┐  ┌──────────────────────────────┐
│ config/store   │  │ config/secrets (keyring) │  │ tasks (SQLite)                │
│ config/mcp     │  │ runtime/process 子进程   │  │ error / events / state        │
└────────────────┘  └──────────────────────────┘  └──────────────────────────────┘
```

## 2. 数据流

1. 前端启动：`bindEvents()` → `list_agents()` → `get_config()` → `detect_all()`
2. 探测：后端 `detect_all` 并发跑 `detect_one`，每完成一个目标 `emit("detect://progress")`；
   结果写入 `AppState::detect_cache`（内存缓存）。
3. 安装：`install_start(agent_id, action)` 生成 task_id 并入串行队列；
   阶段变化 `emit("install://status")`，子进程输出 `emit("install://log")`；结束后写入 SQLite `tasks` 表。
4. 配置：`get_config` / `save_config` 读写 `%APPDATA%\AgentControl\config.json`（临时文件 + rename 原子替换）。
5. 凭据：密钥本体存 Windows 凭据管理器（keyring，服务名 `AgentControl`），配置文件只保存引用 ID 与掩码。

## 3. 后端模块

| 模块 | 关键类型 / 函数 | 说明 |
|---|---|---|
| `agent/model.rs` | `AgentDefinition`、`AgentStatus`、`DetectSource`、`AgentOverride`、`Provider` | 清单与探测结果的数据契约，serde camelCase 与前端 `src/types/*.ts` 对齐 |
| `agent/registry.rs` | `builtin_agents()`、`providers()`、`merge_overrides()`、`find()` | 11 个内置智能体 + 6 个凭据服务商工；`merge_overrides` 合并用户自定义安装源 |
| `detect/mod.rs` | `detect_all()`、`detect_one()` | 并发编排（`join_all`）+ 进度事件；CLI 型走 npm/PATH，桌面型走注册表/安装目录 |
| `detect/scanners.rs` | `npm_global_packages()`、`find_executable()`、`extra_bin_dirs()`、`probe_version()`、`scan_uninstall_registry()`、`scan_install_dirs()` | 具体扫描器；关键词归一化（忽略大小写/空格/连字符），`OpenCode Desktop` 可匹配 `@opencodedesktop` |
| `install/mod.rs` | `InstallAction`（Install/Upgrade/Uninstall） | 安装动作抽象 |
| `install/npm.rs` | `npm_args()`、`npm_preview()` | `npm -g install|update|uninstall <pkg> --registry <mirror> --no-fund --no-audit --loglevel info` |
| `install/official.rs` | `download()`、`run_installer()`、`default_download_dir()` | 官方包流式下载（进度回调）+ 静默安装，超时 900s |
| `install/checksum.rs` | `sha256_file()`、`sha256_bytes()` | 完整性校验 |
| `config/store.rs` | `AppConfig`、`Project`、`Credential`、`load()`、`save()`、`write_atomic()` | 全局配置；默认镜像 `https://registry.npmmirror.com`、探测超时 8s、安装超时 30min |
| `config/secrets.rs` | `set_secret()`、`get_secret()`、`delete_secret()`、`mask()` | Windows 凭据管理器读写 + 掩码（前 3 后 4） |
| `config/mcp.rs` | `read_mcp()`、`validate_json()`、`save_mcp()`、`expand_path()` | MCP 配置文件定位（支持 `~`、`%VAR%`）、JSON 校验、原子写回 |
| `config/workspace.rs` | `validate_workspace()`、`list_roots()` | 工作区目录校验与根目录枚举 |
| `tasks/mod.rs` | `TaskRecord`、`TaskDb` | SQLite `tasks` 表 + `idx_tasks_ts` 索引 |
| `runtime/process.rs` | `run_capture_timeout()`、`run_streaming()`、`which()`、`build_command()` | 子进程封装：stdout/stderr 分流、超时、`kill_on_drop`、取消标志 |
| `state.rs` | `AppState` | 配置缓存、探测缓存、安装任务句柄、DB 连接、数据目录 `%APPDATA%\AgentControl` |
| `events.rs` | `DetectProgress`、`InstallStatus`、`InstallLogLine`、`events::*` | 事件名与负载定义 |

## 4. Tauri 命令（17 个）

| 模块 | 命令 | 说明 |
|---|---|---|
| agent | `list_agents` / `list_providers` | 清单与凭据服务商工 |
| detect | `detect_all` | 全量探测，返回 `AgentStatus[]` |
| install | `install_start` / `install_cancel` | 安装任务入队（返回 task_id）与取消 |
| config | `get_config` / `save_config` | 全局配置读写 |
| config | `save_agent_override` | 保存单个智能体的安装源覆盖 |
| config | `save_project` / `delete_project` | 项目增删改 |
| config | `save_credential` / `delete_credential` | 凭据增删（密钥入凭据库） |
| config | `load_mcp` / `save_mcp` | MCP 文档读取与保存 |
| config | `list_workspace_roots` | 可选工作区根目录 |
| tasks | `list_tasks` / `clear_tasks` | 历史查询（默认 200 条，clamp 1..1000）与清空 |

## 5. 事件

| 事件 | 负载 | 触发时机 |
|---|---|---|
| `detect://progress` | `{ agentId, agentName, phase, done, total, finished }` | 每探测完一个目标 |
| `install://status` | `{ taskId, agentId, agentName, action, phase, progress, message, ts }` | 阶段变化（Queued/Resolving/Downloading/Verifying/Installing/Uninstalling/Finished/Failed/Cancelled） |
| `install://log` | `{ taskId, agentId, stream, text, ts }` | 子进程 stdout/stderr 按行推送 |

## 6. 前端结构

| 文件 | 职责 |
|---|---|
| `src/App.vue` | 工作台骨架：可折叠侧边栏 + 顶部状态条 + 路由视图 + 右侧抽屉 |
| `src/api/ipc.ts` | `invoke` 封装（17 个命令）与事件订阅（3 个事件） |
| `src/stores/agents.ts` | 清单、探测状态、安装任务（每个 task 的阶段 + 日志，上限 500 行） |
| `src/stores/config.ts` | 全局配置、项目、凭据、覆盖项 |
| `src/stores/tasks.ts` | 历史任务列表与筛选 |
| `src/views/*.vue` | 六个功能视图 |
| `src/components/*.vue` | `AgentCard`、`InstallDrawer`（阶段进度 + 实时日志）、`StatusTag`、`EmptyState` |

## 7. 构建注意事项

- **Windows 清单**：`tauri-build` 生成的资源只链接到 bin 目标，`cargo test` 的测试可执行文件缺少
  Common-Controls v6 依赖，会在启动时因 `TaskDialogIndirect` 无法解析而以 `0xC0000139` 退出。
  `src-tauri/build.rs` 因此改用 `WindowsAttributes::new_without_app_manifest()`，
  由链接器通过 `/MANIFEST:EMBED` + `/MANIFESTINPUT:` 统一嵌入清单，bin 与测试可执行文件同时生效。
- **rc.exe 路径转义**：若改回用 `rc.exe` 编译资源，`.rc` 中的清单路径必须使用正斜杠，否则 `\a`、`\t` 会被当作转义符。
- **link.exe 参数**：`/MANIFESTINPUT:` 的路径不能加引号，否则会被当作相对路径。
- **图标**：`scripts/generate-icons.ps1` 生成 PNG 全套 + `icon.ico`；Windows PowerShell 5.1 读取脚本需 UTF-8 BOM，
  否则中文注释会导致解析错误。
