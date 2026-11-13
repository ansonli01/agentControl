<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import {
  ChevronRight,
  Plug,
  RefreshCw,
  Search,
  Sparkles,
  Users,
  Zap,
} from "lucide-vue-next";

import { api } from "../api/ipc";
import type { CatalogCategory, WorkBuddyCatalog } from "../types/workbuddy";

/**
 * WorkBuddy 能力目录：专家 / 技能 / 连接器。
 *
 * 专家来自官方专家中心公网 API（含头像、分类、标签），
 * 技能复用 WorkBuddy 自己的市场缓存（含图标与使用次数），
 * 连接器读本机连接器市场目录。
 */

const tabs = [
  { id: "experts", label: "专家", icon: Users },
  { id: "skills", label: "技能", icon: Sparkles },
  { id: "connectors", label: "连接器", icon: Plug },
] as const;

type TabId = (typeof tabs)[number]["id"];

const catalog = ref<WorkBuddyCatalog | null>(null);
const loading = ref(false);
const failed = ref(false);
const keyword = ref("");
const tab = ref<TabId>("experts");
const expertCategory = ref("all");
const skillCategory = ref("all");
/**
 * 专家排序，对齐 WorkBuddy 官方专家中心的三档：
 * `reco` 综合（后端 reco_rank 推荐序）、`popular` 最热（use_count）、`newest` 最新（published_at）。
 * 公开清单不带热度，`popular` 无数据时回退到 `reco`（与官方 sanitizeMarketSortBy 同样的兜底）。
 */
const expertSort = ref<"reco" | "popular" | "newest">("reco");
/**
 * 技能排序，对齐 WorkBuddy 内置市场的四档：
 * `score` 综合（市场推荐序）、`use_count` 使用量、`install_count` 安装量、`updated_at` 最新。
 */
const skillSort = ref<"score" | "use_count" | "install_count" | "updated_at">("score");
/** 选中的精选场景（推荐位）。 */
const featuredScene = ref("");
const showMine = ref(false);
/** 技能推荐条的换一批偏移。 */
const recommendOffset = ref(0);
/** 技能推荐条当日关闭次数（连续关闭 3 次则今日不再推荐，与 WorkBuddy 一致）。 */
const recommendDismissCount = ref(0);

const RECOMMEND_DISMISS_LIMIT = 3;
const RECOMMEND_KEY = "agentcontrol.skill-recommend";

