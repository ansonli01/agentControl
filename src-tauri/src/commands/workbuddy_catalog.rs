//! WorkBuddy 能力目录：技能、连接器、专家团。
//!
//! 数据来源分三类：
//! - **公网 API**：专家团市场 `expert_center.json`（腾讯 COS 静态资源，公开可访问）。
//! - **WorkBuddy API 缓存**：技能列表 `~/.workbuddy/.skill-list-cache.json`
//!   （WorkBuddy 自己从 `api.skillhub.cn` 的 knot 网关拉取后的缓存；该网关需要登录态，
//!   令牌在本机是密文、无法解密还原，因此直接复用 WorkBuddy 的缓存）。
//! - **本地目录**：已安装技能 `~/.workbuddy/skills/*`、连接器市场
//!   `~/.workbuddy/connectors-marketplace/connectors/*`、本机专家团 `~/.workbuddy/teams/*`。

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::Serialize;
use serde_json::Value;

/// 官方专家中心清单（151 位专家，含头像、分类、标签）。
const EXPERT_CENTER_URL: &str =
    "https://acc-1258344699.cos.accelerate.myqcloud.com/workbuddy/experts/expert_center.json";
/// 专家中心静态资源基址（头像相对路径拼接用）。
const EXPERT_CENTER_BASE: &str =
    "https://acc-1258344699.cos.accelerate.myqcloud.com/workbuddy/experts";
/// 扩展专家市场清单（429 位，含团队型专家与 OPC 标记）。
const EXPERT_MARKETPLACE_URL: &str =
    "https://acc-1258344699.cos.accelerate.myqcloud.com/workbuddy/expert-marketplace/expert_center.json";
/// 扩展专家市场静态资源基址。
const EXPERT_MARKETPLACE_BASE: &str =
    "https://acc-1258344699.cos.accelerate.myqcloud.com/workbuddy/expert-marketplace";
/// 专家团市场精选场景。
const EXPERT_SCENES_URL: &str =
    "https://acc-1258344699.cos.accelerate.myqcloud.com/workbuddy/expert-marketplace/featuredScenes.json";

/// 技能条目。
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CatalogSkill {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    /// 来源（`userSettings` / `builtin` / `marketplace` 等）。
    pub source: Option<String>,
    /// 所属市场（`skillhub` / `marketplace`）。
    pub marketplace: Option<String>,
    pub namespace: Option<String>,
    pub slug: Option<String>,
    pub version: Option<String>,
    /// 是否已安装到本机。
    pub installed: bool,
    pub file_path: Option<String>,
    pub installed_at: Option<i64>,
    pub tags: Vec<String>,
    /// 图标地址（CDN / 本地文件）。
    pub icon: Option<String>,
    /// 使用次数（市场统计）。
    pub use_count: Option<i64>,
    /// 安装次数（市场统计）。
    pub install_count: Option<i64>,
    /// 市场分类（`content-creation` 等）。
    pub categories: Vec<String>,
    /// 市场更新时间（ISO 字符串）。
    pub updated_at: Option<String>,
}

/// 连接器条目。
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CatalogConnector {
    pub id: String,
    pub name: String,
    /// `streamableHttp` / `sse` / `stdio` 等。
    pub kind: Option<String>,
    pub url: Option<String>,
    pub command: Option<String>,
    /// 是否已在本机接入。
    pub enabled: bool,
}

/// 市场专家。
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CatalogExpert {
    pub id: String,
    /// 中文名（缺失时回退英文）。
    pub name: String,
    pub name_en: Option<String>,
    /// 职业定位。
    pub profession: Option<String>,
    pub description: Option<String>,
    /// 所属分类（中文名）。
    pub category: Option<String>,
    /// 分类 id。
    pub category_id: Option<String>,
    pub expert_type: Option<String>,
    pub tags: Vec<String>,
    /// 头像完整地址。
    pub avatar: Option<String>,
    /// 数据来源（`官方专家库` / `扩展专家库` / `本机`）。
    pub source: Option<String>,
    /// 团队型专家的成员数量。
    pub member_count: Option<usize>,
    /// 更新时间（ISO 字符串）。
    pub updated_at: Option<String>,
    /// 使用次数（WorkBuddy 官方「最热」排序字段；公开清单未带，仅登录态接口可取）。
    pub use_count: Option<u64>,
}

