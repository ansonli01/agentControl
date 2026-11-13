<template>
  <div class="space-y-4">
    <!-- 平台切换：WorkBuddy / 豆包工作 / 小米智电能，三者内容互不混排 -->
    <div class="flex items-center gap-1 border-b border-base-600/60 pb-2">
      <button
        v-for="item in platformTabs"
        :key="item.id"
        class="rounded-pill px-3 py-1 text-xs transition-colors duration-150"
        :class="
          platform === item.id
            ? 'bg-brand/15 text-brand'
            : 'text-ink-soft hover:bg-base-700 hover:text-ink'
        "
        @click="platform = item.id"
      >
        {{ item.label }}
      </button>
      <span v-if="loading" class="ml-2 text-[11px] text-ink-soft">加载中…</span>
    </div>

    <!-- WorkBuddy：专家 / 技能 / 连接器 -->
    <WorkBuddyCatalogPanel v-if="platform === 'workbuddy'" />

    <!-- 豆包工作 / 小米智电能：技能 / 插件 -->
    <div v-else class="space-y-3">
      <div class="flex items-center gap-1">
        <button
          v-for="sec in sections"
          :key="sec.id"
          class="rounded-pill px-3 py-1 text-xs transition-colors duration-150"
          :class="
            section === sec.id
              ? 'bg-brand/15 text-brand'
              : 'text-ink-soft hover:bg-base-700 hover:text-ink'
          "
          @click="section = sec.id"
        >
          {{ sec.label }}
          <span class="ml-1 text-[10px] opacity-70">{{ sec.count }}</span>
        </button>
        <span class="ml-auto text-[11px] text-ink-soft">
          {{ currentCatalog?.name || platformLabel }} · 本机已安装
        </span>
      </div>

      <!-- 技能 -->
      <div v-if="section === 'skills'" class="grid grid-cols-3 gap-4">
        <article v-for="skill in filteredSkills" :key="skill.id" class="panel p-4">
          <div class="flex items-start gap-3">
            <div
              class="flex h-11 w-11 shrink-0 items-center justify-center overflow-hidden rounded-lg bg-base-700 text-sm font-semibold text-ink"
            >
              <img
                v-if="isImageIcon(skill.icon)"
                :src="skill.icon ?? ''"
                :alt="skill.name"
                class="h-full w-full object-cover"
              />
              <span v-else>{{ skill.name.slice(0, 1).toUpperCase() }}</span>
            </div>
            <div class="min-w-0 flex-1">
              <div class="flex items-center gap-2">
                <h4 class="truncate text-sm font-medium text-ink">{{ skill.name }}</h4>
                <span
                  v-if="skill.version"
                  class="rounded-pill bg-base-700 px-1.5 py-0.5 text-[10px] text-ink-soft"
                >
                  v{{ skill.version }}
                </span>
              </div>
              <p class="mt-1 line-clamp-2 text-xs leading-relaxed text-ink-soft">
                {{ skill.description || "暂无描述" }}
              </p>
            </div>
          </div>
          <div class="mt-3 flex flex-wrap items-center gap-1">
            <span v-if="skill.category" class="chip">{{ skill.category }}</span>
            <span v-if="skill.origin" class="chip">{{ skill.origin }}</span>
          </div>
        </article>
        <p v-if="!filteredSkills.length" class="text-xs text-ink-soft">
          未在本机发现该平台的技能。
        </p>
      </div>

      <!-- 插件 -->
      <div v-else class="grid grid-cols-3 gap-4">
        <article v-for="plugin in filteredPlugins" :key="plugin.id" class="panel p-4">
          <div class="flex items-center gap-2">
            <div
              class="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-base-700 text-xs font-semibold text-ink"
            >
              {{ plugin.name.slice(0, 2).toUpperCase() }}
            </div>
            <h4 class="truncate text-sm font-medium text-ink">{{ plugin.name }}</h4>
            <span v-if="plugin.kind" class="chip">{{ kindLabel(plugin.kind) }}</span>
          </div>
          <p class="mt-2 line-clamp-2 text-xs leading-relaxed text-ink-soft">
            {{ plugin.description || "暂无描述" }}
          </p>
          <div v-if="plugin.tools.length" class="mt-3 flex flex-wrap gap-1">
            <span v-for="tool in plugin.tools.slice(0, 4)" :key="tool" class="chip">
              {{ tool }}
            </span>
            <span v-if="plugin.tools.length > 4" class="chip">
              +{{ plugin.tools.length - 4 }}
            </span>
          </div>
          <div v-if="plugin.source" class="mt-3 text-[10px] text-ink-soft">
            {{ plugin.source }}
          </div>
        </article>
        <p v-if="!filteredPlugins.length" class="text-xs text-ink-soft">
          未在本机发现该平台的插件。
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import WorkBuddyCatalogPanel from "./WorkBuddyCatalogPanel.vue";
import { api } from "../api/ipc";
import type { PlatformCatalog, PlatformPlugin, PlatformSkill } from "../types/workbuddy";

type PlatformId = "workbuddy" | "doubao-work" | "xiaomi-mimo";
type SectionId = "skills" | "plugins";

const platformTabs: { id: PlatformId; label: string }[] = [
  { id: "workbuddy", label: "WorkBuddy" },
  { id: "doubao-work", label: "豆包工作" },
  { id: "xiaomi-mimo", label: "小米智电能" },
];

const platform = ref<PlatformId>("workbuddy");
const section = ref<SectionId>("skills");
const catalogs = ref<PlatformCatalog[]>([]);
const loading = ref(false);

const currentCatalog = computed(() =>
  catalogs.value.find((c) => c.id === platform.value),
);

const platformLabel = computed(
  () => platformTabs.find((p) => p.id === platform.value)?.label ?? "",
);

const sections = computed(() => {
  const catalog = currentCatalog.value;
  if (!catalog) {
    return [
      { id: "skills" as SectionId, label: "技能", count: 0 },
      { id: "plugins" as SectionId, label: "插件", count: 0 },
    ];
  }
  return [
    { id: "skills" as SectionId, label: "技能", count: catalog.skills.length },
    { id: "plugins" as SectionId, label: "插件", count: catalog.plugins.length },
  ];
});

const filteredSkills = computed<PlatformSkill[]>(
  () => currentCatalog.value?.skills ?? [],
);

const filteredPlugins = computed<PlatformPlugin[]>(
  () => currentCatalog.value?.plugins ?? [],
);

function isImageIcon(icon: string | null): boolean {
  return !!icon && /^(https?:|data:|file:)/i.test(icon);
}

function kindLabel(kind: string): string {
  if (kind === "mcp") return "MCP";
  if (kind === "cli") return "命令行";
  return "工具";
}

onMounted(async () => {
  loading.value = true;
  try {
    catalogs.value = await api.platformCatalogs();
  } catch (err) {
    console.error("读取平台目录失败", err);
  } finally {
    loading.value = false;
  }
});
</script>
