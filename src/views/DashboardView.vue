<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { use } from "echarts/core";
import { BarChart } from "echarts/charts";
import { GridComponent, TooltipComponent } from "echarts/components";
import { CanvasRenderer } from "echarts/renderers";
import VChart from "vue-echarts";
import { ArrowRight, ChevronDown, CornerDownLeft, FolderKanban, Plus, Sparkles } from "lucide-vue-next";

import { useAgentsStore } from "../stores/agents";
import { useConfigStore } from "../stores/config";
import { useTasksStore } from "../stores/tasks";

use([BarChart, GridComponent, TooltipComponent, CanvasRenderer]);

type Mode = "office" | "dev" | "design";

const router = useRouter();
const agents = useAgentsStore();
const config = useConfigStore();
const tasks = useTasksStore();

const mode = ref<Mode>("dev");
const prompt = ref("");
const busy = ref(false);

const modes: { key: Mode; label: string }[] = [
  { key: "office", label: "办公" },
  { key: "dev", label: "开发" },
  { key: "design", label: "设计" },
];

const suggestions = computed(() => {
  if (mode.value === "office") {
    return ["整理安装清单", "汇总本机智能体", "生成日报", "检查凭据状态", "管理定时任务", "导出历史记录"];
  }
  if (mode.value === "design") {
    return ["换一套界面配色", "设计图标集", "绘制架构图", "调整排版间距", "生成界面规范", "对比深浅主题"];
  }
  return ["扫描本机智能体", "安装缺失的 CLI", "配置 API Key", "接入 MCP 服务", "管理工作区", "查看历史记录"];
});

const cards = computed(() => {
  if (mode.value === "office") {
    return [
      { title: "把安装记录汇总成一页周报", desc: "自动统计成功与失败任务" },
      { title: "给团队整理一份智能体清单", desc: "包含版本、路径与来源" },
      { title: "写一份凭据保管说明", desc: "强调密钥不进明文文件" },
      { title: "做一次依赖与版本体检", desc: "定位过期与冲突的 CLI" },
    ];
  }
  if (mode.value === "design") {
    return [
      { title: "按 WorkBuddy 风格重做工作台", desc: "左图标栏 + 侧栏 + 主工作区" },
      { title: "设计一套状态标签规范", desc: "已安装 / 缺失 / 进行中 / 失败" },
      { title: "绘制一张技术架构图", desc: "探测、配置、安装三层" },
      { title: "做一份浅色主题配色表", desc: "含主色、辅助色与状态色" },
    ];
  }
  return [
    { title: "扫描本机，一眼看清装了几个智能体", desc: "识别 PATH、npm 全局、注册表与安装目录" },
    { title: "把安装流程封装成可复用 Skill", desc: "一键装好 Codex 与 OpenCode" },
    { title: "从零写一个最小可用的 MCP 服务", desc: "接入到任意已装智能体" },
    { title: "做一次不会被围观的多智能体编排", desc: "统一调度各家的 CLI" },
  ];
});

const missingCount = computed(
  () => agents.agents.filter((a) => !agents.statusOf(a.id)?.installed).length,
);

const installCount = computed(
  () => tasks.records.filter((r) => r.kind === "install" && r.result === "success").length,
);

const vendorData = computed(() => {
  const map = new Map<string, number>();
  agents.agents.forEach((a) => {
    if (agents.statusOf(a.id)?.installed) {
      map.set(a.vendor, (map.get(a.vendor) ?? 0) + 1);
    }
  });
  const rows = Array.from(map.entries()).map(([vendor, count]) => ({ vendor, count }));
  return {
    xAxis: { type: "category", data: rows.map((r) => r.vendor), axisLabel: { color: "#86909C" } },
    yAxis: {
      type: "value",
      minInterval: 1,
      axisLabel: { color: "#86909C" },
      splitLine: { lineStyle: { color: "#EEF0F3" } },
    },
    series: [
      {
        type: "bar",
        data: rows.map((r) => r.count),
        barWidth: 18,
        itemStyle: { color: "#3B6EF5", borderRadius: [4, 4, 0, 0] },
      },
    ],
  };
});

