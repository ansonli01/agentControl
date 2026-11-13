# AgentControl · 多智能体控制器实施计划

> 本文件是本次开发任务的落地计划，代码实现严格按此执行。

## 一、产品目标

一个基于 Rust + Vue 的 Windows 桌面端智能体管家：

1. 识别本机已安装的各类智能体 CLI 与桌面客户端（名称、版本、路径、来源）。
2. 下载安装对应智能体的 CLI（npm 型走国内镜像，官方包型走官网直链）。
3. 统一管理 API Key、MCP 配置、工作区路径与历史任务记录。

首批覆盖：OpenCode、WorkBuddy（腾讯桌面 Agent）、豆包；并预留 Codex、Claude Code、Gemini CLI、Qwen Code、TRAE、Cursor、CodeBuddy 的接入扩展。
界面为 WorkBuddy 式工作台骨架（深色、左侧导航 + 中部工作区 + 右侧详情抽屉）。

## 二、技术栈

| 层 | 选型 | 说明 |
|----|------|------|
| 桌面框架 | Tauri v2 | Rust 后端 + 系统 WebView2，进程与文件权限可控 |
| 后端 | Rust 1.96 | tokio / serde / reqwest(rustls) / rusqlite(bundled) / keyring / sha2 / dirs / chrono / winreg |
| 前端 | Vue 3 + TS | Vite + Pinia + Vue Router + TailwindCSS + lucide-vue-next + vue-echarts |
| 落盘 | 用户配置目录 AgentControl | config.json（配置）、history.db（历史）；密钥进 Windows 凭据管理器 |
## 三、模块划分

```
src-tauri/src/
  agent/      清单模型与内置条目（registry 可插拔）
  detect/     多源并发探测：PATH / npm 全局包 / 注册表 / 安装目录
  install/    npm 镜像安装、官方包下载校验、串行队列与流式日志
  config/     配置读写、凭据库、MCP 配置、工作区校验
  tasks/      SQLite 历史任务
  runtime/    子进程封装（流式输出、超时、取消）
  commands/   Tauri IPC 命令
src/
  views/      总览 / 智能体 / 项目 / 凭据与 MCP / 历史 / 设置
  components/ AgentCard / InstallDrawer / StatusTag / EmptyState
  stores/     agents / config / tasks
  api/        IPC 封装与事件订阅
```

## 四、关键决策

1. **清单驱动**：新增智能体只加一条 `AgentDefinition`，不改业务代码。
2. **豆包 / WorkBuddy 无官方 CLI**：已核实 WorkBuddy 仅有桌面客户端与开放平台（open.workbuddy.cn），豆包无官方 CLI 包；因此二者走「官方包直链 + API Key 配置」路径，直链可在设置页自定义，绝不写死未证实包名与仿冒域名。
3. **探测性能**：一次 `npm ls -g --depth=0 --json` 覆盖全部 npm 型智能体，避免逐个 `--version`（O(n) 进程调用 → O(1)）。
4. **安装串行、探测并行**：npm 全局安装存在目录锁，安装任务进入单消费者队列；探测不排队。
5. **安全边界**：外部命令一律参数数组传参，不拼 shell 字符串；API Key 只进系统凭据库，日志与回显一律掩码；配置文件原子写入（临时文件 + rename）。
6. **预留能力**：`runtime/process.rs` 已封装「以某智能体 CLI + 工作区 + 参数启动子进程并流式输出」，后续做统一会话与多智能体编排无需重写进程层。

## 五、执行步骤

1. 工程骨架（Tauri v2 + Vue3 + TS + Tailwind + 图标 + 权限声明）
2. 智能体清单模型与内置条目
3. 探测模块（多源并发 + 版本降级 + 缓存）
4. 安装模块（镜像安装 / 官方包下载校验 / 串行队列 / 流式日志）
5. 配置、凭据、MCP 与历史任务服务
6. IPC 命令与前端事件契约
7. 前端六视图与配套组件
8. 实机验证（cargo test / npm run build / tauri dev）并补文档

## 六、验收标准

- `cargo check` / `cargo test` 零错误；`npm run build`（vue-tsc + vite）零错误。
- 打开应用后一键扫描能识别本机已安装的 @openai/codex 与 OpenCode 桌面版等真实产物。
- 安装任务串行执行，日志与阶段进度实时推送前端，失败给出可复制的命令或手动下载链接。
- API Key 明文不落盘，界面仅显示掩码；MCP 配置写入真实智能体配置文件且 JSON 合法。
- 历史任务落库可查，崩溃后重启仍在。

## 七、实施状态（2026-11-11 已完成）

| 步骤 | 状态 | 说明 |
|---|---|---|
| 1 工程骨架 | 完成 | Tauri v2 + Vue3 + TS + Tailwind + 自定义图标 + 最小权限声明 |
| 2 智能体清单 | 完成 | 11 个内置条目 + 用户覆盖合并 + 单元测试 |
| 3 探测模块 | 完成 | 并发探测 + 版本降级 + 缓存，实机命中 4/11 |
| 4 安装模块 | 完成 | npm 镜像安装 / 官方包下载校验 / 串行队列 / 流式日志 |
| 5 配置与历史 | 完成 | 配置原子写、Windows 凭据库、MCP 读写、SQLite 历史 |
| 6 IPC 契约 | 完成 | 17 个命令 + 3 个事件，前后端类型对齐 |
| 7 前端界面 | 完成 | 总览 / 智能体 / 项目 / 凭据与 MCP / 历史 / 设置 六视图 |
| 8 实机验证与文档 | 完成 | `cargo test` 14 passed、`npm run build` 通过、release 可运行；README + docs/ + .loop/ 齐备 |

验收对照：`cargo test` 零错误 ✅；`npm run build` 零错误 ✅；一键扫描识别到 `@openai/codex 0.138.0`、WorkBuddy 5.9.2、豆包 2.25.18、OpenCode 桌面版 ✅；
API Key 只进系统凭据库、界面掩码 ✅；MCP 配置写回真实文件且 JSON 合法 ✅；历史任务落库 ✅。

配套文档：[README.md](README.md)、[docs/architecture.md](docs/architecture.md)、[docs/agents.md](docs/agents.md)、[docs/configuration.md](docs/configuration.md)、[.loop/](.loop/)。