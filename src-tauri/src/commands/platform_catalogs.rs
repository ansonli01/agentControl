//! 多平台能力目录：豆包工作、小米智能体的技能与插件。
//!
//! 与 [`super::workbuddy_catalog`]（WorkBuddy 专家 / 技能 / 连接器）并列，
//! 这里负责另外两个本机已安装的智能体客户端：
//!
//! - **豆包工作**（DoubaoWork）：技能在
//!   `%LOCALAPPDATA%\DoubaoWork\User Data\Default\.doubaowork\agent_mode\workspace\.skills\<skill>\SKILL.md`，
//!   插件（本地工具包 DLC）在 `sandbox_envs_dir\.prepared_packages\*\dlcs\*` 与技能声明的
//!   `metadata.requires.bins`。
//! - **小米智电能**（Xiaomi MiMo）：技能在
//!   `%APPDATA%\Xiaomi MiMo\engine-config\skills\<skill>\SKILL.md`，
//!   插件在 `engine-config\mimocode.json` 的 `plugin` 数组与 `resources\runtimes\win32-x64\*`（MCP 服务器）。

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use super::workbuddy_catalog::CatalogSource;

/// 平台下的技能条目。
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PlatformSkill {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    /// 图标（小米技能用 `brand:xxx` 形式，豆包多为空）。
    pub icon: Option<String>,
    pub version: Option<String>,
    /// 分类（按名称前缀归类）。
    pub category: Option<String>,
    /// `内置` / `用户自建`。
    pub origin: Option<String>,
    /// 本机路径。
    pub path: Option<String>,
}

/// 平台下的插件条目。
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PlatformPlugin {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    /// `cli` / `mcp` / `tool`。
    pub kind: Option<String>,
    /// 插件提供的工具名。
    pub tools: Vec<String>,
    pub source: Option<String>,
    pub path: Option<String>,
}

/// 单个平台的技能与插件目录。
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PlatformCatalog {
    /// `doubao-work` / `xiaomi-mimo`。
    pub id: String,
    /// 展示名。
    pub name: String,
    pub skills: Vec<PlatformSkill>,
    pub plugins: Vec<PlatformPlugin>,
    pub sources: Vec<CatalogSource>,
}



/// 本机应用数据目录（`%LOCALAPPDATA%` / `%APPDATA%`）。
fn app_data_dir(local: bool) -> Option<PathBuf> {
    let key = if local { "LOCALAPPDATA" } else { "APPDATA" };
    std::env::var_os(key).map(PathBuf::from)
}

/// 极简 YAML frontmatter 解析：返回扁平的 `key -> value`。
///
/// 只处理 `key: value` 这种单行标量（值可带引号），够用来取
/// `name` / `description` / `icon` / `version` / `bins`。
fn parse_frontmatter(text: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    let mut inside = false;
    for (idx, line) in text.lines().enumerate() {
        let trimmed = line.trim_end();
        if idx == 0 {
            if trimmed.trim() != "---" {
                return map;
            }
            inside = true;
            continue;
        }
        if !inside {
            continue;
        }
        if trimmed.trim() == "---" {
            break;
        }
        // 列表项（`- xxx`）与嵌套块不单独处理。
        if trimmed.trim_start().starts_with('-') {
            continue;
        }
        if trimmed.starts_with(' ') {
            // 缩进行：属于上级字段的子项（metadata.requires.bins 等），仅保留常见标量键。
            let inner = trimmed.trim();
            if let Some(pos) = inner.find(':') {
                let key = inner[..pos].trim().to_string();
                let value = inner[pos + 1..].trim().trim_matches('"').to_string();
                if key == "version" || key == "bins" {
                    map.insert(key, value);
                }
            }
            continue;
        }
        if let Some(pos) = trimmed.find(':') {
            let key = trimmed[..pos].trim().to_string();
            let value = trimmed[pos + 1..].trim().trim_matches('"').to_string();
            if !key.is_empty() {
                map.insert(key, value);
            }
        }
    }
    map
}