/// 精选场景（WorkBuddy 专家中心的推荐位）。
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FeaturedScene {
    pub id: String,
    /// 场景中文名。
    pub name: String,
    pub description: Option<String>,
    /// 场景封面图（相对路径）。
    pub image: Option<String>,
    /// 场景内的专家 id。
    pub expert_ids: Vec<String>,
}

/// 分类（含封面图，用于顶部大卡）。
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CatalogCategory {
    pub id: String,
    pub name: String,
    pub count: usize,
    /// 封面图（该分类下首位专家的头像）。
    pub cover: Option<String>,
    /// 分类下前几个专家名（大卡副标题）。
    pub samples: Vec<String>,
}

/// 专家团成员。
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CatalogTeamMember {
    pub agent_id: Option<String>,
    pub name: String,
    pub role: Option<String>,
    pub agent_type: Option<String>,
    pub model: Option<String>,
    pub workspace: Option<String>,
}

/// 本机专家团（AgentTeams）。
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CatalogTeam {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub lead: Option<String>,
    pub members: Vec<CatalogTeamMember>,
    pub member_count: usize,
    pub created_at: Option<i64>,
}

/// 数据来源说明。
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CatalogSource {
    /// `skills` / `connectors` / `experts` / `teams`。
    pub id: String,
    pub label: String,
    /// `api`（公网接口）或 `local`（本机缓存 / 目录）。
    pub kind: String,
    /// 数据来源地址或路径。
    pub endpoint: String,
    pub ok: bool,
    pub count: usize,
    pub note: Option<String>,
}

/// WorkBuddy 能力目录。
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkBuddyCatalog {
    pub skills: Vec<CatalogSkill>,
    pub connectors: Vec<CatalogConnector>,
    pub experts: Vec<CatalogExpert>,
    pub teams: Vec<CatalogTeam>,
    pub sources: Vec<CatalogSource>,
    /// 专家分类（用于筛选 chips 与顶部大卡）。
    pub expert_categories: Vec<CatalogCategory>,
    /// 技能分类。
    pub skill_categories: Vec<CatalogCategory>,
    /// 本机常用专家（`storage/user-*/personal/global/expert.json`）。
    pub my_experts: Vec<CatalogExpert>,
    /// 精选场景（专家中心的推荐位）。
    pub featured_scenes: Vec<FeaturedScene>,
}

/// 用户主目录。
fn home_dir() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
}

/// WorkBuddy 数据目录。
fn data_dir() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".workbuddy"))
}

/// 读取 JSON 文件（UTF-8）。
fn read_json(path: &Path) -> Option<Value> {
    let text = fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

/// 压缩成一行并截断。
fn summarize(text: &str, limit: usize) -> Option<String> {
    let one_line: String = text
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    if one_line.is_empty() {
        return None;
    }
    if one_line.chars().count() > limit {
        let cut: String = one_line.chars().take(limit).collect();
        Some(format!("{cut}…"))
    } else {
        Some(one_line)
    }
}

/// 取中英双语文本的中文部分。
fn localized(value: Option<&Value>) -> Option<String> {
    let obj = value?.as_object()?;
    obj.get("zh")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| obj.get("en").and_then(|v| v.as_str()).map(|s| s.to_string()))
}

