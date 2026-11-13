<script setup lang="ts">
import { onMounted, ref } from "vue";
import { Trash2 } from "lucide-vue-next";

import EmptyState from "../components/EmptyState.vue";
import { useTasksStore } from "../stores/tasks";

const tasks = useTasksStore();
const expanded = ref<string>("");

const kindLabels: Record<string, string> = {
  install: "安装",
  upgrade: "升级",
  uninstall: "卸载",
  workbuddy: "WorkBuddy 任务",
};

const kinds = [
  { key: "", label: "全部" },
  { key: "install", label: "安装" },
  { key: "upgrade", label: "升级" },
  { key: "uninstall", label: "卸载" },
  { key: "workbuddy", label: "WorkBuddy 任务" },
];

async function changeKind(kind: string) {
  tasks.kindFilter = kind;
  await tasks.load();
}

onMounted(() => tasks.load());
</script>

<template>
  <div class="space-y-4">
    <header class="mb-1">
      <h1 class="text-h1 font-semibold text-ink">定时任务</h1>
      <p class="mt-1 text-[13px] text-ink-mute">安装、升级与卸载的历史记录，均落库可查</p>
    </header>

    <div class="panel flex items-center gap-3 p-3">
      <div class="flex items-center gap-1 rounded-lg bg-base-700 p-1">
        <button
          v-for="k in kinds"
          :key="k.key"
          class="rounded-md px-3 py-1.5 text-xs transition-colors duration-150"
          :class="
            tasks.kindFilter === k.key
              ? 'bg-brand text-white'
              : 'text-ink-soft hover:bg-base-600 hover:text-ink'
          "
          @click="changeKind(k.key)"
        >
          {{ k.label }}
        </button>
      </div>
      <div class="ml-auto flex items-center gap-3 text-xs text-ink-mute">
        <span>成功 {{ tasks.successCount }}</span>
        <span>失败 {{ tasks.failedCount }}</span>
        <button class="btn-danger" @click="tasks.clear()">
          <Trash2 :size="14" />
          清空历史
        </button>
      </div>
    </div>

    <div class="panel divide-y divide-base-600/50">
      <div
        v-for="rec in tasks.filtered"
        :key="rec.id"
        class="cursor-pointer px-4 py-3 transition-colors duration-150 hover:bg-base-700/40"
        @click="expanded = expanded === rec.id ? '' : rec.id"
      >
        <div class="flex items-center gap-3">
          <span
            class="h-1.5 w-1.5 shrink-0 rounded-full"
            :class="
              rec.result === 'success'
                ? 'bg-ok'
                : rec.result === 'failed'
                  ? 'bg-err'
                  : 'bg-warn'
            "
          ></span>
          <span class="text-sm text-ink">{{ rec.agentName }}</span>
          <span class="chip bg-base-600/60 text-ink-mute">{{
            kindLabels[rec.kind] ?? rec.kind
          }}</span>
          <span class="truncate text-xs text-ink-mute">{{ rec.source || "—" }}</span>
          <span class="ml-auto text-[11px] text-ink-mute">
            {{ rec.durationMs }} ms · {{ new Date(rec.ts).toLocaleString() }}
          </span>
        </div>
        <pre
          v-if="expanded === rec.id && rec.message"
          class="wb-terminal mt-2 whitespace-pre-wrap font-mono"
          >{{ rec.message }}</pre>
      </div>
    </div>

    <EmptyState
      v-if="tasks.filtered.length === 0"
      title="暂无历史任务"
      hint="执行安装、升级或卸载后会自动记录在此"
    />
  </div>
</template>