/// 读取目录下的技能（每个子目录一个 `SKILL.md`）。
fn collect_skills(root: &Path, platform: &str) -> Vec<PlatformSkill> {
    let mut skills = Vec::new();
    let Ok(entries) = fs::read_dir(root) else {
        return skills;
    };
    for entry in entries.flatten() {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let manifest = dir.join("SKILL.md");
        let Ok(text) = fs::read_to_string(&manifest) else {
            continue;
        };
        let meta = parse_frontmatter(&text);
        let id = dir
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let name = meta.get("name").cloned().unwrap_or_else(|| id.clone());
        let description = meta
            .get("description")
            .map(|d| d.trim().to_string())
            .filter(|d| !d.is_empty());
        skills.push(PlatformSkill {
            id: format!("{platform}:{id}"),
            name,
            description,
            icon: meta.get("icon").cloned(),
            version: meta.get("version").cloned(),
            category: Some(skill_category(platform, &id)),
            origin: Some(if id.starts_with('.') { "用户自建".into() } else { "内置".into() }),
            path: Some(manifest.to_string_lossy().to_string()),
        });
    }
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    skills
}

/// 按名称前缀推断技能分类。
fn skill_category(platform: &str, id: &str) -> String {
    if platform == "doubao-work" {
        return match id.split('-').next().unwrap_or_default() {
            "lark" => "飞书",
            "doubao" => "豆包",
            "byted" => "字节跳动",
            "seedream" => "图像生成",
            "browser" | "computer" | "general" => "电脑操作",
            _ => "通用",
        }
        .to_string();
    }
    match id {
        "imagegen" | "3d-creation" | "figma" | "visualizer" => "设计创作",
        "websearch" => "信息检索",
        "session-chat" => "会话",
        "lieflat-less-ai-tone" => "写作",
        "threejs-game-skills" => "开发",
        "windows-powershell-reference" => "系统",
        id if id.starts_with("mimo-") => "小米智电能",
        _ => "通用",
    }
    .to_string()
}

/// 收集豆包工作的技能与插件。
fn collect_doubao_work() -> PlatformCatalog {
    let mut catalog = PlatformCatalog {
        id: "doubao-work".into(),
        name: "豆包工作".into(),
        ..Default::default()
    };
    let Some(local) = app_data_dir(true) else {
        return catalog;
    };
    let workspace = local
        .join("DoubaoWork")
        .join("User Data")
        .join("Default")
        .join(".doubaowork")
        .join("agent_mode")
        .join("workspace");

    let mut skills = collect_skills(&workspace.join(".skills"), "doubao-work");
    skills.extend(collect_skills(&workspace.join(".user_skills"), "doubao-work"));
    let skill_count = skills.len();

    // 插件：技能声明依赖的本地工具（metadata.requires.bins）+ 沙箱已下载的 DLC 工具包。
    let mut plugins: Vec<PlatformPlugin> = Vec::new();
    let mut seen: BTreeMap<String, PlatformPlugin> = BTreeMap::new();
    let mut bins: Vec<String> = Vec::new();
    for entry in fs::read_dir(workspace.join(".skills")).into_iter().flatten().flatten() {
        let manifest = entry.path().join("SKILL.md");
        if let Ok(text) = fs::read_to_string(&manifest) {
            let meta = parse_frontmatter(&text);
            if let Some(raw) = meta.get("bins") {
                for bin in raw
                    .trim_start_matches('[')
                    .trim_end_matches(']')
                    .split(',')
                    .map(|s| s.trim().trim_matches('"').to_string())
                    .filter(|s| !s.is_empty())
                {
                    if !bins.iter().any(|b| b == &bin) {
                        bins.push(bin);
                    }
                }
            }
        }
    }
    for bin in bins {
        let id = format!("doubao-work:bin:{bin}");
        seen.entry(id.clone()).or_insert_with(|| PlatformPlugin {
            id,
            name: bin.clone(),
            description: Some(format!("{bin} 本地工具包")),
            icon: None,
            kind: Some("cli".into()),
            source: Some("技能依赖".into()),
            ..Default::default()
        });
    }
    let dlcs = workspace
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .unwrap_or_default()
        .join("sandbox_envs_dir")
        .join(".prepared_packages");
    if let Ok(packages) = fs::read_dir(&dlcs) {
        for package in packages.flatten() {
            let Ok(items) = fs::read_dir(package.path().join("dlcs")) else {
                continue;
            };
            for item in items.flatten() {
                let path = item.path();
                // 目录名是 `<序号>-<哈希>`，真实工具名取其中的可执行文件名。
                let tool = fs::read_dir(&path)
                    .ok()
                    .into_iter()
                    .flatten()
                    .flatten()
                    .find_map(|f| {
                        let file_name = f.file_name().to_string_lossy().to_string();
                        file_name
                            .strip_suffix(".exe")
                            .map(|s| s.to_string())
                            .or_else(|| {
                                path.file_stem().map(|s| s.to_string_lossy().to_string())
                            })
                    });
                let Some(name) = tool else {
                    continue;
                };
                if seen.values().any(|p| p.name == name) {
                    continue;
                }
                let id = format!("doubao-work:dlc:{name}");
                seen.entry(id.clone()).or_insert_with(|| PlatformPlugin {
                    id,
                    name: name.clone(),
                    description: Some("豆包工作本地工具包（DLC）".into()),
                    icon: None,
                    kind: Some("cli".into()),
                    source: Some("沙箱工具包".into()),
                    path: Some(path.to_string_lossy().to_string()),
                    ..Default::default()
                });
            }
        }
    }
    plugins.extend(seen.into_values());

    catalog.sources.push(CatalogSource {
        id: "skills".into(),
        label: "技能".into(),
        kind: "local".into(),
        endpoint: workspace.join(".skills").to_string_lossy().to_string(),
        ok: skill_count > 0,
        count: skill_count,
        note: Some("豆包工作已安装技能（SKILL.md）".into()),
    });
    catalog.sources.push(CatalogSource {
        id: "plugins".into(),
        label: "插件".into(),
        kind: "local".into(),
        endpoint: workspace.to_string_lossy().to_string(),
        ok: !plugins.is_empty(),
        count: plugins.len(),
        note: Some("技能依赖工具 + 沙箱 DLC 工具包".into()),
    });
    catalog.skills = skills;
    catalog.plugins = plugins;
    catalog
}