async function run(action: string): Promise<void> {
  busy.value = true;
  try {
    if (action === "scan") {
      await agents.scan();
      router.push({ name: "agents" });
      return;
    }
    router.push({ name: action });
  } finally {
    busy.value = false;
  }
}

async function submit(): Promise<void> {
  const text = prompt.value.trim();
  if (!text) {
    await run("scan");
    return;
  }
  if (/扫描||探测||detect/i.test(text)) {
    await run("scan");
    return;
  }
  if (/凭据||密钥||key||mcp/i.test(text)) {
    await run("credentials");
    return;
  }
  if (/项目||空间||workspace/i.test(text)) {
    await run("projects");
    return;
  }
  if (/历史||记录/i.test(text)) {
    await run("tasks");
    return;
  }
  await run("agents");
}

function cardAction(title: string): string {
  if (title.includes("扫描")) return "scan";
  if (title.includes("清单") || title.includes("周报")) return "tasks";
  if (title.includes("凭据") || title.includes("保管")) return "credentials";
  return "agents";
}

onMounted(async () => {
  await tasks.load();
});
</script>

<template>
  <div class="mx-auto flex min-h-full w-full max-w-[880px] flex-col px-6 pb-16 pt-10">
    <h1 class="text-center text-[26px] font-semibold tracking-tight text-ink">AgentControl，我帮你</h1>

    <div class="mt-4 flex justify-center">
      <div class="inline-flex items-center gap-1 rounded-full bg-base-600/70 p-1">
        <button
          v-for="item in modes"
          :key="item.key"
          class="rounded-full px-4 py-1 text-[13px] transition-colors"
          :class="mode === item.key ? 'bg-white font-medium text-ink shadow-card' : 'text-ink-soft hover:text-ink'"
          @click="mode = item.key"
        >
          {{ item.label }}
        </button>
      </div>
    </div>

    <div class="mt-5 flex flex-wrap justify-center gap-2">
      <button
        v-for="item in suggestions"
        :key="item"
        class="wb-tag"
        @click="prompt = item; submit()"
      >
        {{ item }}
      </button>
    </div>

    <div class="mt-4 rounded-2xl border border-base-600 bg-white p-3 shadow-card">
      <textarea
        v-model="prompt"
        rows="3"
        class="w-full resize-none border-none bg-transparent px-1 text-[14px] leading-relaxed text-ink outline-none placeholder:text-ink-mute"
        placeholder="今天你想做什么？@ 添加上下文，/ 调用技能与指令"
        @keydown.enter.exact.prevent="submit"
      ></textarea>
      <div class="mt-1 flex items-center gap-2 px-1">
        <button
          class="flex h-7 w-7 items-center justify-center rounded-full border border-base-600 text-ink-soft transition-colors hover:bg-base-700"
          title="添加上下文"
        >
          <Plus :size="15" />
        </button>
        <div class="ml-auto flex items-center gap-2">
          <button
            class="flex items-center gap-1 rounded-full border border-base-600 px-3 py-1 text-[12px] text-ink-soft transition-colors hover:bg-base-700"
          >
            Hy3
            <ChevronDown :size="12" />
          </button>
          <button
            class="flex h-7 w-7 items-center justify-center rounded-full bg-ink text-white transition-opacity disabled:opacity-40"
            :disabled="busy"
            title="发送"
            @click="submit"
          >
            <CornerDownLeft :size="14" />
          </button>
        </div>
      </div>
    </div>

    <div class="mt-2 flex flex-wrap items-center gap-4 px-1 text-[12px] text-ink-mute">
      <span class="flex items-center gap-1">
        <FolderKanban :size="13" />
        工作区 agentControl
      </span>
      <span>严格分支</span>
      <span class="text-err">允许安全访问</span>
    </div>

    <div class="mt-8 space-y-2">
      <button
        v-for="card in cards"
        :key="card.title"
        class="flex w-full items-center gap-3 rounded-xl border border-base-600 bg-white px-4 py-3 text-left transition-colors hover:border-brand/50"
        @click="run(cardAction(card.title))"
      >
        <Sparkles :size="15" class="shrink-0 text-brand" />
        <span class="min-w-0">
          <span class="block truncate text-[13px] text-ink">{{ card.title }}</span>
          <span class="mt-0.5 block truncate text-[12px] text-ink-mute">{{ card.desc }}</span>
        </span>
        <ArrowRight :size="15" class="ml-auto shrink-0 text-ink-mute" />
      </button>
    </div>

    <div class="mt-8 grid grid-cols-4 gap-3">
      <div class="rounded-xl border border-base-600 bg-white px-4 py-3">
        <p class="text-[12px] text-ink-mute">已安装智能体</p>
        <p class="mt-1 text-xl font-semibold text-ink">{{ agents.installedCount }}</p>
        <p class="mt-0.5 text-[11px] text-ink-mute">共收录 {{ agents.totalCount }} 个</p>
      </div>
      <div class="rounded-xl border border-base-600 bg-white px-4 py-3">
        <p class="text-[12px] text-ink-mute">可安装</p>
        <p class="mt-1 text-xl font-semibold text-brand">{{ missingCount }}</p>
        <p class="mt-0.5 text-[11px] text-ink-mute">来自官方与国内镜像</p>
      </div>
      <div class="rounded-xl border border-base-600 bg-white px-4 py-3">
        <p class="text-[12px] text-ink-mute">成功安装次数</p>
        <p class="mt-1 text-xl font-semibold text-ok">{{ installCount }}</p>
        <p class="mt-0.5 text-[11px] text-ink-mute">来自历史任务记录</p>
      </div>
      <div class="rounded-xl border border-base-600 bg-white px-4 py-3">
        <p class="text-[12px] text-ink-mute">已配置凭据</p>
        <p class="mt-1 text-xl font-semibold text-magic">{{ config.credentials.length }}</p>
        <p class="mt-0.5 text-[11px] text-ink-mute">存于系统凭据库</p>
      </div>
    </div>

    <div class="mt-3 grid grid-cols-5 gap-3">
      <div class="rounded-xl border border-base-600 bg-white p-4 col-span-2">
        <h3 class="text-[13px] font-medium text-ink">已安装厂商分布</h3>
        <VChart class="mt-3 h-48" :option="vendorData" autoresize />
      </div>
      <div class="rounded-xl border border-base-600 bg-white p-4 col-span-3">
        <div class="flex items-center justify-between">
          <h3 class="text-[13px] font-medium text-ink">最近操作</h3>
          <RouterLink :to="{ name: 'tasks' }" class="text-xs text-brand hover:text-brand-light">
            查看全部
          </RouterLink>
        </div>
        <ul class="mt-3 space-y-1">
          <li
            v-for="rec in tasks.records.slice(0, 6)"
            :key="rec.id"
            class="flex items-center gap-3 rounded-lg px-3 py-1.5 hover:bg-base-700"
          >
            <span
              class="h-1.5 w-1.5 shrink-0 rounded-full"
              :class="rec.result === 'success' ? 'bg-ok' : rec.result === 'failed' ? 'bg-err' : 'bg-warn'"
            ></span>
            <span class="text-xs text-ink">{{ rec.agentName }}</span>
            <span class="text-[11px] text-ink-mute">{{ rec.kind }}</span>
            <span class="ml-auto text-[11px] text-ink-mute">{{ new Date(rec.ts).toLocaleString() }}</span>
          </li>
          <li v-if="tasks.records.length === 0" class="py-6 text-center text-xs text-ink-mute">
            暂无操作记录
          </li>
        </ul>
      </div>
    </div>

    <p class="mt-8 text-center text-[11px] text-ink-mute">内容由 AI 生成，请核实重要信息</p>
  </div>
</template>