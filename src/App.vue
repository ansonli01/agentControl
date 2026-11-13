<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  Bell,
  Bot,
  Boxes,
  CalendarClock,
  ChevronDown,
  Compass,
  Copy,
  Database,
  FolderKanban,
  MessageSquare,
  Minus,
  MoreHorizontal,
  RefreshCw,
  Square,
  SquarePen,
  Workflow,
  X,
} from "lucide-vue-next";

import InstallDrawer from "./components/InstallDrawer.vue";
import UserMenu from "./components/UserMenu.vue";
import { useAgentsStore } from "./stores/agents";
import { useConfigStore } from "./stores/config";
import { useTasksStore } from "./stores/tasks";

const route = useRoute();
const router = useRouter();
const agents = useAgentsStore();
const config = useConfigStore();
const tasks = useTasksStore();

const appWindow = getCurrentWindow();
const maximized = ref(false);
const userMenuOpen = ref(false);
const userMenuRef = ref<HTMLElement | null>(null);
const userBtnRef = ref<HTMLElement | null>(null);

function toggleUserMenu(): void {
  userMenuOpen.value = !userMenuOpen.value;
}

function onGlobalClick(e: MouseEvent): void {
  if (!userMenuOpen.value) return;
  const target = e.target as Node;
  if (userMenuRef.value?.contains(target)) return;
  if (userBtnRef.value?.contains(target)) return;
  userMenuOpen.value = false;
}

function onGlobalKey(e: KeyboardEvent): void {
  if (e.key === "Escape") {
    userMenuOpen.value = false;
    if (agents.selectedId) {
      agents.selectedId = "";
      e.preventDefault();
    }
    return;
  }
  if (e.ctrlKey && e.key === ",") {
    e.preventDefault();
    userMenuOpen.value = false;
    router.push({ name: "settings" });
  }
}

async function syncWindowState(): Promise<void> {
  try {
    maximized.value = await appWindow.isMaximized();
  } catch {
    maximized.value = false;
  }
}

async function minimizeWindow(): Promise<void> {
  await appWindow.minimize();
}

async function toggleMaximize(): Promise<void> {
  await appWindow.toggleMaximize();
  await syncWindowState();
}

async function closeWindow(): Promise<void> {
  await appWindow.close();
}

const navItems = [
  { name: "dashboard", label: "新建任务", icon: SquarePen },
  { name: "agents", label: "助理", icon: Bot, tag: "BETA" },
  { name: "projects", label: "专家·技能·连接器", icon: Boxes },
  { name: "workbuddy", label: "WorkBuddy", icon: Workflow },
  { name: "tasks", label: "定时任务", icon: CalendarClock },
  { name: "credentials", label: "资料库", icon: Database },
  { name: "settings", label: "更多", icon: MoreHorizontal },
];

const menuItems = ["文件", "搜索", "窗口", "帮助"];

// 侧栏宽度（可拖拽调整，持久化到本地）
const SIDEBAR_KEY = "agentcontrol.sidebarWidth";
const SIDEBAR_MIN = 200;
const SIDEBAR_MAX = 420;
const sidebarWidth = ref(248);
const resizing = ref(false);
const handleY = ref(0);
let resizeStartX = 0;
let resizeStartWidth = 0;

function trackHandle(e: MouseEvent): void {
  const host = (e.currentTarget as HTMLElement).parentElement;
  if (!host) return;
  const rect = host.getBoundingClientRect();
  handleY.value = e.clientY - rect.top;
}

function startResize(e: MouseEvent): void {
  resizing.value = true;
  resizeStartX = e.clientX;
  resizeStartWidth = sidebarWidth.value;
  trackHandle(e);
  document.body.style.cursor = "col-resize";
  document.body.style.userSelect = "none";
  document.addEventListener("mousemove", onResize);
  document.addEventListener("mouseup", endResize);
  e.preventDefault();
}