/// 收集小米智能体的技能与插件。
fn collect_xiaomi_mimo() -> PlatformCatalog {
    let mut catalog = PlatformCatalog {
        id: "xiaomi-mimo".into(),
        name: "小米智电能".into(),
        ..Default::default()
    };
    let Some(roaming) = app_data_dir(false) else {
        return catalog;
    };
    let root = roaming.join("Xiaomi MiMo");
    let engine = root.join("engine-config");

    let skills = collect_skills(&engine.join("skills"), "xiaomi-mimo");
    let skill_count = skills.len();

    let mut plugins: Vec<PlatformPlugin> = Vec::new();
    // ① engine-config/mimocode.json 里声明的插件（file:// 形式的 mjs 包）。
    if let Ok(text) = fs::read_to_string(engine.join("mimocode.json")) {
        if let Ok(doc) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(list) = doc.get("plugin").and_then(|v| v.as_array()) {
                for (idx, item) in list.iter().enumerate() {
                    let Some(pair) = item.as_array() else {
                        continue;
                    };
                    let Some(first) = pair.first() else {
                        continue;
                    };
                    let path = first.as_str().unwrap_or_default().to_string();
                    let name = path
                        .rsplit('/')
                        .next()
                        .map(|s| s.split('~').next().unwrap_or(s).to_string())
                        .unwrap_or_else(|| format!("插件 {idx}"));
                    let tools = pair
                        .get(1)
                        .and_then(|v| v.get("tools"))
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|t| t.as_str().map(|s| s.to_string()))
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                    plugins.push(PlatformPlugin {
                        id: format!("xiaomi-mimo:plugin:{name}"),
                        name,
                        description: Some(format!("内置插件，提供 {} 个工具", tools.len())),
                        icon: None,
                        kind: Some("tool".into()),
                        tools,
                        source: Some("内置插件".into()),
                        path: Some(path),
                    });
                }
            }
        }
    }
    // ② runtime-res 里下发的插件包。
    let releases = root.join("runtime-res").join("releases").join("main");
    if let Ok(entries) = fs::read_dir(&releases) {
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(name) = path.file_stem().map(|s| s.to_string_lossy().to_string()) else {
                continue;
            };
            if !name.starts_with("plugin") {
                continue;
            }
            let label = name.split('~').next().unwrap_or(&name).to_string();
            plugins.push(PlatformPlugin {
                id: format!("xiaomi-mimo:runtime:{label}"),
                name: label,
                description: Some("小米智电能运行时插件包".into()),
                icon: None,
                kind: Some("tool".into()),
                tools: Vec::new(),
                source: Some("运行时下发".into()),
                path: Some(path.to_string_lossy().to_string()),
            });
        }
    }
    // ③ MCP 服务器形式的插件（resources/runtimes/win32-x64/*）。
    let runtimes = root.join("resources").join("runtimes").join("win32-x64");
    if let Ok(entries) = fs::read_dir(&runtimes) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let Some(name) = path.file_name().map(|s| s.to_string_lossy().to_string()) else {
                continue;
            };
            plugins.push(PlatformPlugin {
                id: format!("xiaomi-mimo:mcp:{name}"),
                name: name.clone(),
                description: Some("MCP 服务器插件".into()),
                icon: None,
                kind: Some("mcp".into()),
                tools: Vec::new(),
                source: Some("本地运行时".into()),
                path: Some(path.to_string_lossy().to_string()),
            });
        }
    }

    catalog.sources.push(CatalogSource {
        id: "skills".into(),
        label: "技能".into(),
        kind: "local".into(),
        endpoint: engine.join("skills").to_string_lossy().to_string(),
        ok: skill_count > 0,
        count: skill_count,
        note: Some("小米智电能内置技能（SKILL.md）".into()),
    });
    catalog.sources.push(CatalogSource {
        id: "plugins".into(),
        label: "插件".into(),
        kind: "local".into(),
        endpoint: engine.join("mimocode.json").to_string_lossy().to_string(),
        ok: !plugins.is_empty(),
        count: plugins.len(),
        note: Some("mimocode.json 插件声明 + 运行时插件包 + MCP 服务器".into()),
    });
    catalog.skills = skills;
    catalog.plugins = plugins;
    catalog
}

