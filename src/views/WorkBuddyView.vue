<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { listen } from "@tauri-apps/api/event";
import {
  Boxes,
  Bot,
  ExternalLink,
  FolderOpen,
  Plug,
  Play,
  RefreshCw,
  Sparkles,
  Square,
  Terminal,
} from "lucide-vue-next";

import { api } from "../api/ipc";
import EmptyState from "../components/EmptyState.vue";
import type {
  WorkBuddyCliInfo,
  WorkBuddyCliModel,
  WorkBuddyCliRun,
  WorkBuddyLogLine,
  WorkBuddyOverview,
  WorkBuddySession,
  WorkBuddySkill,
  WorkBuddyTaskStatus,
} from "../types/workbuddy";

const overview = ref<WorkBuddyOverview | null>(null);
const loading = ref(false);
const error = ref("");
const activeTab = ref<"sessions" | "skills" | "mcp" | "connectors" | "models">(
  "sessions",
);

// CLI 引擎
const cli = ref<WorkBuddyCliInfo | null>(null);
const cliPrompt = ref("");
const cliCwd = ref("");
const cliRunning = ref(false);
const cliResult = ref<WorkBuddyCliRun | null>(null);

// 派发任务
const cliModels = ref<WorkBuddyCliModel[]>([]);
const dispatchModel = ref("");
const dispatchPrompt = ref("");
const dispatchCwd = ref("");
const dispatchTaskId = ref("");
const dispatchPhase = ref<"idle" | "running" | "finished" | "failed" | "cancelled">(
  "idle",
);
const dispatchLogs = ref<WorkBuddyLogLine[]>([]);
const dispatchMessage = ref("");

const unlisteners: Array<() => void> = [];

const status = computed(() => overview.value?.status ?? null);

const tabs = computed(() => [
  { key: "sessions" as const, label: "会话", count: overview.value?.sessions.length ?? 0 },
  { key: "skills" as const, label: "技能", count: overview.value?.skills.length ?? 0 },
  { key: "mcp" as const, label: "MCP 服务", count: overview.value?.mcps.length ?? 0 },
  {
    key: "connectors" as const,
    label: "连接器",
    count: overview.value?.connectors.length ?? 0,
  },
  { key: "models" as const, label: "模型", count: overview.value?.models.length ?? 0 },
]);

