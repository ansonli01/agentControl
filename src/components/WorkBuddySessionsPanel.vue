<script setup lang="ts">
import { onMounted, ref } from "vue";
import { ChevronDown, ChevronRight, RefreshCw } from "lucide-vue-next";

import { api } from "../api/ipc";
import type { WorkBuddySessionOverview } from "../types/workbuddy";

/**
 * WorkBuddy 会话数据面板。
 * 仅在腾讯 CodeBuddy 系（WorkBuddy / CodeBuddy）智能体的详情右栏里展示。
 */

const sessions = ref<WorkBuddySessionOverview | null>(null);
const sessionsLoading = ref(false);
const sessionsOpen = ref(true);

async function loadSessions() {
  sessionsLoading.value = true;
  try {
    sessions.value = await api.workbuddySessions(30);
  } catch {
    sessions.value = null;
  } finally {
    sessionsLoading.value = false;
  }
}

function toggleSessions() {
  sessionsOpen.value = !sessionsOpen.value;
  if (sessionsOpen.value && !sessions.value) void loadSessions();
}

function formatTime(ts: number | null): string {
  if (!ts) return "—";
  return new Date(ts).toLocaleString("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function formatNumber(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(2)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return String(n);
}

const sessionStatusText: Record<string, string> = {
  completed: "已完成",
  running: "进行中",
  active: "进行中",
  failed: "失败",
  cancelled: "已取消",
  idle: "空闲",
};

onMounted(() => {
  void loadSessions();
});
</script>

<template>
  <section class="panel overflow-hidden p-3">
    <div class="flex items-center gap-2">
      <button
        class="flex items-center gap-1 text-xs font-medium text-ink"
        @click="toggleSessions"
      >
        <ChevronDown v-if="sessionsOpen" :size="13" class="text-ink-mute" />
        <ChevronRight v-else :size="13" class="text-ink-mute" />
        WorkBuddy 会话数据
      </button>
      <span v-if="sessions" class="chip bg-base-600/60 text-ink-mute">
        {{ sessions.totalSessions }} 个会话
      </span>
      <button
        class="btn-ghost ml-auto h-6 px-1.5 text-[11px]"
        :disabled="sessionsLoading"
        title="刷新"
        @click="loadSessions"
      >
        <RefreshCw :size="12" :class="{ 'animate-spin': sessionsLoading }" />
      </button>
    </div>

    <div v-if="sessionsOpen" class="mt-2.5">
      <div v-if="sessionsLoading" class="text-[11px] text-ink-mute">读取中…</div>
      <template v-else-if="sessions">
        <div class="grid grid-cols-2 gap-2">
          <div class="min-w-0 rounded-lg bg-base-700/60 px-2 py-1.5">
            <p class="text-[10px] text-ink-mute">会话总数</p>
            <p class="mt-0.5 text-xs font-semibold text-ink">
              {{ sessions.totalSessions }}
            </p>
          </div>
          <div class="min-w-0 rounded-lg bg-base-700/60 px-2 py-1.5">
            <p class="text-[10px] text-ink-mute">未结束</p>
            <p class="mt-0.5 text-xs font-semibold text-ink">
              {{ sessions.activeSessions }}
            </p>
          </div>
          <div class="min-w-0 rounded-lg bg-base-700/60 px-2 py-1.5">
            <p class="text-[10px] text-ink-mute">累计 token</p>
            <p class="mt-0.5 text-xs font-semibold text-ink">
              {{ formatNumber(sessions.totalUsed) }}
            </p>
          </div>
          <div class="min-w-0 rounded-lg bg-base-700/60 px-2 py-1.5">
            <p class="text-[10px] text-ink-mute">累计额度</p>
            <p class="mt-0.5 text-xs font-semibold text-ink">
              {{ sessions.totalCredit.toFixed(2) }}
            </p>
          </div>
        </div>

        <div class="mt-2.5 max-h-56 space-y-1 overflow-y-auto">
          <div
            v-for="row in sessions.sessions"
            :key="row.id"
            class="rounded-lg bg-base-700/60 px-2.5 py-1.5 text-[11px]"
          >
            <div class="flex items-start gap-2">
              <p class="min-w-0 flex-1 truncate text-ink" :title="row.title ?? ''">
                {{ row.title?.split("\n")[0] || "（无标题）" }}
              </p>
              <span class="shrink-0 text-[10px] text-ink-mute">
                {{ row.status ? (sessionStatusText[row.status] ?? row.status) : "—" }}
              </span>
            </div>
            <div class="mt-0.5 flex items-center gap-2 text-[10px] text-ink-mute">
              <span class="truncate">{{ row.model ?? "—" }}</span>
              <span v-if="row.used" class="shrink-0">
                {{ formatNumber(row.used) }} tok
              </span>
              <span class="ml-auto shrink-0">{{ formatTime(row.updatedAt) }}</span>
            </div>
          </div>
          <p v-if="!sessions.sessions.length" class="text-[11px] text-ink-mute">
            没有会话记录
          </p>
        </div>

        <div v-if="sessions.workspaces.length" class="mt-2.5">
          <p class="text-[11px] text-ink-mute">最近工作区</p>
          <div class="mt-1 flex flex-wrap gap-1">
            <span
              v-for="ws in sessions.workspaces.slice(0, 8)"
              :key="ws.path"
              class="chip bg-base-600/60 text-ink-soft"
              :title="ws.path"
            >
              {{ ws.path.split(/[\\/]/).filter(Boolean).slice(-1)[0] ?? ws.path }}
              <span class="ml-1 text-ink-mute">{{ ws.sessions }}</span>
            </span>
          </div>
        </div>

        <p v-if="sessions.note" class="mt-2 text-[11px] text-warn">
          {{ sessions.note }}
        </p>
      </template>
      <p v-else class="text-[11px] text-ink-mute">未读取到会话数据</p>
    </div>
  </section>
</template>