function todayKey(): string {
  const d = new Date();
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

function readRecommendDismiss(): number {
  try {
    const raw = localStorage.getItem(`${RECOMMEND_KEY}.${todayKey()}`);
    return raw ? Number(raw) : 0;
  } catch {
    return 0;
  }
}

function recordRecommendDismiss(): void {
  const next = Math.min(RECOMMEND_DISMISS_LIMIT, recommendDismissCount.value + 1);
  recommendDismissCount.value = next;
  try {
    localStorage.setItem(`${RECOMMEND_KEY}.${todayKey()}`, String(next));
  } catch {
    /* 忽略存储失败 */
  }
}

async function load() {
  loading.value = true;
  failed.value = false;
  recommendDismissCount.value = readRecommendDismiss();
  try {
    catalog.value = await api.workbuddyCatalog();
  } catch {
    failed.value = true;
    catalog.value = null;
  } finally {
    loading.value = false;
  }
}

function has(text: string | null | undefined, kw: string): boolean {
  return (text ?? "").toLowerCase().includes(kw);
}

const expertCategories = computed<CatalogCategory[]>(
  () => catalog.value?.expertCategories ?? [],
);
const skillCategories = computed<CatalogCategory[]>(
  () => catalog.value?.skillCategories ?? [],
);

const filteredExperts = computed(() => {
  const kw = keyword.value.trim().toLowerCase();
  const base = showMine.value
    ? catalog.value?.myExperts ?? []
    : catalog.value?.experts ?? [];
  let out = base.filter((e) => {
    if (!kw) return true;
    return (
      has(e.name, kw) ||
      has(e.profession, kw) ||
      has(e.description, kw) ||
      has(e.category, kw) ||
      e.tags.some((t) => has(t, kw))
    );
  });
  if (!showMine.value && expertCategory.value !== "all") {
    out = out.filter((e) => e.categoryId === expertCategory.value);
  }
  if (!showMine.value && featuredScene.value) {
    const scene = catalog.value?.featuredScenes.find((s) => s.id === featuredScene.value);
    const ids = new Set(scene?.expertIds ?? []);
    out = out.filter((e) => ids.has(e.id));
  }
  if (expertSort.value === "popular") {
    // 官方「最热」= use_count 降序；公开清单没有热度时回退综合（保持官方推荐序）。
    const withHeat = out.some((e) => (e.useCount ?? 0) > 0);
    if (withHeat) {
      out = [...out].sort((a, b) => (b.useCount ?? 0) - (a.useCount ?? 0));
    }
  } else if (expertSort.value === "newest") {
    out = [...out].sort((a, b) =>
      (b.updatedAt ?? "").localeCompare(a.updatedAt ?? ""),
    );
  }
  return out;
});

const filteredSkills = computed(() => {
  const kw = keyword.value.trim().toLowerCase();
  let out = (catalog.value?.skills ?? []).filter((s) => {
    if (!kw) return true;
    return (
      has(s.name, kw) ||
      has(s.description, kw) ||
      has(s.namespace, kw) ||
      s.categories.some((c) => has(c, kw))
    );
  });
  if (skillCategory.value !== "all") {
    out = out.filter((s) => s.categories.includes(skillCategory.value));
  }
  if (skillSort.value === "use_count") {
    out = [...out].sort((a, b) => (b.useCount ?? 0) - (a.useCount ?? 0));
  } else if (skillSort.value === "install_count") {
    out = [...out].sort((a, b) => (b.installCount ?? 0) - (a.installCount ?? 0));
  } else if (skillSort.value === "updated_at") {
    out = [...out].sort((a, b) =>
      (b.updatedAt ?? "").localeCompare(a.updatedAt ?? ""),
    );
  }
  return out;
});

/**
 * 技能「为你推荐」：对齐 WorkBuddy 的推荐池逻辑
 * （market.list(sortBy=use_count, sortOrder=desc) 后按热度排序，取前若干条展示）。
 */
const recommendedSkills = computed(() => {
  const pool = (catalog.value?.skills ?? [])
    .filter((s) => (s.useCount ?? 0) > 0)
    .sort((a, b) => (b.useCount ?? 0) - (a.useCount ?? 0));
  if (!pool.length) return [];
  const size = 6;
  const start = recommendOffset.value % pool.length;
  return Array.from({ length: Math.min(size, pool.length) }, (_, i) => pool[(start + i) % pool.length]);
});

/** 推荐条是否今日已关闭（连续关闭 3 次后当天不再推荐）。 */
const recommendHidden = computed(() => recommendDismissCount.value >= RECOMMEND_DISMISS_LIMIT);

function refreshRecommendations() {
  const total = (catalog.value?.skills ?? []).filter((s) => (s.useCount ?? 0) > 0).length;
  if (!total) return;
  recommendOffset.value = (recommendOffset.value + 6) % total;
}

function dismissRecommendations() {
  recordRecommendDismiss();
}

const filteredConnectors = computed(() => {
  const kw = keyword.value.trim().toLowerCase();
  return (catalog.value?.connectors ?? []).filter(
    (c) =>
      !kw ||
      has(c.id, kw) ||
      has(c.name, kw) ||
      has(c.kind, kw) ||
      has(c.url, kw),
  );
});

/** 大数字转「w」显示（与 WorkBuddy 的 formatExpertUseCount 一致：两位小数 + 千分位）。 */
function formatCount(n: number | null | undefined): string {
  if (!n) return "0";
  if (n >= 10000) return `${parseFloat((n / 10000).toFixed(2))}w`;
  return n.toLocaleString("zh-CN");
}

/** 分类大卡封面：用该分类首位专家的头像铺底。 */
function coverStyle(cat: CatalogCategory): Record<string, string> {
  if (!cat.cover) return {};
  return {
    backgroundImage: `url(${cat.cover})`,
    backgroundSize: "cover",
    backgroundPosition: "center 20%",
  };
}

function pickExpertCategory(id: string) {
  expertCategory.value = expertCategory.value === id ? "all" : id;
}

onMounted(() => {
  void load();
});
</script>

<template>
  <div class="space-y-3">
    <!-- 顶部：页签 / 搜索 / 我的专家 -->
    <div class="panel p-3">
      <div class="flex items-center gap-3">
        <nav class="flex items-center gap-1">
          <button
            v-for="t in tabs"
            :key="t.id"
            class="flex items-center gap-1.5 rounded-pill px-3 py-1.5 text-[13px] transition-colors duration-150"
            :class="
              tab === t.id
                ? 'bg-brand/15 font-medium text-brand'
                : 'text-ink-soft hover:bg-base-700 hover:text-ink'
            "
            @click="tab = t.id"
          >
            <component :is="t.icon" :size="14" />
            {{ t.label }}
          </button>
        </nav>

        <div class="relative ml-auto w-56">
          <Search
            :size="13"
            class="pointer-events-none absolute left-2 top-1/2 -translate-y-1/2 text-ink-mute"
          />
          <input
            v-model="keyword"
            class="field h-8 pl-7 text-[12px]"
            placeholder="搜索专家 / 技能 / 连接器"
          />
        </div>

        <button
          class="btn-ghost h-8 px-2.5 text-[12px]"
          :class="showMine ? 'bg-brand/15 text-brand' : ''"
          @click="showMine = !showMine"
        >
          <Users :size="13" />
          我的专家
        </button>
        <button class="btn-ghost h-8 px-2" :disabled="loading" title="重新获取" @click="load">
          <RefreshCw :size="13" :class="{ 'animate-spin': loading }" />
        </button>
      </div>

      <!-- 数据来源 -->
      <div class="mt-2 flex flex-wrap items-center gap-1.5 border-t border-base-600/60 pt-2">
        <span
          v-for="src in catalog?.sources ?? []"
          :key="src.id"
          class="chip"
          :class="src.ok ? 'bg-ok/15 text-ok' : 'bg-base-600/60 text-ink-mute'"
          :title="src.endpoint"
        >
          {{ src.label }} {{ src.count }}
          <span class="ml-1 opacity-70">{{ src.kind === "api" ? "API" : "本机" }}</span>
        </span>
        <span
          v-for="src in (catalog?.sources ?? []).filter((s) => s.note)"
          :key="`${src.id}-note`"
          class="text-[10px] text-warn"
        >
          {{ src.label }}：{{ src.note }}
        </span>
      </div>
    </div>

    <div v-if="loading" class="text-[11px] text-ink-mute">读取 WorkBuddy 数据…</div>
    <div v-else-if="failed" class="text-[11px] text-err">读取能力目录失败</div>

    <template v-else-if="catalog">
      <!-- 专家 -->
      <template v-if="tab === 'experts'">
        <!-- 一级分类大卡 -->
        <div v-if="!showMine" class="flex gap-3 overflow-x-auto pb-1">
          <button
            v-for="cat in expertCategories"
            :key="cat.id"
            class="w-72 shrink-0 overflow-hidden rounded-xl border text-left transition-shadow duration-150 hover:shadow-md"
            :class="
              expertCategory === cat.id
                ? 'border-brand'
                : 'border-base-600/60'
            "
            @click="pickExpertCategory(cat.id)"
          >
            <div
              class="relative flex h-24 items-end bg-gradient-to-br from-brand/30 via-brand/10 to-base-600 p-2"
              :style="coverStyle(cat)"
            >
              <span
                class="relative rounded-md bg-white/85 px-1.5 py-0.5 text-[12px] font-semibold text-ink"
              >
                {{ cat.name }}
              </span>
            </div>
            <div class="space-y-1 bg-base-600/40 p-2">
              <p
                v-for="sample in cat.samples"
                :key="sample"
                class="flex items-center gap-0.5 text-xs text-ink-soft"
              >
                <span class="truncate">{{ sample }}</span>
                <ChevronRight :size="10" class="shrink-0 text-ink-mute" />
              </p>
              <p v-if="!cat.samples.length" class="text-[11px] text-ink-mute">
                {{ cat.count }} 位专家
              </p>
            </div>
          </button>
        </div>

        <!-- 分类筛选 + 排序 -->
        <div class="flex items-center gap-2">
          <div class="flex min-w-0 flex-1 gap-1.5 overflow-x-auto">
            <button
              class="chip shrink-0"
              :class="expertCategory === 'all' ? 'bg-brand/15 text-brand' : 'bg-base-600/60 text-ink-mute'"
              @click="expertCategory = 'all'"
            >
              全部
            </button>
            <button
              v-for="cat in expertCategories"
              :key="cat.id"
              class="chip shrink-0"
              :class="
                expertCategory === cat.id
                  ? 'bg-brand/15 text-brand'
                  : 'bg-base-600/60 text-ink-mute'
              "
              @click="expertCategory = cat.id"
            >
              {{ cat.name }}
            </button>
          </div>
          <div class="flex shrink-0 items-center gap-1">
            <button
              class="chip"
              :class="expertSort === 'reco' ? 'bg-brand/15 text-brand' : 'bg-base-600/60 text-ink-mute'"
              @click="expertSort = 'reco'"
            >
              综合
            </button>
            <button
              class="chip"
              :class="expertSort === 'popular' ? 'bg-brand/15 text-brand' : 'bg-base-600/60 text-ink-mute'"
              title="按使用次数排序（热度需登录态接口，公开清单未带时同综合）"
              @click="expertSort = 'popular'"
            >
              最热
            </button>
            <button
              class="chip"
              :class="expertSort === 'newest' ? 'bg-brand/15 text-brand' : 'bg-base-600/60 text-ink-mute'"
              @click="expertSort = 'newest'"
            >
              最新
            </button>
          </div>
        </div>

        <!-- 精选场景（WorkBuddy 官方推荐位） -->
        <div
          v-if="!showMine && (catalog?.featuredScenes ?? []).length"
          class="flex items-center gap-1.5 overflow-x-auto"
        >
          <span class="shrink-0 text-[11px] text-ink-mute">精选场景</span>
          <button
            class="chip shrink-0"
            :class="featuredScene === '' ? 'bg-brand/15 text-brand' : 'bg-base-600/60 text-ink-mute'"
            @click="featuredScene = ''"
          >
            全部
          </button>
          <button
            v-for="scene in catalog?.featuredScenes ?? []"
            :key="scene.id"
            class="chip shrink-0"
            :class="featuredScene === scene.id ? 'bg-brand/15 text-brand' : 'bg-base-600/60 text-ink-mute'"
            :title="scene.description ?? scene.name"
            @click="featuredScene = featuredScene === scene.id ? '' : scene.id"
          >
            {{ scene.name }}
          </button>
        </div>

        <!-- 专家网格 -->
    <div class="grid grid-cols-3 gap-4">
      <div v-for="e in filteredExperts" :key="e.id" class="panel p-4">
            <div class="flex items-start gap-2">
              <img
                v-if="e.avatar"
                :src="e.avatar"
                :alt="e.name"
                class="h-9 w-9 shrink-0 rounded-full bg-base-600 object-cover"
              />
              <div
                v-else
                class="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-base-600 text-[12px] text-ink-mute"
              >
                {{ e.name.slice(0, 1) }}
              </div>
              <div class="min-w-0 flex-1">
                <p class="truncate text-[12px] font-medium text-ink">{{ e.name }}</p>
                <p class="truncate text-[11px] text-ink-mute">
                  {{ e.profession || e.category || "专家" }}
                </p>
              </div>
            </div>
            <p class="mt-1.5 line-clamp-2 text-[11px] leading-4 text-ink-soft">
              {{ e.description }}
            </p>
            <div class="mt-1.5 flex flex-wrap gap-1">
              <span
                v-for="tag in e.tags.slice(0, 3)"
                :key="tag"
                class="chip bg-base-600/60 text-ink-mute"
              >
                {{ tag }}
              </span>
            </div>
            <div class="mt-1.5 flex items-center gap-1 text-[10px] text-ink-mute">
              <Zap :size="11" />
              <span v-if="e.memberCount">{{ e.memberCount }} 人团队</span>
              <span v-else>{{ e.source }}</span>
            </div>
          </div>
          <p v-if="!filteredExperts.length" class="col-span-4 text-[11px] text-ink-mute">
            没有匹配的专家
          </p>
        </div>

        <!-- 本机专家团 -->
        <section v-if="!showMine" class="panel p-3">
          <div class="flex items-center gap-1.5">
            <Users :size="13" class="text-ink-mute" />
            <span class="text-h2 text-ink">本机专家团</span>
            <span class="chip bg-base-600/60 text-ink-mute">{{ catalog.teams.length }}</span>
          </div>
          <div class="mt-2.5 space-y-2">
            <div
              v-for="team in catalog.teams"
              :key="team.id"
              class="rounded-lg bg-base-700/60 px-2.5 py-2"
            >
              <div class="flex items-center gap-2">
                <p class="min-w-0 flex-1 truncate text-[12px] font-medium text-ink">
                  {{ team.name }}
                </p>
                <span class="chip shrink-0 bg-base-600/60 text-ink-mute">
                  {{ team.memberCount }} 名成员
                </span>
              </div>
              <p
                v-if="team.description"
                class="mt-1 line-clamp-2 text-[11px] leading-4 text-ink-mute"
              >
                {{ team.description }}
              </p>
              <div class="mt-1.5 flex flex-wrap gap-1">
                <span
                  v-for="member in team.members"
                  :key="member.agentId ?? member.name"
                  class="chip bg-base-600/60 text-ink-soft"
                  :title="member.role ?? member.agentType ?? ''"
                >
                  {{ member.name }}
                </span>
              </div>
            </div>
            <p v-if="!catalog.teams.length" class="text-[11px] text-ink-mute">
              本机还没有专家团
            </p>
          </div>
        </section>
      </template>
      <!-- 技能 -->
      <template v-else-if="tab === 'skills'">
        <div class="flex items-center gap-2">
          <div class="flex min-w-0 flex-1 gap-1.5 overflow-x-auto">
            <button
              class="chip shrink-0"
              :class="skillCategory === 'all' ? 'bg-brand/15 text-brand' : 'bg-base-600/60 text-ink-mute'"
              @click="skillCategory = 'all'"
            >
              全部 {{ catalog.skills.length }}
            </button>
            <button
              v-for="cat in skillCategories"
              :key="cat.id"
              class="chip shrink-0"
              :class="
                skillCategory === cat.id
                  ? 'bg-brand/15 text-brand'
                  : 'bg-base-600/60 text-ink-mute'
              "
              @click="skillCategory = cat.id"
            >
              {{ cat.name }} {{ cat.count }}
            </button>
          </div>
          <div class="flex shrink-0 items-center gap-1">
            <button
              class="chip"
              :class="skillSort === 'score' ? 'bg-brand/15 text-brand' : 'bg-base-600/60 text-ink-mute'"
              @click="skillSort = 'score'"
            >
              综合
            </button>
            <button
              class="chip"
              :class="skillSort === 'use_count' ? 'bg-brand/15 text-brand' : 'bg-base-600/60 text-ink-mute'"
              title="按使用次数排序"
              @click="skillSort = 'use_count'"
            >
              使用量
            </button>
            <button
              class="chip"
              :class="skillSort === 'install_count' ? 'bg-brand/15 text-brand' : 'bg-base-600/60 text-ink-mute'"
              title="按安装次数排序"
              @click="skillSort = 'install_count'"
            >
              安装量
            </button>
            <button
              class="chip"
              :class="skillSort === 'updated_at' ? 'bg-brand/15 text-brand' : 'bg-base-600/60 text-ink-mute'"
              @click="skillSort = 'updated_at'"
            >
              最新
            </button>
          </div>
        </div>

        <!-- 为你推荐（对齐 WorkBuddy SkillRecommendBar：换一批 + 连续关闭 3 次今日不再推荐） -->
        <section
          v-if="!recommendHidden && recommendedSkills.length"
          class="panel p-3"
        >
          <div class="flex items-center gap-2">
            <h3 class="flex items-center gap-1 text-[13px] font-medium text-ink">
              <Sparkles :size="13" class="text-brand" />
              为你推荐
            </h3>
            <span class="text-[11px] text-ink-mute">按使用次数推荐</span>
            <div class="ml-auto flex items-center gap-1">
              <button class="btn-ghost h-7 px-2 text-[11px]" @click="refreshRecommendations">
                <RefreshCw :size="12" />
                换一批
              </button>
              <button
                class="btn-ghost h-7 px-2 text-[11px]"
                :title="`连续关闭 ${RECOMMEND_DISMISS_LIMIT} 次，今日将不再推荐`"
                @click="dismissRecommendations"
              >
                关闭
              </button>
            </div>
          </div>
          <div class="mt-2 grid grid-cols-6 gap-2">
            <div
              v-for="s in recommendedSkills"
              :key="s.id"
              class="flex items-center gap-2 rounded-lg bg-base-600/40 p-2"
            >
              <img
                v-if="s.icon"
                :src="s.icon"
                :alt="s.name"
                class="h-7 w-7 shrink-0 rounded-md bg-base-600 object-cover"
              />
              <div
                v-else
                class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md bg-base-600 text-[11px] text-ink-mute"
              >
                {{ s.name.slice(0, 1) }}
              </div>
              <div class="min-w-0">
                <p class="truncate text-[12px] text-ink">{{ s.name }}</p>
                <p class="text-[10px] text-ink-mute">{{ formatCount(s.useCount) }} 次使用</p>
              </div>
            </div>
          </div>
        </section>

    <div class="grid grid-cols-3 gap-4">
      <div v-for="s in filteredSkills" :key="s.id" class="panel p-4">
            <div class="flex items-start gap-2">
              <img
                v-if="s.icon"
                :src="s.icon"
                :alt="s.name"
                class="h-9 w-9 shrink-0 rounded-lg bg-base-600 object-cover"
              />
              <div
                v-else
                class="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-base-600 text-[12px] text-ink-mute"
              >
                {{ s.name.slice(0, 1) }}
              </div>
              <div class="min-w-0 flex-1">
                <p class="truncate text-[12px] font-medium text-ink">{{ s.name }}</p>
                <p class="truncate text-[10px] text-ink-mute">
                  {{ s.namespace || s.slug || s.marketplace || "技能" }}
                </p>
              </div>
              <span v-if="s.installed" class="chip shrink-0 bg-ok/15 text-ok">已安装</span>
            </div>
            <p class="mt-1.5 line-clamp-2 text-[11px] leading-4 text-ink-soft">
              {{ s.description }}
            </p>
            <div class="mt-1.5 flex items-center gap-2 text-[10px] text-ink-mute">
              <span class="flex items-center gap-0.5">
                <Zap :size="10" />
                {{ formatCount(s.useCount) }} 次使用
              </span>
              <span v-if="s.installCount">
                {{ formatCount(s.installCount) }} 次安装
              </span>
            </div>
          </div>
          <p v-if="!filteredSkills.length" class="col-span-4 text-[11px] text-ink-mute">
            没有匹配的技能
          </p>
        </div>
      </template>
      <!-- 连接器 -->
      <template v-else>
    <div class="grid grid-cols-3 gap-4">
          <div
            v-for="conn in filteredConnectors"
            :key="`${conn.id}-${conn.name}`"
      class="panel p-4"
          >
            <div class="flex items-center gap-2">
              <Plug :size="13" class="shrink-0 text-ink-mute" />
              <p class="min-w-0 flex-1 truncate text-[12px] font-medium text-ink">
                {{ conn.name }}
              </p>
              <span v-if="conn.enabled" class="chip shrink-0 bg-ok/15 text-ok">已接入</span>
            </div>
            <p class="mt-1 truncate text-[10px] text-ink-mute">
              {{ conn.kind ?? "mcp" }} · {{ conn.id }}
            </p>
            <p
              v-if="conn.url"
              class="mt-0.5 truncate text-[10px] text-ink-mute"
              :title="conn.url"
            >
              {{ conn.url }}
            </p>
            <p v-else-if="conn.command" class="mt-0.5 truncate text-[10px] text-ink-mute">
              {{ conn.command }}
            </p>
          </div>
          <p v-if="!filteredConnectors.length" class="col-span-3 text-[11px] text-ink-mute">
            没有匹配的连接器
          </p>
        </div>
      </template>
    </template>
  </div>
</template>