async function refresh() {
  loading.value = true;
  error.value = "";
  try {
    overview.value = await api.workbuddyOverview();
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

async function refreshCli() {
  try {
    cli.value = await api.workbuddyCliInfo();
  } catch (e) {
    error.value = String(e);
  }
}

async function runCli() {
  const prompt = cliPrompt.value.trim();
  if (!prompt || cliRunning.value) return;
  cliRunning.value = true;
  cliResult.value = null;
  try {
    cliResult.value = await api.workbuddyCliRun(
      prompt,
      cliCwd.value.trim() || undefined,
    );
  } catch (e) {
    error.value = String(e);
  } finally {
    cliRunning.value = false;
  }
}

function copyText(text: string) {
  void navigator.clipboard?.writeText(text);
}

async function refreshModels() {
  try {
    cliModels.value = await api.workbuddyCliModels();
    if (!dispatchModel.value && cliModels.value.length) {
      const preferred =
        cliModels.value.find((m) => m.supportsToolCall) ?? cliModels.value[0];
      dispatchModel.value = preferred.model;
    }
  } catch (e) {
    error.value = String(e);
  }
}

async function startDispatch() {
  const prompt = dispatchPrompt.value.trim();
  if (!prompt || dispatchPhase.value === "running") return;
  dispatchLogs.value = [];
  dispatchMessage.value = "";
  dispatchPhase.value = "running";
  try {
    dispatchTaskId.value = await api.workbuddyDispatchStart(
      prompt,
      dispatchCwd.value.trim() || undefined,
      dispatchModel.value || undefined,
    );
  } catch (e) {
    dispatchPhase.value = "failed";
    dispatchMessage.value = String(e);
  }
}

async function cancelDispatch() {
  if (!dispatchTaskId.value) return;
  try {
    await api.workbuddyDispatchCancel(dispatchTaskId.value);
  } catch (e) {
    error.value = String(e);
  }
}

async function open(target: string) {
  if (!target) return;
  try {
    await api.workbuddyOpen(target);
  } catch (e) {
    error.value = String(e);
  }
}

function openDataDir() {
  if (status.value?.dataDir) open(status.value.dataDir);
}

function openSessionsDir() {
  if (status.value?.dataDir) {
    open(`${status.value.dataDir}/workspace/sessions`);
  }
}

function formatTime(ms: number): string {
  if (!ms) return "-";
  return new Date(ms).toLocaleString("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function sessionTitle(session: WorkBuddySession): string {
  return session.id.replace(/-/g, "").slice(0, 12);
}

function skillSource(skill: WorkBuddySkill): string {
  if (skill.source === "builtin") return "内置";
  if (skill.source === "userSettings") return "已安装";
  return skill.source ?? "未知";
}

onMounted(async () => {
  refresh();
  refreshCli();
  refreshModels();

  const offLog = await listen<WorkBuddyLogLine>("workbuddy://log", (e) => {
    if (e.payload.taskId !== dispatchTaskId.value) return;
    dispatchLogs.value.push(e.payload);
  });
  const offStatus = await listen<WorkBuddyTaskStatus>(
    "workbuddy://status",
    (e) => {
      if (e.payload.taskId !== dispatchTaskId.value) return;
      dispatchPhase.value = e.payload.phase as typeof dispatchPhase.value;
      dispatchMessage.value = e.payload.message ?? "";
    },
  );
  unlisteners.push(offLog, offStatus);
});

onUnmounted(() => {
  unlisteners.forEach((off) => off());
});
</script>

<template>
  <div class="space-y-4">
    <header class="mb-1 flex items-start justify-between gap-4">
      <div>
        <h1 class="text-h1 font-semibold text-ink">WorkBuddy</h1>
        <p class="mt-1 text-[13px] text-ink-mute">
          读取本机 WorkBuddy 的运行状态与能力（会话 / 技能 / MCP / 连接器 / 模型）
        </p>
      </div>
      <div class="flex shrink-0 items-center gap-2">
        <button class="btn-ghost h-8 px-3 text-xs" :disabled="loading" @click="refresh">
          <RefreshCw :size="14" :class="loading ? 'animate-spin' : ''" />
          刷新
        </button>
        <button class="btn-ghost h-8 px-3 text-xs" @click="open('https://www.workbuddy.cn')">
          <ExternalLink :size="14" />
          官网
        </button>
      </div>
    </header>

    <div v-if="error" class="panel border-err/40 bg-err/5 p-3 text-[13px] text-err">
      {{ error }}
    </div>

    <div v-if="status" class="grid grid-cols-4 gap-3">
      <div class="panel p-3">
        <p class="text-[11px] text-ink-mute">客户端</p>
        <p class="mt-1 flex items-center gap-1.5 text-[15px] font-semibold text-ink">
          <span
            class="h-2 w-2 rounded-full"
            :class="status.running ? 'bg-ok' : 'bg-base-600'"
          ></span>
          {{ status.installed ? (status.running ? "运行中" : "已安装") : "未安装" }}
        </p>
        <p class="mt-1 text-[11px] text-ink-mute">
          {{ status.version ? `v${status.version}` : "版本未知" }}
        </p>
      </div>
      <div class="panel p-3">
        <p class="text-[11px] text-ink-mute">会话</p>
        <p class="mt-1 text-[15px] font-semibold text-ink">{{ status.sessionCount }}</p>
        <p class="mt-1 text-[11px] text-ink-mute">最近 {{ overview?.sessions.length ?? 0 }} 条已列出</p>
      </div>
      <div class="panel p-3">
        <p class="text-[11px] text-ink-mute">技能</p>
        <p class="mt-1 text-[15px] font-semibold text-ink">{{ status.skillCount }}</p>
        <p class="mt-1 text-[11px] text-ink-mute">来自 SkillHub 市场</p>
      </div>
      <div class="panel p-3">
        <p class="text-[11px] text-ink-mute">MCP / 连接器</p>
        <p class="mt-1 text-[15px] font-semibold text-ink">
          {{ status.mcpCount }} / {{ status.connectorCount }}
        </p>
        <p class="mt-1 text-[11px] text-ink-mute">模型 {{ status.modelCount }} 个</p>
      </div>
    </div>

    <div class="panel p-3">
      <div class="flex items-center gap-2">
        <FolderOpen :size="14" class="text-ink-mute" />
        <span class="text-[12px] text-ink-mute">数据目录</span>
        <code class="truncate text-[12px] text-ink">{{ status?.dataDir ?? "-" }}</code>
        <button class="btn-ghost ml-auto h-7 px-2 text-xs" @click="openDataDir">
          <FolderOpen :size="13" />
          打开目录
        </button>
        <button class="btn-ghost h-7 px-2 text-xs" @click="openSessionsDir">会话目录</button>
      </div>
      <div v-if="status?.installDir" class="mt-2 flex items-center gap-2">
        <Plug :size="14" class="text-ink-mute" />
        <span class="text-[12px] text-ink-mute">安装目录</span>
        <code class="truncate text-[12px] text-ink">{{ status.installDir }}</code>
      </div>
    </div>

    <div class="panel p-3">
      <div class="flex items-center gap-2">
        <Terminal :size="14" class="text-ink-mute" />
        <span class="text-[13px] font-medium text-ink">CLI 引擎</span>
        <span
          class="chip bg-base-600/60 text-ink-mute"
        >
          {{ cli?.available ? "就绪" : "未就绪" }}
        </span>
        <span v-if="cli?.version" class="text-[11px] text-ink-mute">
          codebuddy {{ cli.version }} · {{ cli.bundle }}
        </span>
        <span v-if="cli?.source" class="text-[11px] text-ink-mute">
          来源 {{ cli.source }}
        </span>
        <button class="btn-ghost ml-auto h-7 px-2 text-xs" @click="refreshCli">
          <RefreshCw :size="13" />
          重新检测
        </button>
      </div>

      <p v-if="cli?.reason" class="mt-2 text-[12px] text-warn">
        {{ cli.reason }}
      </p>
      <div v-if="cli?.cliPath" class="mt-2 flex items-center gap-2">
        <span class="shrink-0 text-[12px] text-ink-mute">CLI</span>
        <code class="truncate text-[12px] text-ink">{{ cli.cliPath }}</code>
        <button class="btn-ghost h-6 px-2 text-[11px]" @click="copyText(cli.cliPath)">
          复制
        </button>
      </div>
      <div v-if="cli?.nodePath" class="mt-1 flex items-center gap-2">
        <span class="shrink-0 text-[12px] text-ink-mute">Node</span>
        <code class="truncate text-[12px] text-ink">{{ cli.nodePath }}</code>
      </div>

      <div class="mt-3 flex gap-2">
        <input
          v-model="cliPrompt"
          class="field flex-1"
          placeholder="给 WorkBuddy CLI 发一句话，例如：列出当前目录下的文件"
          @keyup.enter="runCli"
        />
        <input
          v-model="cliCwd"
          class="field w-64"
          placeholder="工作目录（可选）"
        />
        <button
          class="btn-primary h-9 px-3 text-xs"
          :disabled="cliRunning || !cli?.available || !cliPrompt.trim()"
          @click="runCli"
        >
          {{ cliRunning ? "执行中…" : "试跑" }}
        </button>
      </div>

      <div
        v-if="cliResult"
        class="mt-3 rounded-lg border border-base-600 bg-base-700/60 p-3"
      >
        <div class="mb-1.5 flex items-center gap-2 text-[11px] text-ink-mute">
          <span
            class="h-1.5 w-1.5 rounded-full"
            :class="cliResult.ok ? 'bg-ok' : 'bg-err'"
          ></span>
          <span>
            {{ cliResult.ok ? "执行成功" : "执行失败" }}
            <template v-if="cliResult.exitCode !== null">
              · exit {{ cliResult.exitCode }}
            </template>
            · {{ (cliResult.elapsedMs / 1000).toFixed(1) }}s
          </span>
        </div>
        <p
          v-if="cliResult.authRequired"
          class="mb-2 text-[12px] text-warn"
        >
          CLI 尚未登录：在终端运行 <code>codebuddy</code> 后执行
          <code>/login</code> 完成登录，即可复用 WorkBuddy 账号与模型配置。
        </p>
        <pre
          class="max-h-64 overflow-auto whitespace-pre-wrap break-words font-mono text-[12px] leading-relaxed text-ink"
        >{{ cliResult.stdout || cliResult.stderr || "（无输出）" }}</pre>
      </div>
    </div>

    <div class="panel p-3">
      <div class="flex items-center gap-2">
        <Play :size="14" class="text-ink-mute" />
        <span class="text-[13px] font-medium text-ink">派发任务给 WorkBuddy</span>
        <span
          class="chip bg-base-600/60 text-ink-mute"
        >
          {{
            dispatchPhase === "running"
              ? "执行中"
              : dispatchPhase === "finished"
                ? "已完成"
                : dispatchPhase === "failed"
                  ? "失败"
                  : dispatchPhase === "cancelled"
                    ? "已取消"
                    : "待命"
          }}
        </span>
        <button
          class="btn-ghost ml-auto h-7 px-2 text-xs"
          :disabled="dispatchPhase === 'running' || !dispatchTaskId"
          @click="cancelDispatch"
        >
          <Square :size="12" />
          取消
        </button>
      </div>

      <div class="mt-3 flex flex-col gap-2">
        <div class="flex gap-2">
          <select v-model="dispatchModel" class="field w-64">
            <option value="">自动选择模型</option>
            <option
              v-for="m in cliModels"
              :key="m.model"
              :value="m.model"
            >
              {{ m.name }}{{ m.local ? "（本机）" : "" }}
            </option>
          </select>
          <input
            v-model="dispatchCwd"
            class="field flex-1"
            placeholder="工作目录（可选，默认取当前目录）"
          />
        </div>
        <textarea
          v-model="dispatchPrompt"
          class="field min-h-[72px] resize-y"
          placeholder="描述要交给 WorkBuddy 的任务，例如：读取 package.json 并总结依赖"
        ></textarea>
        <div class="flex items-center gap-2">
          <button
            class="btn-primary h-9 px-3 text-xs"
            :disabled="dispatchPhase === 'running' || !dispatchPrompt.trim()"
            @click="startDispatch"
          >
            {{ dispatchPhase === "running" ? "执行中…" : "派发" }}
          </button>
          <span class="text-[11px] text-ink-mute">
            通过 CLI 非交互模式运行，具备 Read / Write / Bash 等工具权限
          </span>
        </div>
      </div>

      <div
        v-if="dispatchLogs.length || dispatchMessage"
        class="mt-3 rounded-lg border border-base-600 bg-base-700/60 p-3"
      >
        <div class="mb-1.5 flex items-center gap-2 text-[11px] text-ink-mute">
          <span
            class="h-1.5 w-1.5 rounded-full"
            :class="
              dispatchPhase === 'finished'
                ? 'bg-ok'
                : dispatchPhase === 'running'
                  ? 'bg-brand'
                  : 'bg-err'
            "
          ></span>
          <span>{{ dispatchMessage || "输出" }}</span>
        </div>
        <pre
          class="max-h-72 overflow-auto whitespace-pre-wrap break-words font-mono text-[12px] leading-relaxed text-ink"
        >{{
            dispatchLogs
              .map((l) => (l.stream === "text" ? l.text : `[${l.stream}] ${l.text}`))
              .join("\n") || "（无输出）"
          }}</pre>
      </div>
    </div>

    <div class="panel overflow-hidden">
      <div class="flex items-center gap-1 border-b border-base-600 px-3 py-2">
        <button
          v-for="tab in tabs"
          :key="tab.key"
          class="rounded-md px-3 py-1.5 text-xs transition-colors duration-150"
          :class="
            activeTab === tab.key
              ? 'bg-brand text-white'
              : 'text-ink-soft hover:bg-base-600 hover:text-ink'
          "
          @click="activeTab = tab.key"
        >
          {{ tab.label }} ({{ tab.count }})
        </button>
      </div>

      <div v-if="loading" class="p-6 text-center text-[13px] text-ink-mute">正在读取 WorkBuddy 数据…</div>

      <template v-else-if="overview">
        <div v-if="activeTab === 'sessions'" class="divide-y divide-base-600">
          <div
            v-for="session in overview.sessions"
            :key="session.id"
            class="flex items-center gap-3 px-4 py-2.5"
          >
            <Bot :size="15" class="shrink-0 text-ink-mute" />
            <div class="min-w-0 flex-1">
              <p class="truncate text-[13px] text-ink">{{ sessionTitle(session) }}</p>
              <p class="truncate text-[11px] text-ink-mute">{{ session.id }}</p>
            </div>
            <span class="shrink-0 text-[11px] text-ink-mute">
              {{ formatTime(session.updatedAt) }}
            </span>
          </div>
          <EmptyState v-if="!overview.sessions.length" title="没有会话记录" hint="在 WorkBuddy 中新建会话后会显示在这里" />
        </div>

        <div v-else-if="activeTab === 'skills'" class="divide-y divide-base-600">
          <div
            v-for="skill in overview.skills"
            :key="skill.name"
            class="flex items-start gap-3 px-4 py-2.5"
          >
            <Sparkles :size="15" class="mt-0.5 shrink-0 text-brand" />
            <div class="min-w-0 flex-1">
              <p class="truncate text-[13px] text-ink">{{ skill.name }}</p>
              <p v-if="skill.description" class="truncate text-[11px] text-ink-mute">
                {{ skill.description }}
              </p>
            </div>
            <span class="chip shrink-0 bg-base-600/60 text-ink-mute">
              {{ skillSource(skill) }}
            </span>
          </div>
          <EmptyState v-if="!overview.skills.length" title="没有已安装的技能" hint="可在 WorkBuddy 的技能市场中安装" />
        </div>

        <div v-else-if="activeTab === 'mcp'" class="divide-y divide-base-600">
          <div v-for="mcp in overview.mcps" :key="mcp.name" class="flex items-center gap-3 px-4 py-2.5">
            <Plug :size="15" class="shrink-0 text-ink-mute" />
            <div class="min-w-0 flex-1">
              <p class="truncate text-[13px] text-ink">{{ mcp.name }}</p>
              <p class="truncate text-[11px] text-ink-mute">
                {{ mcp.url || mcp.command || "本地进程" }}
              </p>
            </div>
            <span class="chip shrink-0 bg-base-600/60 text-ink-mute">
              {{ mcp.kind || (mcp.command ? "stdio" : "http") }}
            </span>
            <span
              v-if="mcp.disabled"
              class="chip shrink-0 bg-base-600/60 text-ink-mute"
            >
              已禁用
            </span>
          </div>
          <EmptyState v-if="!overview.mcps.length" title="没有配置 MCP 服务" hint="可在 WorkBuddy 中添加 MCP 服务" />
        </div>

        <div v-else-if="activeTab === 'connectors'" class="divide-y divide-base-600">
          <div v-for="name in overview.connectors" :key="name" class="flex items-center gap-3 px-4 py-2.5">
            <Boxes :size="15" class="shrink-0 text-ink-mute" />
            <p class="truncate text-[13px] text-ink">{{ name }}</p>
          </div>
          <EmptyState v-if="!overview.connectors.length" title="没有已接入的连接器" hint="可在 WorkBuddy 中接入连接器" />
        </div>

        <div v-else class="divide-y divide-base-600">
          <div v-for="model in overview.models" :key="model.id" class="flex items-center gap-3 px-4 py-2.5">
            <Sparkles :size="15" class="shrink-0 text-ink-mute" />
            <div class="min-w-0 flex-1">
              <p class="truncate text-[13px] text-ink">{{ model.name || model.id }}</p>
              <p class="truncate text-[11px] text-ink-mute">{{ model.url ?? "内置模型" }}</p>
            </div>
            <span class="chip shrink-0 bg-base-600/60 text-ink-mute">{{ model.vendor ?? "自定义" }}</span>
          </div>
          <EmptyState v-if="!overview.models.length" title="没有自定义模型" hint="可在 WorkBuddy 的模型设置中新增" />
        </div>
      </template>
    </div>
  </div>
</template>