function onResize(e: MouseEvent): void {
  const next = resizeStartWidth + (e.clientX - resizeStartX);
  sidebarWidth.value = Math.min(SIDEBAR_MAX, Math.max(SIDEBAR_MIN, Math.round(next)));
}

function endResize(): void {
  resizing.value = false;
  document.body.style.cursor = "";
  document.body.style.userSelect = "";
  document.removeEventListener("mousemove", onResize);
  document.removeEventListener("mouseup", endResize);
  saveSidebarWidth();
}

function resetSidebar(): void {
  sidebarWidth.value = 248;
  saveSidebarWidth();
}

function saveSidebarWidth(): void {
  try {
    localStorage.setItem(SIDEBAR_KEY, String(sidebarWidth.value));
  } catch {
    /* 忽略存储异常 */
  }
}

const recentTasks = computed(() => tasks.records.slice(0, 12));
const spaces = computed(() => config.projects);

const selectedAgent = computed(() => {
  if (!agents.selectedId) return null;
  const agent = agents.agentOf(agents.selectedId);
  if (!agent) return null;
  return { agent, status: agents.statusOf(agents.selectedId) };
});

function formatDay(ts: number): string {
  const d = new Date(ts);
  const now = new Date();
  if (d.toDateString() === now.toDateString()) {
    const h = String(d.getHours()).padStart(2, "0");
    const m = String(d.getMinutes()).padStart(2, "0");
    return h + ":" + m;
  }
  return d.getMonth() + 1 + "月" + d.getDate() + "日";
}

function go(name: string): void {
  router.push({ name });
}

async function scan(): Promise<void> {
  await agents.scan();
}

onMounted(async () => {
  agents.bindEvents();
  try {
    const saved = Number(localStorage.getItem(SIDEBAR_KEY));
    if (saved >= SIDEBAR_MIN && saved <= SIDEBAR_MAX) sidebarWidth.value = saved;
  } catch {
    /* 忽略存储异常 */
  }
  await agents.loadAgents();
  await config.load();
  await tasks.load();
  await agents.scan();
  await syncWindowState();
  window.addEventListener("resize", syncWindowState);
  document.addEventListener("click", onGlobalClick);
  document.addEventListener("keydown", onGlobalKey);
});

onBeforeUnmount(() => {
  window.removeEventListener("resize", syncWindowState);
  document.removeEventListener("click", onGlobalClick);
  document.removeEventListener("keydown", onGlobalKey);
});
</script>