/// 取双语文本字段列表。
fn localized_list(value: Option<&Value>) -> Vec<String> {
    let Some(items) = value.and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|item| {
            item.as_str()
                .map(|s| s.to_string())
                .or_else(|| localized(Some(item)))
        })
        .collect()
}
/// 读取 WorkBuddy 技能市场缓存。
///
/// 位置：`~/.workbuddy/storage/user-<uid>-personal/scoped/<scope>/skill-market.json`，
/// 是 WorkBuddy 自己从 `api.skillhub.cn` 拉取后落盘的首屏市场数据，
/// 内含图标地址、分类、使用/安装次数。
fn collect_skill_market(root: &Path) -> (Vec<CatalogSkill>, Vec<CatalogCategory>) {
    let mut skills: Vec<CatalogSkill> = Vec::new();
    let mut categories: Vec<CatalogCategory> = Vec::new();
    let storage = root.join("storage");
    let Ok(users) = fs::read_dir(&storage) else {
        return (skills, categories);
    };
    for user in users.flatten() {
        // 目录形如 `user-<uid>-personal`，其下直接是 `global/` 与 `scoped/`。
        let scoped = user.path().join("scoped");
        let Ok(scopes) = fs::read_dir(&scoped) else {
            continue;
        };
        for scope in scopes.flatten() {
            let path = scope.path().join("skill-market.json");
            let Some(doc) = read_json(&path) else {
                continue;
            };
            let Some(node) = doc
                .get("first-screen.v1:personal::__general__:fixed:recommend")
            else {
                continue;
            };

            if let Some(cats) = node.get("primaryCategories").and_then(|v| v.as_array()) {
                for cat in cats {
                    let id = cat
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string();
                    if id.is_empty() || categories.iter().any(|c| c.id == id) {
                        continue;
                    }
                    let name = localized(cat.get("name")).unwrap_or_else(|| id.clone());
                    categories.push(CatalogCategory {
                        id,
                        name,
                        ..Default::default()
                    });
                }
            }

            let mut lists: Vec<&Value> = Vec::new();
            if let Some(list) = node.get("primaryList").and_then(|v| v.get("skills")) {
                lists.push(list);
            }
            if let Some(list) = node.get("featured").and_then(|v| v.get("skills")) {
                lists.push(list);
            }
            for list in lists {
                let Some(items) = list.as_array() else {
                    continue;
                };
                for item in items {
                    let slug = item
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string();
                    if slug.is_empty() {
                        continue;
                    }
                    if skills.iter().any(|s| s.slug.as_deref() == Some(slug.as_str())) {
                        continue;
                    }
                    let id = item
                        .get("skill_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&slug)
                        .to_string();
                    let use_stat = item.get("use_stat");
                    skills.push(CatalogSkill {
                        id,
                        name: item
                            .get("display_name_zh")
                            .and_then(|v| v.as_str())
                            .or_else(|| item.get("name").and_then(|v| v.as_str()))
                            .unwrap_or_default()
                            .to_string(),
                        slug: Some(slug),
                        description: item
                            .get("description_zh")
                            .and_then(|v| v.as_str())
                            .or_else(|| item.get("description").and_then(|v| v.as_str()))
                            .and_then(|s| summarize(s, 200)),
                        version: item.get("version").and_then(|v| v.as_str()).map(|s| s.to_string()),
                        icon: item.get("icon").and_then(|v| v.as_str()).map(|s| s.to_string()),
                        use_count: use_stat
                            .and_then(|v| v.get("use_count"))
                            .and_then(|v| v.as_i64()),
                        install_count: use_stat
                            .and_then(|v| v.get("install_count"))
                            .and_then(|v| v.as_i64()),
                        categories: item
                            .get("categories")
                            .and_then(|v| v.as_array())
                            .map(|a| {
                                a.iter()
                                    .filter_map(|c| c.as_str().map(|s| s.to_string()))
                                    .collect()
                            })
                            .unwrap_or_default(),
                        source: Some("marketplace".into()),
                        installed: false,
                        updated_at: item
                            .get("updated_at")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        ..Default::default()
                    });
                }
            }
        }
    }

    for cat in categories.iter_mut() {
        cat.count = skills
            .iter()
            .filter(|s| s.categories.iter().any(|c| c == &cat.id))
            .count();
    }
    (skills, categories)
}

/// 解析技能（WorkBuddy API 缓存 + 本机已安装目录 + 市场缓存补充图标与热度）。
fn collect_skills(root: &Path, market: &[CatalogSkill]) -> (Vec<CatalogSkill>, CatalogSource) {
    let mut skills: Vec<CatalogSkill> = Vec::new();
    let cache_path = root.join(".skill-list-cache.json");
    let mut note = None;

    if let Some(doc) = read_json(&cache_path) {
        if let Some(results) = doc.get("results").and_then(|v| v.as_array()) {
            for item in results {
                let name = item
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                if name.is_empty() {
                    continue;
                }
                let file_path = item
                    .get("filePath")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let installed = file_path
                    .as_ref()
                    .map(|p| Path::new(p).exists())
                    .unwrap_or(false);
                let id = item
                    .get("canonicalName")
                    .and_then(|v| v.as_str())
                    .or_else(|| item.get("slug").and_then(|v| v.as_str()))
                    .unwrap_or(&name)
                    .to_string();
                let tags = item
                    .get("labels")
                    .and_then(|v| v.as_object())
                    .map(|map| {
                        map.iter()
                            .filter(|(_, v)| v.as_bool() == Some(true))
                            .map(|(k, _)| k.clone())
                            .collect()
                    })
                    .unwrap_or_default();
                skills.push(CatalogSkill {
                    id,
                    name,
                    description: item
                        .get("description")
                        .and_then(|v| v.as_str())
                        .and_then(|s| summarize(s, 240)),
                    source: item
                        .get("source")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    marketplace: item
                        .get("marketplaceSource")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    namespace: item
                        .get("namespace")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    slug: item
                        .get("slug")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    version: item
                        .get("version")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    installed,
                    file_path,
                    installed_at: item.get("installedAt").and_then(|v| v.as_i64()),
                    tags,
                    icon: item
                        .get("iconRemoteUrl")
                        .and_then(|v| v.as_str())
                        .filter(|u| u.starts_with("http"))
                        .map(|s| s.to_string()),
                    ..Default::default()
                });
            }
        }
    } else {
        note = Some("未找到技能列表缓存".to_string());
    }

    // 补齐只存在于本地目录、但不在缓存里的技能。
    let skills_dir = root.join("skills");
    if let Ok(entries) = fs::read_dir(&skills_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let dir_name = entry.file_name().to_string_lossy().to_string();
            if dir_name.starts_with('.') {
                continue;
            }
            let skill_file = path.join("SKILL.md");
            if !skill_file.exists() {
                continue;
            }
            let already = skills.iter().any(|s| {
                s.installed
                    && s.file_path
                        .as_ref()
                        .map(|p| p.contains(&dir_name))
                        .unwrap_or(false)
            });
            if already {
                continue;
            }
            let (title, desc) = read_skill_md(&skill_file);
            skills.push(CatalogSkill {
                id: dir_name.clone(),
                name: title.unwrap_or_else(|| dir_name.clone()),
                description: desc,
                source: Some("local".into()),
                installed: true,
                file_path: Some(skill_file.display().to_string()),
                ..Default::default()
            });
        }
    }

    // 合并市场缓存：补齐图标、热度、分类，并把未安装的市场技能一并列出。
    for m in market {
        let matched = m
            .slug
            .as_ref()
            .and_then(|slug| {
                skills
                    .iter_mut()
                    .find(|s| s.slug.as_deref() == Some(slug.as_str()))
            });
        match matched {
            Some(existing) => {
                if existing.icon.is_none() {
                    existing.icon = m.icon.clone();
                }
                if existing.use_count.is_none() {
                    existing.use_count = m.use_count;
                }
                if existing.install_count.is_none() {
                    existing.install_count = m.install_count;
                }
                if existing.categories.is_empty() {
                    existing.categories = m.categories.clone();
                }
                if existing.description.is_none() {
                    existing.description = m.description.clone();
                }
            }
            None => skills.push(m.clone()),
        }
    }

    let count = skills.len();
    (
        skills,
        CatalogSource {
            id: "skills".into(),
            label: "技能".into(),
            kind: "local".into(),
            endpoint: cache_path.display().to_string(),
            ok: count > 0,
            count,
            note,
        },
    )
}

