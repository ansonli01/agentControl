<script setup lang="ts">
import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  Bot,
  ChevronRight,
  Clock,
  Database,
  FolderKanban,
  LogOut,
  RefreshCw,
  Settings2,
  Sparkles,
} from "lucide-vue-next";

import { useAgentsStore } from "../stores/agents";
import { useConfigStore } from "../stores/config";

const emit = defineEmits<{ (e: "close"): void }>();

const router = useRouter();
const agents = useAgentsStore();
const config = useConfigStore();

const checking = ref(false);
const checkResult = ref("");

const installed = computed(() => agents.installedCount);
const total = computed(() => agents.totalCount);

interface MenuRow {
  key: string;
  label: string;
  icon: typeof Bot;
  shortcut?: string;
  route?: string;
}

const rows = computed<MenuRow[]>(() => [
  { key: "agents", label: "助理市场", icon: Bot, route: "agents" },
  { key: "projects", label: "工作区与空间", icon: FolderKanban, route: "projects" },
  { key: "credentials", label: "凭据与 MCP", icon: Database, route: "credentials" },
  { key: "tasks", label: "定时任务记录", icon: Clock, route: "tasks" },
  { key: "settings", label: "设置", icon: Settings2, shortcut: "Ctrl+,", route: "settings" },
  { key: "memory", label: "记忆与进化", icon: Sparkles, route: "settings" },
]);

function go(row: MenuRow): void {
  emit("close");
  if (row.route) router.push({ name: row.route });
}

async function checkUpdate(): Promise<void> {
  checking.value = true;
  checkResult.value = "";
  await agents.scan();
  checking.value = false;
  checkResult.value = `已是最新 v0.1.0 · 已安装 ${agents.installedCount}/${agents.totalCount}`;
}

async function quit(): Promise<void> {
  emit("close");
  try {
    await getCurrentWindow().close();
  } catch {
    /* 忽略关闭异常 */
  }
}
</script>

<template>
  <div
    class="w-[248px] overflow-hidden rounded-xl border border-base-600 bg-white shadow-lift"
    @click.stop
  >
    <div class="flex items-center gap-2.5 px-3 py-3">
      <div
        class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-brand text-[12px] font-semibold text-white"
      >
        AC
      </div>
      <div class="min-w-0">
        <p class="truncate text-[13px] font-medium text-ink">AgentControl</p>
        <p class="mt-0.5 truncate text-[11px] text-ink-mute">
          本地工作区 · agentControl
        </p>
      </div>
    </div>

    <div class="border-t border-base-600 px-3 py-2.5">
      <div class="flex items-start justify-between">
        <div>
          <p class="text-[11px] text-ink-mute">套餐</p>
          <p class="mt-0.5 text-[13px] text-ink">本地版</p>
        </div>
        <div class="text-right">
          <p class="text-[11px] text-ink-mute">已装智能体</p>
          <p class="mt-0.5 text-[13px] text-ink">{{ installed }} / {{ total }}</p>
        </div>
      </div>
      <div class="mt-2.5 flex gap-2">
        <button
          class="btn-primary h-7 flex-1 justify-center px-2 text-xs"
          @click="go({ key: 'agents', label: '', icon: Bot })"
        >
          打开助理市场
        </button>
        <button
          class="btn-ghost h-7 flex-1 justify-center px-2 text-xs"
          :disabled="checking"
          @click="checkUpdate"
        >
          <RefreshCw :size="12" :class="checking ? 'animate-spin' : ''" />
          {{ checking ? "检查中…" : "检查更新" }}
        </button>
      </div>
      <p v-if="checkResult" class="mt-2 text-[11px] text-ink-mute">{{ checkResult }}</p>
    </div>

    <div class="border-t border-base-600 py-1">
      <button
        v-for="row in rows"
        :key="row.key"
        class="flex w-full items-center gap-2 px-3 py-1.5 text-left text-[13px] text-ink-soft transition-colors hover:bg-base-700 hover:text-ink"
        @click="go(row)"
      >
        <component :is="row.icon" :size="14" class="shrink-0 text-ink-mute" />
        <span class="truncate">{{ row.label }}</span>
        <span v-if="row.shortcut" class="ml-auto text-[11px] text-ink-mute">
          {{ row.shortcut }}
        </span>
        <ChevronRight v-else :size="13" class="ml-auto shrink-0 text-ink-mute" />
      </button>
    </div>

    <div class="border-t border-base-600 py-1">
      <button
        class="flex w-full items-center gap-2 px-3 py-1.5 text-left text-[13px] text-ink-soft transition-colors hover:bg-base-700 hover:text-ink"
        @click="quit"
      >
        <LogOut :size="14" class="shrink-0 text-ink-mute" />
        退出应用
      </button>
    </div>
  </div>
</template>
