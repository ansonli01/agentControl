<script setup lang="ts">
import { computed } from "vue";
import { Download, ExternalLink, RefreshCw, Trash2 } from "lucide-vue-next";

import StatusTag from "./StatusTag.vue";
import type { AgentDefinition, AgentStatus } from "../types/agent";

const props = defineProps<{
  agent: AgentDefinition;
  status: AgentStatus | null;
  /** 是否为当前选中（详情右栏展示中） */
  selected?: boolean;
}>();

const emit = defineEmits<{
  (e: "select"): void;
  (e: "install", action: string): void;
}>();

const tagStatus = computed(() => {
  if (!props.status || !props.status.installed) return "missing" as const;
  if (props.status.versionUnknown) return "unknown" as const;
  return "installed" as const;
});

const methodLabel: Record<string, string> = {
  npm: "npm 全局包",
  official: "官方安装包",
  manual: "仅检测",
};
</script>

<template>
  <article
    class="panel panel-hover cursor-pointer p-4"
    :class="selected ? 'ring-2 ring-brand/45' : ''"
    @click="emit('select')"
  >
    <div class="flex items-start gap-3">
      <div
        class="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl text-sm font-bold text-white"
        :style="{ background: agent.accent }"
      >
        {{ agent.iconLetter }}
      </div>
      <div class="min-w-0 flex-1">
        <div class="flex items-center gap-2">
          <h3 class="truncate text-sm font-semibold text-ink">{{ agent.name }}</h3>
          <StatusTag :status="tagStatus" />
        </div>
        <p class="mt-0.5 text-xs text-ink-mute">
          {{ agent.vendor }} · {{ agent.kind === "cli" ? "命令行 CLI" : "桌面客户端" }} ·
          {{ methodLabel[agent.installMethod] }}
        </p>
        <p class="mt-2 line-clamp-2 text-xs leading-5 text-ink-soft">
          {{ agent.description }}
        </p>
      </div>
    </div>

    <dl class="mt-3 space-y-1 border-t border-base-600/60 pt-3 text-[11px] text-ink-mute">
      <div class="flex justify-between gap-2">
        <dt>版本</dt>
        <dd class="truncate text-ink-soft">
          {{ status?.version ?? "—" }}
        </dd>
      </div>
      <div class="flex justify-between gap-2">
        <dt>路径</dt>
        <dd class="truncate text-ink-soft">{{ status?.path ?? "—" }}</dd>
      </div>
    </dl>

    <div class="mt-3 flex items-center gap-2" @click.stop>
      <button
        v-if="agent.installMethod !== 'manual'"
        class="btn-primary"
        @click="emit('install', 'install')"
      >
        <Download :size="14" />
        安装
      </button>
      <button
        v-if="agent.installMethod === 'npm' && status?.installed"
        class="btn-ghost"
        @click="emit('install', 'upgrade')"
      >
        <RefreshCw :size="14" />
        升级
      </button>
      <button
        v-if="agent.installMethod === 'npm' && status?.installed"
        class="btn-danger"
        @click="emit('install', 'uninstall')"
      >
        <Trash2 :size="14" />
        卸载
      </button>
      <a
        v-if="agent.officialPage"
        class="btn-ghost ml-auto"
        :href="agent.officialPage"
        target="_blank"
        rel="noreferrer"
      >
        <ExternalLink :size="14" />
        官网
      </a>
    </div>
  </article>
</template>