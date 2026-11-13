# 智能体清单与扩展

所有探测与安装逻辑都只面向一份内置清单（`src-tauri/src/agent/registry.rs`）编程，
新增一个智能体 = 追加一条 `AgentDefinition`，不需要改动任何业务代码。

## 1. 内置清单

| id | 名称 | 厂商 | 形态 | 安装方式 | npm 包 | 官网 / 下载入口 |
|---|---|---|---|---|---|---|
| `opencode` | OpenCode | SST | CLI | Npm | `opencode-ai` | https://opencode.ai |
| `codex` | Codex CLI | OpenAI | CLI | Npm | `@openai/codex` | https://developers.openai.com/codex/cli |
| `claude-code` | Claude Code | Anthropic | CLI | Npm | `@anthropic-ai/claude-code` | https://docs.anthropic.com/en/docs/claude-code/overview |
| `gemini-cli` | Gemini CLI | Google | CLI | Npm | `@google/gemini-cli` | https://github.com/google-gemini/gemini-cli |
| `qwen-code` | Qwen Code | 阿里云 | CLI | Npm | `@qwen-code/qwen-code` | https://github.com/QwenLM/qwen-code |
| `workbuddy` | WorkBuddy | 腾讯 | 桌面端 | Official | — | https://www.workbuddy.cn |
| `doubao` | 豆包 | 字节跳动 | 桌面端 | Official | — | https://www.doubao.com |
| `opencode-desktop` | OpenCode 桌面版 | SST | 桌面端 | Manual | — | https://opencode.ai/download |
| `codebuddy` | CodeBuddy | 腾讯 | 桌面端 | Manual | — | https://www.codebuddy.ai |
| `trae` | TRAE | 字节跳动 | 桌面端 | Official | — | https://www.trae.cn |
| `cursor` | Cursor | Anysphere | 桌面端 | Manual | — | https://www.cursor.com/downloads |

### 关于 WorkBuddy 与豆包

- **WorkBuddy** 是腾讯的桌面 AI Agent 工作台（`workbuddy.cn` / `open.workbuddy.cn`），**没有官方 CLI 或 npm 包**，
  因此只能走「官方客户端下载 + API Key 配置」路径。
- **豆包** 同样没有官方 CLI，Windows 客户端主要通过 Microsoft Store 分发。
- 两者的官方安装包直链目前没有可核实的公开地址（且存在 `workbuddy-sy` / `workbuddy-by.com.cn` 等仿冒域名），
  因此清单默认**只指向官网入口**，并在设置页支持手动填入直链（见下节），不写死未证实的包名。

## 2. 清单字段

```rust
AgentDefinition {
    id, name, vendor, kind,          // Cli | Desktop
    description, tags,
    cli_names: Vec<String>,          // 可执行文件名 / 桌面端匹配关键词
    version_args: Vec<String>,       // 默认 ["--version"]
    npm_package: Option<String>,     // npm 型：包名
    official_download: Option<String>, // 官方包型：直链（默认 None，走官网入口）
    official_page: Option<String>,   // 官网 / 下载页
    open_platform: Option<String>,   // 开放平台地址
    silent_args: Vec<String>,        // 静默安装参数
    mcp_config_path: Option<String>,  // MCP 配置文件路径
    mcp_config_format: Option<ConfigFormat>, // Json | Toml
    config_path: Option<String>,
    credential_provider: Option<String>,     // 关联的凭据服务商工
    icon_letter, accent,             // 卡片展示
    install_method: InstallMethod,   // Npm | Official | Manual
    builtin: bool,
}
```

### 三种安装方式

| InstallMethod | 行为 |
|---|---|
| `Npm` | `npm -g install|update|uninstall <pkg> --registry <镜像>`，日志实时推送 |
| `Official` | 按 `official_download` 直链流式下载 → SHA-256 校验（可选期望值）→ 按 `silent_args` 静默安装；未填直链时提示手动下载 |
| `Manual` | 不做自动安装，仅提供官网入口与「手动安装」提示 |

## 3. 用户覆盖（不改代码即可调整安装源）

「设置 → 智能体安装源」保存的 `AgentOverride` 会与内置清单合并（`merge_overrides`），可覆盖：

```rust
AgentOverride {
    agent_id,
    official_download,   // 官方安装包直链
    official_page,       // 官网入口
    npm_package,         // npm 包名
    npm_registry,        // 单独指定镜像源
    silent_args,         // 静默安装参数
    enabled,             // 是否参与探测
}
```

例如把豆包指向 Microsoft Store 之外的直链，只需在界面填入地址并保存，无需重新编译。

## 4. 新增一个智能体

1. 在 `src-tauri/src/agent/registry.rs` 的 `builtin_agents()` 中追加一条 `AgentDefinition`；
2. 如需独立凭据服务商工，在 `providers()` 追加一条 `Provider`；
3. 前端无需改动——清单由 `list_agents` 命令动态下发，卡片、图标字母、状态标签自动生成；
4. 若为 npm 型，安装走默认镜像；建议补充单元测试断言（参考 `registry.rs` 内既有测试）。

## 5. 探测规则

| 形态 | 探测来源 |
|---|---|
| CLI | ① `npm ls -g --depth=0 --json` 解析包名与版本；② PATH + `npm prefix -g` + `%APPDATA%\npm` 扫描可执行文件；③ 执行 `<cli> --version` 取版本（超时 8s，失败降级为「版本未知」） |
| 桌面端 | ① `HKCU` / `HKLM` / `WOW6432Node` 卸载注册表（`DisplayName` / `DisplayVersion` / `InstallLocation`）；② `%LOCALAPPDATA%\Programs`、`Program Files`、`Program Files (x86)` 目录名匹配；③ 关键词归一化后匹配，`OpenCode Desktop` 可命中 `@opencodedesktop` |

探测全程并发执行，单目标失败只影响该目标；结果带 `sources` 字段说明命中来源，便于用户判断依据。
