<script setup lang="ts">
import { computed, ref } from "vue";
import { Search } from "lucide-vue-next";

import AgentCard from "../components/AgentCard.vue";
import EmptyState from "../components/EmptyState.vue";
import { useAgentsStore } from "../stores/agents";

const agents = useAgentsStore();
const keyword = ref("");
const filter = ref<"all" | "installed" | "missing" | "running">("all");

const filtered = computed(() => {
  const text = keyword.value.trim().toLowerCase();
  return agents.sortedAgents.filter((row) => {
    const st = row.status;
    if (filter.value === "installed" && !st?.installed) return false;
    if (filter.value === "missing" && st?.installed) return false;
    if (filter.value === "running" && !agents.latestTaskFor(row.agent.id)) return false;
    if (!text) return true;
    return (
      row.agent.name.toLowerCase().includes(text) ||
      row.agent.vendor.toLowerCase().includes(text) ||
      row.agent.tags.some((t) => t.toLowerCase().includes(text))
    );
  });
});

const tabs = [
  { key: "all", label: "全部" },
  { key: "installed", label: "已安装" },
  { key: "missing", label: "未安装" },
  { key: "running", label: "有任务" },
] as const;

async function onInstall(agentId: string, action: string) {
  agents.selectedId = agentId;
  await agents.install(agentId, action);
}
</script>
<template>
  <div class="space-y-4">
    <header class="mb-1">
      <h1 class="text-h1 font-semibold text-ink">助理</h1>
      <p class="mt-1 text-[13px] text-ink-mute">识别本机已安装的智能体 CLI 与桌面客户端，支持一键安装与升级</p>
    </header>


    <div class="panel flex items-center gap-3 p-3">
      <div class="flex items-center gap-1 rounded-lg bg-base-700 p-1">
        <button
          v-for="tab in tabs"
          :key="tab.key"
          class="rounded-md px-3 py-1.5 text-xs transition-colors duration-150"
          :class="
            filter === tab.key
              ? 'bg-brand text-white'
              : 'text-ink-soft hover:bg-base-600 hover:text-ink'
          "
          @click="filter = tab.key"
        >
          {{ tab.label }}
        </button>
      </div>
      <div class="relative ml-auto w-64">
        <Search
          :size="14"
          class="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-ink-mute"
        />
        <input v-model="keyword" class="field !pl-8" placeholder="搜索名称、厂商或标签" />
      </div>
    </div>

    <div v-if="filtered.length" class="grid grid-cols-3 gap-4">
      <AgentCard
        v-for="row in filtered"
        :key="row.agent.id"
        :agent="row.agent"
        :status="row.status"
        :selected="agents.selectedId === row.agent.id"
        @select="agents.selectedId = row.agent.id"
        @install="(action) => onInstall(row.agent.id, action)"
      />
    </div>
    <EmptyState
      v-else
      title="没有匹配的智能体"
      hint="调整筛选条件，或在设置中补充自定义智能体源"
    />
  </div>
</template>
