<script setup lang="ts">
import { ref, watch } from "vue";
import { Copy, RefreshCw, ShieldCheck } from "lucide-vue-next";

import { api } from "../api/ipc";
import type { AgentCredentialReport } from "../types/agent";

const props = defineProps<{
  agentId: string;
  agentName: string;
}>();

const report = ref<AgentCredentialReport | null>(null);
const loading = ref(false);
const failed = ref(false);

async function load() {
  loading.value = true;
  failed.value = false;
  try {
    report.value = await api.agentCredentials(props.agentId);
  } catch {
    failed.value = true;
    report.value = null;
  } finally {
    loading.value = false;
  }
}

async function copy(text: string) {
  try {
    await navigator.clipboard.writeText(text);
  } catch {
    /* 忽略 */
  }
}

function formatSize(size: number): string {
  if (size <= 0) return "—";
  if (size >= 1024 * 1024) return `${(size / 1024 / 1024).toFixed(1)} MB`;
  if (size >= 1024) return `${(size / 1024).toFixed(1)} KB`;
  return `${size} B`;
}

watch(() => props.agentId, load, { immediate: true });
</script>

<template>
  <div class="space-y-2.5">
    <div v-if="loading" class="text-[11px] text-ink-mute">读取中…</div>
    <div v-else-if="failed" class="text-[11px] text-err">读取登录态失败</div>

    <template v-else-if="report">
      <!-- 账号体系 -->
      <div class="panel p-3">
        <div class="flex flex-wrap items-center gap-2">
          <ShieldCheck :size="13" class="text-ink-mute" />
          <span class="text-xs font-medium text-ink">{{ report.accountSystem }}</span>
          <span
            class="chip"
            :class="report.loggedIn ? 'bg-ok/15 text-ok' : 'bg-base-600/60 text-ink-mute'"
          >
            {{ report.loggedIn ? "已登录" : "未检出登录态" }}
          </span>
          <button
            class="btn-ghost ml-auto h-6 px-1.5 text-[11px]"
            :disabled="loading"
            title="刷新"
            @click="load"
          >
            <RefreshCw :size="12" :class="{ 'animate-spin': loading }" />
          </button>
        </div>

        <dl class="mt-2 space-y-1 text-[11px]">
          <div class="flex justify-between gap-2">
            <dt class="text-ink-mute">登录方式</dt>
            <dd class="truncate text-ink-soft">{{ report.method }}</dd>
          </div>
          <div v-if="report.loginCommand" class="flex items-center justify-between gap-2">
            <dt class="shrink-0 text-ink-mute">登录命令</dt>
            <dd class="flex min-w-0 items-center gap-1">
              <code class="truncate text-ink-soft">{{ report.loginCommand }}</code>
              <button
                class="btn-ghost h-4 px-1 text-[10px]"
                title="复制命令"
                @click="copy(report.loginCommand ?? '')"
              >
                <Copy :size="10" />
              </button>
            </dd>
          </div>
        </dl>

        <div v-if="report.sharedWith.length" class="mt-2">
          <p class="text-[10px] text-ink-mute">共用同一份登录态</p>
          <div class="mt-1 flex flex-wrap gap-1">
            <span
              v-for="name in report.sharedWith"
              :key="name"
              class="chip bg-base-600/60 text-ink-soft"
            >
              {{ name }}
            </span>
          </div>
        </div>

        <p v-if="report.note" class="mt-2 text-[11px] leading-4 text-ink-mute">
          {{ report.note }}
        </p>
      </div>

      <!-- 脱敏字段 -->
      <div v-if="report.fields.length" class="panel p-3">
        <p class="text-[11px] font-medium text-ink">登录信息（已脱敏）</p>
        <dl class="mt-1.5 space-y-1 text-[11px]">
          <div
            v-for="(field, i) in report.fields"
            :key="i"
            class="flex items-start justify-between gap-2"
          >
            <dt class="shrink-0 text-ink-mute">{{ field.label }}</dt>
            <dd
              class="min-w-0 flex-1 break-all text-right"
              :class="field.secret ? 'text-ink-soft' : 'text-ink'"
            >
              {{ field.value }}
            </dd>
          </div>
        </dl>
      </div>

      <!-- 凭据来源 -->
      <div v-if="report.sources.length" class="panel p-3">
        <p class="text-[11px] font-medium text-ink">凭据来源</p>
        <div class="mt-1.5 space-y-1.5">
          <div
            v-for="(src, i) in report.sources"
            :key="i"
            class="rounded-lg bg-base-700/60 px-2.5 py-1.5"
          >
            <div class="flex items-center gap-2 text-[10px]">
              <span
                class="chip"
                :class="src.exists ? 'bg-ok/15 text-ok' : 'bg-base-600/60 text-ink-mute'"
              >
                {{ src.exists ? "存在" : "缺失" }}
              </span>
              <span class="text-ink-soft">{{ src.label }}</span>
              <span class="text-ink-mute">{{ src.kind }}</span>
              <span class="ml-auto text-ink-mute">
                {{
                  src.entries > 0
                    ? `${src.entries} 项`
                    : src.exists
                      ? formatSize(src.size)
                      : "—"
                }}
              </span>
            </div>
            <p class="mt-1 break-all text-[10px] leading-4 text-ink-mute">{{ src.path }}</p>
          </div>
        </div>
      </div>

      <p
        v-if="!report.loggedIn && !report.fields.length"
        class="rounded-md bg-warn/10 px-2 py-1.5 text-[11px] leading-4 text-warn"
      >
        该智能体未在本机检出可读的登录态
      </p>
    </template>
  </div>
</template>