/// 读取 SKILL.md 的 frontmatter 标题与描述。
fn read_skill_md(path: &Path) -> (Option<String>, Option<String>) {
    let Ok(text) = fs::read_to_string(path) else {
        return (None, None);
    };
    let mut title = None;
    let mut description = None;
    for line in text.lines().take(30) {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("name:") {
            title = Some(rest.trim().trim_matches('"').to_string());
        } else if let Some(rest) = line.strip_prefix("description:") {
            description = Some(rest.trim().trim_matches('"').to_string());
        }
    }
    if title.is_none() {
        if let Some(first) = text.lines().find(|l| l.starts_with("# ")) {
            title = Some(first.trim_start_matches("# ").trim().to_string());
        }
    }
    if description.is_none() {
        description = text
            .lines()
            .skip_while(|l| l.trim() != "---")
            .skip(1)
            .find(|l| !l.trim().is_empty())
            .and_then(|l| summarize(l, 240));
    }
    (title, description)
}
/// 解析连接器（市场缓存目录 + 已接入状态）。
fn collect_connectors(root: &Path) -> (Vec<CatalogConnector>, CatalogSource) {
    let mut enabled_ids: Vec<String> = Vec::new();
    let connectors_dir = root.join("connectors");
    if let Ok(entries) = fs::read_dir(&connectors_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let states = read_json(&path.join("connector-states.json"));
            if let Some(map) = states
                .as_ref()
                .and_then(|d| d.get("connectors"))
                .and_then(|v| v.as_object())
            {
                for (id, cfg) in map {
                    let on = cfg
                        .get("enabled")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true);
                    if on {
                        enabled_ids.push(id.clone());
                    }
                }
            }
        }
    }

    let mut connectors: Vec<CatalogConnector> = Vec::new();
    let market_dir = root.join("connectors-marketplace").join("connectors");
    if let Ok(entries) = fs::read_dir(&market_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let id = entry.file_name().to_string_lossy().to_string();
            if id.starts_with('.') {
                continue;
            }
            let mcp = read_json(&path.join("mcp.json"));
            let servers = mcp
                .as_ref()
                .and_then(|d| d.get("mcpServers"))
                .and_then(|v| v.as_object())
                .cloned()
                .unwrap_or_default();
            if servers.is_empty() {
                connectors.push(CatalogConnector {
                    id: id.clone(),
                    name: id.clone(),
                    enabled: enabled_ids.contains(&id),
                    ..Default::default()
                });
                continue;
            }
            for (name, cfg) in servers {
                connectors.push(CatalogConnector {
                    id: id.clone(),
                    name: name.clone(),
                    kind: cfg.get("type").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    url: cfg.get("url").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    command: cfg.get("command").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    enabled: enabled_ids.contains(&id),
                });
            }
        }
    }
    connectors.sort_by(|a, b| a.id.cmp(&b.id));
    let count = connectors.len();
    (
        connectors,
        CatalogSource {
            id: "connectors".into(),
            label: "连接器".into(),
            kind: "local".into(),
            endpoint: market_dir.display().to_string(),
            ok: count > 0,
            count,
            note: None,
        },
    )
}

