<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import { X } from "lucide-vue-next";

import StatusTag from "./StatusTag.vue";
import AgentCredentialPanel from "./AgentCredentialPanel.vue";
import WorkBuddySessionsPanel from "./WorkBuddySessionsPanel.vue";
import { useAgentsStore } from "../stores/agents";
import type { AgentDefinition, AgentStatus } from "../types/agent";

const props = defineProps<{
  agent: AgentDefinition;
  status: AgentStatus | null;
}>();

const emit = defineEmits<{ (e: "close"): void }>();

const agents = useAgentsStore();
const logBox = ref<HTMLElement | null>(null);

const task = computed(() => agents.latestTaskFor(props.agent.id));

/** 腾讯 CodeBuddy 系：额外展示 WorkBuddy 会话数据 */
const codebuddyFamily = computed(() =>
  ["workbuddy", "codebuddy"].includes(props.agent.id),
);

const phaseLabel: Record<string, string> = {
  queued: "排队中",
  resolving: "解析安装源",
  downloading: "下载中",
  verifying: "校验中",
  installing: "安装中",
  uninstalling: "卸载中",
  finished: "已完成",
  failed: "失败",
  cancelled: "已取消",
};

const progress = computed(() => {
  const p = task.value?.status.progress;
  return typeof p === "number" ? Math.max(0, Math.min(1, p)) : null;
});

watch(
  () => task.value?.logs.length,
  async () => {
    await nextTick();
    if (logBox.value) logBox.value.scrollTop = logBox.value.scrollHeight;
  },
);
</script>
<template>
  <aside
    class="flex h-full w-[420px] shrink-0 flex-col border-l border-base-600 bg-white/95 backdrop-blur"
  >
    <div class="flex h-14 items-center justify-between border-b border-base-600/60 px-4">
      <div class="flex items-center gap-2.5">
        <div
          class="flex h-8 w-8 items-center justify-center rounded-lg text-xs font-bold text-white"
          :style="{ background: agent.accent }"
        >
          {{ agent.iconLetter }}
        </div>
        <div>
          <p class="text-sm font-semibold text-ink">{{ agent.name }}</p>
          <p class="text-[11px] text-ink-mute">{{ agent.vendor }}</p>
        </div>
      </div>
      <button class="btn-ghost !px-2" @click="emit('close')">
        <X :size="16" />
      </button>
    </div>

    <div class="min-h-0 flex-1 space-y-4 overflow-y-auto p-4">
      <section class="panel p-3">
        <div class="flex items-center justify-between">
          <h4 class="text-h2 text-ink">安装状态</h4>
          <StatusTag
            :status="status?.installed ? (status.versionUnknown ? 'unknown' : 'installed') : 'missing'"
          />
        </div>
        <dl class="mt-2 space-y-1.5 text-xs">
          <div class="flex justify-between gap-3">
            <dt class="text-ink-mute">版本</dt>
            <dd class="truncate text-ink-soft">{{ status?.version ?? "—" }}</dd>
          </div>
          <div class="flex justify-between gap-3">
            <dt class="text-ink-mute">路径</dt>
            <dd class="truncate text-ink-soft" :title="status?.path ?? ''">
              {{ status?.path ?? "—" }}
            </dd>
          </div>
          <div class="flex justify-between gap-3">
            <dt class="text-ink-mute">探测来源</dt>
            <dd class="truncate text-ink-soft">
              {{ status?.sources.map((s) => s.kind).join(" / ") || "—" }}
            </dd>
          </div>
          <div v-if="status?.note" class="rounded-md bg-warn/10 px-2 py-1 text-warn">
            {{ status.note }}
          </div>
        </dl>
      </section>

      <section class="panel p-3">
        <h4 class="text-h2 text-ink">安装源</h4>
        <dl class="mt-2 space-y-1.5 text-xs">
          <div class="flex justify-between gap-3">
            <dt class="text-ink-mute">方式</dt>
            <dd class="text-ink-soft">
              {{ agent.installMethod === "npm" ? "npm 全局包" : agent.installMethod === "official" ? "官方安装包" : "仅检测" }}
            </dd>
          </div>
          <div v-if="agent.npmPackage" class="flex justify-between gap-3">
            <dt class="text-ink-mute">包名</dt>
            <dd class="truncate text-ink-soft">{{ agent.npmPackage }}</dd>
          </div>
          <div v-if="agent.officialDownload" class="flex justify-between gap-3">
            <dt class="text-ink-mute">直链</dt>
            <dd class="truncate text-ink-soft" :title="agent.officialDownload">
              {{ agent.officialDownload }}
            </dd>
          </div>
          <div v-if="agent.openPlatform" class="flex justify-between gap-3">
            <dt class="text-ink-mute">开放平台</dt>
            <dd class="truncate text-ink-soft">{{ agent.openPlatform }}</dd>
          </div>
        </dl>
      </section>
      <section v-if="task" class="panel p-3">
        <div class="flex items-center justify-between">
          <h4 class="text-h2 text-ink">{{ task.status.action }}进度</h4>
          <span class="text-[11px] text-ink-mute">
            {{ phaseLabel[task.status.phase] ?? task.status.phase }}
          </span>
        </div>        <div
          v-if="progress !== null"
          class="mt-2 h-1.5 w-full overflow-hidden rounded-pill bg-base-600"
        >
          <div
            class="h-full rounded-pill bg-gradient-to-r from-brand to-magic transition-all duration-300"
            :style="{ width: Math.round(progress * 100) + '%' }"
          ></div>
        </div>
        <p v-if="task.status.message" class="mt-2 text-[11px] leading-5 text-ink-soft">
          {{ task.status.message }}
        </p>
        <div
          ref="logBox"
          class="wb-terminal mt-2 max-h-64 overflow-y-auto"
        >
          <p
            v-for="(line, idx) in task.logs"
            :key="idx"
            :class="line.stream === 'stderr' ? 'text-err' : 'text-[#c9cdd4]'"
          >
            {{ line.text }}
          </p>
          <p v-if="task.logs.length === 0" class="text-[#8a9099]">暂无日志输出</p>
        </div>
      </section>

      <!-- 登录态：按选中智能体切换各自的账号体系与凭据来源 -->
      <section class="space-y-2">
        <div class="flex items-center gap-2">
          <h4 class="text-h2 text-ink">登录态</h4>
          <span class="text-[10px] text-ink-mute">{{ agent.vendor }}</span>
        </div>
        <AgentCredentialPanel :agent-id="agent.id" :agent-name="agent.name" />
        <WorkBuddySessionsPanel v-if="codebuddyFamily" />
      </section>
    </div>

    <div class="border-t border-base-600/60 p-3">
      <button
        class="btn-ghost w-full justify-center"
        @click="emit('close')"
      >
        收起详情
      </button>
    </div>
  </aside>
</template>