<template>
  <div class="flex h-full w-full flex-col overflow-hidden bg-base-800">
    <header
      class="flex h-9 shrink-0 items-center justify-between bg-base-850 select-none"
      data-tauri-drag-region
      @dblclick="toggleMaximize"
    >
      <div class="flex h-full items-center gap-3 pl-1" data-tauri-drag-region>
        <div
          class="flex h-5 w-5 items-center justify-center rounded bg-ink text-[10px] font-semibold text-white"
          data-tauri-drag-region
        >
          AC
        </div>
        <span class="text-[12px] font-medium text-ink" data-tauri-drag-region>
          AgentControl · 多智能体控制器
        </span>
        <span class="h-3 w-px bg-base-600" data-tauri-drag-region></span>
        <nav class="flex items-center gap-0.5" @mousedown.stop @dblclick.stop>
          <button
            v-for="item in menuItems"
            :key="item"
            class="rounded px-2 py-0.5 text-[12px] text-ink-soft transition-colors hover:bg-base-700 hover:text-ink"
            @mousedown.stop
            @dblclick.stop
          >
            {{ item }}
          </button>
        </nav>
      </div>
      <div class="flex h-full items-stretch">
        <div class="flex items-center gap-3 pr-2" data-tauri-drag-region>
          <span class="text-[12px] text-ink-mute" data-tauri-drag-region>
            已安装 {{ agents.installedCount }} / {{ agents.totalCount }}
          </span>
          <button
            class="btn-ghost h-7 px-2 text-xs"
            :disabled="agents.scanning"
            @click="scan"
            @mousedown.stop
            @dblclick.stop
          >
            <RefreshCw :size="13" :class="agents.scanning ? 'animate-spin' : ''" />
            {{ agents.scanning ? "扫描中…" : "一键扫描" }}
          </button>
        </div>
        <button
          class="title-btn"
          title="最小化"
          @mousedown.stop
          @click="minimizeWindow"
        >
          <Minus :size="15" />
        </button>
        <button
          class="title-btn"
          :title="maximized ? '还原' : '最大化'"
          @mousedown.stop
          @click="toggleMaximize"
        >
          <Copy v-if="maximized" :size="12" />
          <Square v-else :size="12" />
        </button>
        <button
          class="title-btn title-btn-close"
          title="关闭"
          @mousedown.stop
          @click="closeWindow"
        >
          <X :size="15" />
        </button>
      </div>
    </header>

    <div class="flex min-h-0 flex-1">
    <aside
      class="relative flex shrink-0 flex-col bg-base-850"
      :style="{ width: `${sidebarWidth}px` }"
    >
      <div class="flex items-start justify-between px-4 pb-2 pt-3">
        <div>
          <p class="text-[13px] font-semibold text-ink">AgentControl</p>
          <p class="mt-0.5 text-[11px] text-ink-mute">v0.1.0</p>
        </div>
        <button class="btn-ghost h-7 shrink-0 whitespace-nowrap px-2 text-xs" title="发现应用" @click="go('agents')">
          <Compass :size="14" />
          发现应用
        </button>
      </div>

      <nav class="space-y-0.5 px-2 pb-3">
        <RouterLink
          v-for="item in navItems"
          :key="item.name"
          :to="{ name: item.name }"
          class="wb-nav"
          :class="route.name === item.name ? 'wb-nav-active' : ''"
        >
          <component :is="item.icon" :size="16" class="shrink-0" />
          <span class="truncate">{{ item.label }}</span>
          <span
            v-if="item.tag"
            class="ml-auto rounded bg-brand/10 px-1.5 py-0.5 text-[10px] font-medium text-brand"
          >
            {{ item.tag }}
          </span>
        </RouterLink>
      </nav>

      <div class="border-t border-base-600"></div>

      <div class="min-h-0 flex-1 overflow-y-auto px-2 py-2">
        <p class="px-2 py-1 text-[11px] text-ink-mute">任务 ({{ recentTasks.length }})</p>
        <RouterLink
          v-for="rec in recentTasks"
          :key="rec.id"
          :to="{ name: 'tasks' }"
          class="flex items-center gap-2 rounded-lg px-2 py-1.5 text-[13px] text-ink-soft hover:bg-base-700 hover:text-ink"
        >
          <span
            class="h-1.5 w-1.5 shrink-0 rounded-full"
            :class="rec.result === 'success' ? 'bg-ok' : rec.result === 'failed' ? 'bg-err' : 'bg-warn'"
          ></span>
          <span class="truncate">{{ rec.agentName }}</span>
          <span class="ml-auto shrink-0 text-[11px] text-ink-mute">{{ formatDay(rec.ts) }}</span>
        </RouterLink>
        <RouterLink
          v-if="recentTasks.length"
          :to="{ name: 'tasks' }"
          class="mt-1 flex items-center gap-1 px-2 py-1 text-[12px] text-ink-mute hover:text-ink"
        >
          <ChevronDown :size="13" />
          查看更多 ({{ tasks.records.length }})
        </RouterLink>

        <p class="mt-2 px-2 py-1 text-[11px] text-ink-mute">空间 ({{ spaces.length }})</p>
        <RouterLink
          v-for="space in spaces"
          :key="space.id"
          :to="{ name: 'projects' }"
          class="flex items-center gap-2 rounded-lg px-2 py-1.5 text-[13px] text-ink-soft hover:bg-base-700 hover:text-ink"
        >
          <FolderKanban :size="13" class="shrink-0 text-ink-mute" />
          <span class="truncate">{{ space.name }}</span>
        </RouterLink>
        <RouterLink
          v-if="!spaces.length"
          :to="{ name: 'projects' }"
          class="px-2 py-1 text-[12px] text-ink-mute hover:text-ink"
        >
          还没有空间，去新建
        </RouterLink>
      </div>

      <div class="relative border-t border-base-600 px-3 py-2">
        <div class="flex items-center gap-2">
          <button
            ref="userBtnRef"
            class="flex min-w-0 flex-1 items-center gap-2 rounded-lg px-1.5 py-1 text-left transition-colors hover:bg-base-700"
            title="账户与快捷入口"
            @click="toggleUserMenu"
          >
            <div
              class="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-brand text-[11px] font-semibold text-white"
            >
              AC
            </div>
            <span class="truncate text-[13px] text-ink">AgentControl</span>
          </button>
          <button class="rail-btn" title="通知">
            <Bell :size="15" />
          </button>
          <button class="rail-btn" title="消息">
            <MessageSquare :size="15" />
          </button>
        </div>

        <div
          v-if="userMenuOpen"
          ref="userMenuRef"
          class="absolute bottom-full left-2 z-50 mb-1"
        >
          <UserMenu @close="userMenuOpen = false" />
        </div>
      </div>

      <div
        class="resize-handle"
        :class="{ 'resize-handle-active': resizing }"
        :style="{ '--handle-y': `${handleY}px` }"
        @mouseenter="trackHandle"
        @mousemove="trackHandle"
        @mousedown="startResize"
        @dblclick="resetSidebar"
      ></div>
    </aside>

    <div class="flex min-w-0 flex-1 flex-col">
      <main class="min-h-0 flex-1 overflow-y-auto">
        <RouterView v-slot="{ Component }">
          <Transition name="fade" mode="out-in">
            <component :is="Component" />
          </Transition>
        </RouterView>
      </main>
    </div>

    <InstallDrawer
      v-if="selectedAgent"
      :agent="selectedAgent.agent"
      :status="selectedAgent.status"
      @close="agents.selectedId = ''"
    />
    </div>
  </div>