/// 解析本机专家团。
fn collect_teams(root: &Path) -> (Vec<CatalogTeam>, CatalogSource) {
    let mut teams: Vec<CatalogTeam> = Vec::new();
    let teams_dir = root.join("teams");
    if let Ok(entries) = fs::read_dir(&teams_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let id = entry.file_name().to_string_lossy().to_string();
            if id.starts_with('.') {
                continue;
            }
            let Some(doc) = read_json(&path.join("config.json")) else {
                continue;
            };
            let members: Vec<CatalogTeamMember> = doc
                .get("members")
                .and_then(|v| v.as_array())
                .map(|items| {
                    items
                        .iter()
                        .map(|m| CatalogTeamMember {
                            agent_id: m.get("agentId").and_then(|v| v.as_str()).map(|s| s.to_string()),
                            name: m
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or_default()
                                .to_string(),
                            role: m.get("role").and_then(|v| v.as_str()).map(|s| s.to_string()),
                            agent_type: m
                                .get("agentType")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            model: m.get("model").and_then(|v| v.as_str()).map(|s| s.to_string()),
                            workspace: m.get("cwd").and_then(|v| v.as_str()).map(|s| s.to_string()),
                        })
                        .collect()
                })
                .unwrap_or_default();
            let member_count = members.len();
            teams.push(CatalogTeam {
                name: doc
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&id)
                    .to_string(),
                id,
                description: doc
                    .get("description")
                    .and_then(|v| v.as_str())
                    .and_then(|s| summarize(s, 200)),
                lead: doc
                    .get("leadAgentId")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                members,
                member_count,
                created_at: doc.get("createdAt").and_then(|v| v.as_i64()),
            });
        }
    }
    teams.sort_by(|a, b| a.name.cmp(&b.name));
    let count = teams.len();
    (
        teams,
        CatalogSource {
            id: "teams".into(),
            label: "本机专家团".into(),
            kind: "local".into(),
            endpoint: teams_dir.display().to_string(),
            ok: count > 0,
            count,
            note: None,
        },
    )
}
/// 把相对路径拼成完整地址。
fn absolute(base: &str, path: Option<&str>) -> Option<String> {
    let p = path?.trim();
    if p.is_empty() {
        return None;
    }
    if p.starts_with("http://") || p.starts_with("https://") {
        return Some(p.to_string());
    }
    Some(format!("{}/{}", base.trim_end_matches('/'), p.trim_start_matches('/')))
}