/// 汇总各平台的技能与插件目录。
#[tauri::command]
pub async fn platform_catalogs() -> Vec<PlatformCatalog> {
    let mut catalogs = Vec::new();
    catalogs.push(collect_doubao_work());
    catalogs.push(collect_xiaomi_mimo());
    catalogs
}

/// 供单元测试使用：豆包工作技能目录。
#[cfg(test)]
pub fn doubao_skills_dir() -> Option<PathBuf> {
    app_data_dir(true).map(|local| {
        local
            .join("DoubaoWork")
            .join("User Data")
            .join("Default")
            .join(".doubaowork")
            .join("agent_mode")
            .join("workspace")
            .join(".skills")
    })
}

/// 供单元测试使用：小米智电能技能目录。
#[cfg(test)]
pub fn xiaomi_skills_dir() -> Option<PathBuf> {
    app_data_dir(false).map(|roaming| {
        roaming
            .join("Xiaomi MiMo")
            .join("engine-config")
            .join("skills")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontmatter_is_parsed() {
        let text = "---\nname: lark-doc\ndescription: \"飞书文档\"\nmetadata:\n  version: 1.0.48\n  requires:\n    bins: [\"lark-cli\"]\n---\n\n# body\n";
        let meta = parse_frontmatter(text);
        assert_eq!(meta.get("name").map(|s| s.as_str()), Some("lark-doc"));
        assert_eq!(meta.get("version").map(|s| s.as_str()), Some("1.0.48"));
        assert_eq!(
            meta.get("bins").map(|s| s.as_str()),
            Some("[\"lark-cli\"]")
        );
    }

    #[test]
    fn doubao_skills_are_collected() {
        let Some(dir) = doubao_skills_dir() else {
            return;
        };
        if !dir.exists() {
            return;
        }
        let skills = collect_skills(&dir, "doubao-work");
        assert!(!skills.is_empty(), "应当解析到豆包工作技能");
        assert!(skills.iter().any(|s| s.name == "lark-doc"));
    }

    #[test]
    fn xiaomi_skills_are_collected() {
        let Some(dir) = xiaomi_skills_dir() else {
            return;
        };
        if !dir.exists() {
            return;
        }
        let skills = collect_skills(&dir, "xiaomi-mimo");
        assert!(!skills.is_empty(), "应当解析到小米智电能技能");
    }
}
