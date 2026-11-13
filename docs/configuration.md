# 配置、凭据、MCP 与历史任务

## 1. 全局配置

文件位置：`%APPDATA%/AgentControl/config.json`（写入采用「临时文件 + rename」原子替换，失败保留原文件）。

| 字段 | 说明 | 默认值 |
|---|---|---|
| `npmRegistry` | npm 镜像源 | `https://registry.npmmirror.com` |
| `proxy` | HTTP 代理（下载官方包时使用） | 空 |
| `downloadDir` | 下载目录 | `%APPDATA%/AgentControl/downloads` |
| `detectTimeoutSecs` | 单目标探测超时（3~60） | 8 |
| `installTimeoutMinutes` | 安装超时 | 30 |
| `projects` | 项目列表 | 空 |
| `agentOverrides` | 单智能体安装源覆盖 | 空 |
| `credentials` | 凭据引用（不含明文） | 空 |

界面位置：**设置**页。切换镜像源后建议重新扫描一次，让 npm 型智能体的版本信息刷新。

## 2. 项目

| 字段 | 说明 |
|---|---|
| `id` | uuid |
| `name` | 项目名称 |
| `workspace` | 工作区根目录（保存时校验存在性，如 `E:/work/demo`） |
| `agents` | 绑定的智能体 id 列表 |
| `defaultModel` | 默认模型（可选） |
| `extraArgs` | 附加启动参数（可选） |
| `notes` | 备注 |
| `createdAt` | 创建时间戳（毫秒） |

界面位置：**项目**页，左侧列表 + 右侧表单，工作区路径通过系统目录选择器选取。

## 3. 凭据（API Key / Token）

- 密钥本体写入 **Windows 凭据管理器**（keyring，服务名 `AgentControl`），由 DPAPI 加密；
- `config.json` 只保存 `id` / `provider` / `label` / `masked` / `createdAt`，**不落盘明文**；
- 掩码规则：保留前 3 位与后 4 位，长度小于等于 8 时显示 `********`；
- 日志与前端回显一律使用掩码，禁止打印明文。

内置凭据服务商工：`openai`、`anthropic`、`google`、`ark`（火山方舟）、`dashscope`（阿里百炼）、`workbuddy`。

界面位置：**凭据与 MCP** 页 → 凭据 Tab。

## 4. MCP 配置

- 每个智能体在清单中声明自己的 MCP 配置文件路径，例如
  `~/.codebuddy/mcp.json`（CodeBuddy）、`~/.codex/config.toml`（Codex，TOML）、
  `~/.gemini/settings.json`、`~/.config/opencode/opencode.json`；
- 路径支持 `~` 与 `%VAR%` 展开；
- 读取：文件不存在时返回默认模板 `{"mcpServers": {}}` 并标记 `exists=false`；
- 保存：先做 JSON 语法校验，校验失败返回错误且不落盘；通过后原子写回；
- 当前实现仅支持 JSON 格式，TOML 文件会直接报错提示。

界面位置：**凭据与 MCP** 页 → MCP Tab（左选智能体，右 JSON 编辑器 + 校验结果）。

## 5. 历史任务

- 存储：`%APPDATA%/AgentControl/history.db`（SQLite，表 `tasks`，索引 `idx_tasks_ts`）；
- 记录字段：`id`、`ts`、`kind`（detect/install/upgrade/uninstall）、`agent_id`、`agent_name`、
  `target`、`source`、`result`（success/failed/cancelled）、`duration_ms`、`message`；
- 查询：`list_tasks(limit?, kind?)`，默认 200 条，上限 1000；`clear_tasks` 清空。

界面位置：**历史**页，可按类型过滤、展开查看错误摘要。

## 6. 安全边界

| 约束 | 实现 |
|---|---|
| 命令注入 | 所有外部命令以参数数组传参，不拼接 shell 字符串；`.cmd` / `.bat` 通过 `cmd /C` 且参数仍然分离 |
| 敏感信息 | API Key 不落盘、不打印，仅掩码展示 |
| 文件安全 | 配置与 MCP 写入使用临时文件 + rename，失败保留原文件 |
| 权限最小化 | `capabilities/default.json` 只声明 `core:default`、`opener:default`、`dialog:default` |
| 安装副作用 | 所有安装动作均由用户在界面显式点击触发，安装任务串行执行，可随时取消 |