</template>

<style scoped>
.rail-btn {
  @apply flex h-8 w-8 items-center justify-center rounded-lg text-ink-soft transition-colors hover:bg-base-700 hover:text-ink;
}

.title-btn {
  @apply flex w-11 items-center justify-center text-ink-soft transition-colors hover:bg-base-700 hover:text-ink;
}

.title-btn-close {
  @apply hover:bg-err hover:text-white;
}

.resize-handle {
  position: absolute;
  top: 0;
  right: -7px;
  width: 14px;
  height: 100%;
  cursor: col-resize;
  z-index: 20;
  overflow: hidden;
}

/* 悬停时在分隔处散射出的绿色光晕：5px 细光柱，高度占整体的 80%，中心浓、四周渐隐 */
.resize-handle::after {
  content: "";
  position: absolute;
  top: var(--handle-y, 50%);
  left: 50%;
  width: 5px;
  height: 80%;
  transform: translate(-50%, -50%);
  border-radius: 999px;
  pointer-events: none;
  opacity: 0;
  transition: opacity 0.18s ease;
  filter: blur(0.5px);
  background: linear-gradient(
    to bottom,
    rgba(34, 197, 94, 0) 0%,
    rgba(34, 197, 94, 0.18) 14%,
    rgba(34, 197, 94, 0.55) 32%,
    rgba(34, 197, 94, 0.9) 50%,
    rgba(34, 197, 94, 0.55) 68%,
    rgba(34, 197, 94, 0.18) 86%,
    rgba(34, 197, 94, 0) 100%
  );
}

.resize-handle:hover::after,
.resize-handle-active::after {
  opacity: 1;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.18s ease, transform 0.18s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
  transform: translateY(6px);
}
</style>