/// 拉取官方专家中心（公网 API）。
///
/// 官方清单：`/workbuddy/experts/expert_center.json`（151 位专家、12 个分类），
/// 头像为相对路径，需拼接 `EXPERT_CENTER_BASE`。
/// 同时拉取扩展专家市场，取其中的团队型专家补齐「专家团」品类。
async fn fetch_experts() -> (Vec<CatalogExpert>, Vec<CatalogCategory>, CatalogSource) {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("AgentControl/0.1")
        .build();
    let mut note = None;
    let mut experts: Vec<CatalogExpert> = Vec::new();
    let mut categories: Vec<CatalogCategory> = Vec::new();

    match client {
        Ok(client) => {
            // 1) 官方专家中心
            if let Ok(resp) = client.get(EXPERT_CENTER_URL).send().await {
                if resp.status().is_success() {
                    if let Ok(doc) = resp.json::<Value>().await {
                        let cat_names: Vec<(String, String)> = doc
                            .get("categories")
                            .and_then(|v| v.as_array())
                            .map(|items| {
                                items
                                    .iter()
                                    .filter_map(|c| {
                                        let id =
                                            c.get("id").and_then(|v| v.as_str())?.to_string();
                                        let name = localized(c.get("name"))
                                            .unwrap_or_else(|| id.clone());
                                        Some((id, name))
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();
                        if let Some(items) = doc.get("experts").and_then(|v| v.as_array()) {
                            for item in items {
                                let id = item
                                    .get("id")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or_default()
                                    .to_string();
                                if id.is_empty() {
                                    continue;
                                }
                                let category_id = item
                                    .get("categoryId")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or_default()
                                    .to_string();
                                let category = cat_names
                                    .iter()
                                    .find(|(cid, _)| cid == &category_id)
                                    .map(|(_, name)| name.clone());
                                experts.push(CatalogExpert {
                                    id,
                                    name: localized(item.get("name")).unwrap_or_default(),
                                    name_en: item
                                        .get("name")
                                        .and_then(|v| v.get("en"))
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string()),
                                    profession: localized(item.get("profession")),
                                    description: localized(item.get("description")),
                                    category,
                                    category_id: Some(category_id),
                                    expert_type: Some("agent".into()),
                                    tags: localized_list(item.get("tags")),
                                    avatar: absolute(
                                        EXPERT_CENTER_BASE,
                                        item.get("avatar").and_then(|v| v.as_str()),
                                    ),
                                    source: Some("官方专家库".into()),
                                    member_count: None,
                    use_count: None,
                                    updated_at: item
                                        .get("updatedAt")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string()),
                                });
                            }
                        }
                        categories = cat_names
                            .into_iter()
                            .map(|(id, name)| CatalogCategory {
                                id,
                                name,
                                ..Default::default()
                            })
                            .collect();
                    }
                } else {
                    note = Some(format!("官方专家中心返回 {}", resp.status()));
                }
            } else if let Err(e) = client.get(EXPERT_CENTER_URL).send().await {
                note = Some(format!("请求失败：{e}"));
            }

            // 2) 扩展专家市场：只取团队型专家（专家团）
            if let Ok(resp) = client.get(EXPERT_MARKETPLACE_URL).send().await {
                if let Ok(doc) = resp.json::<Value>().await {
                    let cat_names: Vec<(String, String)> = doc
                        .get("categories")
                        .and_then(|v| v.as_array())
                        .map(|items| {
                            items
                                .iter()
                                .filter_map(|c| {
                                    let id = c.get("id").and_then(|v| v.as_str())?.to_string();
                                    let name =
                                        localized(c.get("name")).unwrap_or_else(|| id.clone());
                                    Some((id, name))
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    if let Some(items) = doc.get("experts").and_then(|v| v.as_array()) {
                        for item in items {
                            let is_team = item
                                .get("expertType")
                                .and_then(|v| v.as_str())
                                .map(|t| t == "team")
                                .unwrap_or(false)
                                || item.get("members").map(|m| m.is_array()).unwrap_or(false);
                            if !is_team {
                                continue;
                            }
                            let id = item
                                .get("id")
                                .and_then(|v| v.as_str())
                                .unwrap_or_default()
                                .to_string();
                            if id.is_empty() || experts.iter().any(|e| e.id == id) {
                                continue;
                            }
                            let category_id = item
                                .get("categoryId")
                                .and_then(|v| v.as_str())
                                .unwrap_or_default()
                                .to_string();
                            let category = cat_names
                                .iter()
                                .find(|(cid, _)| cid == &category_id)
                                .map(|(_, name)| name.clone());
                            let member_count = item
                                .get("members")
                                .and_then(|v| v.as_array())
                                .map(|a| a.len());
                            experts.push(CatalogExpert {
                                id,
                                name: localized(item.get("displayName")).unwrap_or_default(),
                                name_en: item
                                    .get("displayName")
                                    .and_then(|v| v.get("en"))
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string()),
                                profession: localized(item.get("profession")),
                                description: localized(item.get("description")),
                                category,
                                category_id: Some(category_id),
                                expert_type: Some("team".into()),
                                tags: localized_list(item.get("tags")),
                                avatar: absolute(
                                    EXPERT_MARKETPLACE_BASE,
                                    item.get("avatar").and_then(|v| v.as_str()),
                                ),
                                source: Some("扩展专家库".into()),
                                member_count,
                                updated_at: item
                                    .get("updatedAt")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string()),
                                use_count: item
                                    .get("useCount")
                                    .or_else(|| item.get("usageCount"))
                                    .and_then(|v| v.as_u64()),
                            });
                        }
                    }
                }
            }
        }
        Err(e) => {
            note = Some(format!("客户端初始化失败：{e}"));
        }
    }

    // 分类封面与示例专家（大卡用）
    for cat in categories.iter_mut() {
        let members: Vec<&CatalogExpert> = experts
            .iter()
            .filter(|e| e.category_id.as_deref() == Some(cat.id.as_str()))
            .collect();
        cat.count = members.len();
        cat.cover = members.iter().find_map(|e| e.avatar.clone());
        cat.samples = members
            .iter()
            .take(3)
            .map(|e| e.name.clone())
            .filter(|n| !n.is_empty())
            .collect();
    }

    let count = experts.len();
    (
        experts,
        categories,
        CatalogSource {
            id: "experts".into(),
            label: "专家市场".into(),
            kind: "api".into(),
            endpoint: EXPERT_CENTER_URL.to_string(),
            ok: count > 0,
            count,
            note,
        },
    )
}

/// 读取本机常用专家（`storage/user-*/personal/global/expert.json` 的 recents）。
fn collect_my_experts(root: &Path) -> Vec<CatalogExpert> {
    let mut out: Vec<CatalogExpert> = Vec::new();
    let storage = root.join("storage");
    let Ok(users) = fs::read_dir(&storage) else {
        return out;
    };
    for user in users.flatten() {
        for rel in [
            "global/expert.json",
            "expert.json",
        ] {
            let path = user.path().join(rel);
            let Some(doc) = read_json(&path) else {
                continue;
            };
            let Some(recents) = doc.get("recents").and_then(|v| v.as_array()) else {
                continue;
            };
            for item in recents {
                let id = item
                    .get("id")
                    .or_else(|| item.get("marketExpertId"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                if id.is_empty() {
                    continue;
                }
                let expert_type = item
                    .get("expertType")
                    .and_then(|v| v.as_str())
                    .unwrap_or("agent")
                    .to_string();
                out.push(CatalogExpert {
                    id,
                    name: item
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string(),
                    name_en: None,
                    profession: item
                        .get("profession")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    description: item
                        .get("defaultInitPrompt")
                        .and_then(|v| v.as_str())
                        .and_then(|s| summarize(s, 160)),
                    category: None,
                    category_id: None,
                    expert_type: Some(expert_type),
                    tags: Vec::new(),
                    avatar: item
                        .get("avatarUrl")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    source: Some("本机常用".into()),
                    member_count: None,
                    use_count: None,
                    updated_at: None,
                });
            }
        }
    }
    out
}

/// 拉取专家团精选场景（公网 API）。
///
/// WorkBuddy 官方专家中心用它做推荐位：场景内含一组专家 id，展示时给出封面与专家名单。
async fn fetch_scenes() -> Vec<FeaturedScene> {
    let Ok(client) = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
    else {
        return Vec::new();
    };
    let Ok(resp) = client.get(EXPERT_SCENES_URL).send().await else {
        return Vec::new();
    };
    if !resp.status().is_success() {
        return Vec::new();
    }
    let Ok(doc) = resp.json::<Value>().await else {
        return Vec::new();
    };
    let Some(scenes) = doc.get("scenes").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    scenes
        .iter()
        .filter_map(|scene| {
            let id = scene.get("id")?.as_str()?.to_string();
            let name = scene
                .get("displayName")
                .and_then(|v| v.get("zh").or_else(|| v.get("en")))
                .and_then(|v| v.as_str())
                .unwrap_or(&id)
                .to_string();
            let description = scene
                .get("description")
                .and_then(|v| v.get("zh").or_else(|| v.get("en")))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let image = scene
                .get("image")
                .and_then(|v| v.as_str())
                .map(|s| format!("{EXPERT_MARKETPLACE_BASE}{s}"));
            let expert_ids = scene
                .get("expertIds")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            Some(FeaturedScene {
                id,
                name,
                description,
                image,
                expert_ids,
            })
        })
        .collect()
}

/// 获取 WorkBuddy 能力目录（专家 / 技能 / 连接器）。
#[tauri::command]
pub async fn workbuddy_catalog() -> WorkBuddyCatalog {
    let root = data_dir().unwrap_or_else(|| PathBuf::from("."));
    let (market_skills, skill_categories) = collect_skill_market(&root);
    let (skills, skill_source) = collect_skills(&root, &market_skills);
    let (connectors, connector_source) = collect_connectors(&root);
    let (teams, team_source) = collect_teams(&root);
    let (experts, expert_categories, expert_source) = fetch_experts().await;
    let my_experts = collect_my_experts(&root);
    let featured_scenes = fetch_scenes().await;

    let mut sources = vec![skill_source, connector_source, team_source, expert_source];
    if !featured_scenes.is_empty() {
        sources.push(CatalogSource {
            id: "scenes".into(),
            label: "专家团精选场景".into(),
            kind: "api".into(),
            endpoint: EXPERT_SCENES_URL.to_string(),
            ok: true,
            count: featured_scenes.len(),
            note: None,
        });
    }

    WorkBuddyCatalog {
        skills,
        connectors,
        experts,
        teams,
        sources,
        expert_categories,
        skill_categories,
        my_experts,
        featured_scenes,
    }
}

