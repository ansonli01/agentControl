<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { FolderOpen, Plus, Save, Trash2 } from "lucide-vue-next";

import EmptyState from "../components/EmptyState.vue";
import PlatformCatalogPanel from "../components/PlatformCatalogPanel.vue";
import { useAgentsStore } from "../stores/agents";
import { useConfigStore } from "../stores/config";
import type { Project } from "../types/config";

const config = useConfigStore();
const agents = useAgentsStore();

const tab = ref<"projects" | "catalog">("catalog");
const current = ref<Project | null>(null);
const message = ref("");

const dirty = computed(() => {
  if (!current.value) return false;
  const saved = config.projects.find((p) => p.id === current.value?.id);
  if (!saved) return true;
  return JSON.stringify(saved) !== JSON.stringify(current.value);
});

function newProject() {
  current.value = {
    id: crypto.randomUUID(),
    name: "",
    workspace: "",
    agents: [],
    defaultModel: null,
    extraArgs: null,
    notes: null,
    createdAt: Date.now(),
  };
}

function select(p: Project) {
  current.value = JSON.parse(JSON.stringify(p));
}

async function pickWorkspace() {
  const picked = await open({ directory: true, multiple: false });
  if (typeof picked === "string") {
    current.value!.workspace = picked;
  }
}

async function save() {
  if (!current.value) return;
  if (!current.value.name.trim()) {
    message.value = "请填写项目名称";
    return;
  }
  await config.saveProject(JSON.parse(JSON.stringify(current.value)));
  message.value = "已保存";
  window.setTimeout(() => (message.value = ""), 2000);
}

async function remove() {
  if (!current.value) return;
  await config.removeProject(current.value.id);
  current.value = null;
}

function toggleAgent(id: string) {
  if (!current.value) return;
  const list = current.value.agents;
  const idx = list.indexOf(id);
  if (idx >= 0) list.splice(idx, 1);
  else list.push(id);
}

onMounted(() => {
  if (config.projects.length > 0) select(config.projects[0]);
});
</script>
<template>
  <div class="space-y-4">
    <header class="mb-1">
      <h1 class="text-h1 font-semibold text-ink">专家 · 技能 · 连接器</h1>
      <p class="mt-1 text-[13px] text-ink-mute">
        管理工作区、绑定智能体与默认启动参数；能力目录按平台分页展示 WorkBuddy、豆包工作与小米智能能的技能、插件
      </p>
    </header>

    <div class="flex items-center gap-1 border-b border-base-600/60 pb-2">
      <button
        class="rounded-pill px-3 py-1 text-xs transition-colors duration-150"
        :class="
          tab === 'projects'
            ? 'bg-brand/15 text-brand'
            : 'text-ink-soft hover:bg-base-700 hover:text-ink'
        "
        @click="tab = 'projects'"
      >
        项目
      </button>
      <button
        class="rounded-pill px-3 py-1 text-xs transition-colors duration-150"
        :class="
          tab === 'catalog'
            ? 'bg-brand/15 text-brand'
            : 'text-ink-soft hover:bg-base-700 hover:text-ink'
        "
        @click="tab = 'catalog'"
      >
        能力目录
      </button>
    </div>

    <PlatformCatalogPanel v-if="tab === 'catalog'" />

    <template v-else>
    <div class="grid grid-cols-3 gap-4">
    <div class="panel flex flex-col p-3">
      <div class="flex items-center justify-between px-1 pb-2">
        <h3 class="text-h2 text-ink">项目列表</h3>
        <button class="btn-ghost !px-2" @click="newProject">
          <Plus :size="15" />
          新建
        </button>
      </div>
      <ul class="min-h-0 flex-1 space-y-1 overflow-y-auto">
        <li v-for="p in config.projects" :key="p.id">
          <button
            class="w-full rounded-lg px-3 py-2 text-left transition-colors duration-150"
            :class="
              current?.id === p.id
                ? 'bg-brand/10 text-ink'
                : 'text-ink-soft hover:bg-base-700 hover:text-ink'
            "
            @click="select(p)"
          >
            <p class="truncate text-sm">{{ p.name }}</p>
            <p class="truncate text-[11px] text-ink-mute">{{ p.workspace || "未设置工作区" }}</p>
          </button>
        </li>
      </ul>
    </div>

    <div class="panel col-span-2 p-4">
      <template v-if="current">
        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="field-label">项目名称</label>
            <input v-model="current.name" class="field" placeholder="例如：瓶坯检测系统" />
          </div>
          <div>
            <label class="field-label">工作区根目录</label>
            <div class="flex gap-2">
              <input v-model="current.workspace" class="field" placeholder="E:\obj\my-project" />
              <button class="btn-ghost shrink-0" @click="pickWorkspace">
                <FolderOpen :size="15" />
                浏览
              </button>
            </div>
          </div>
          <div>
            <label class="field-label">默认模型</label>
            <input
              v-model="current.defaultModel"
              class="field"
              placeholder="例如 gpt-5 / claude-sonnet-4"
            />
          </div>
          <div>
            <label class="field-label">附加启动参数</label>
            <input
              v-model="current.extraArgs"
              class="field"
              placeholder="例如 --model qwen3-coder"
            />
          </div>
        </div>

        <div class="mt-4">
          <label class="field-label">绑定智能体</label>
          <div class="flex flex-wrap gap-2">
            <button
              v-for="a in agents.agents"
              :key="a.id"
              class="rounded-pill border px-3 py-1 text-xs transition-colors duration-150"
              :class="
                current.agents.includes(a.id)
                  ? 'border-brand bg-brand/20 text-brand'
                  : 'border-base-600 text-ink-soft hover:border-brand/50 hover:text-ink'
              "
              @click="toggleAgent(a.id)"
            >
              {{ a.name }}
            </button>
          </div>
        </div>

        <div class="mt-4">
          <label class="field-label">备注</label>
          <textarea
            v-model="current.notes"
            rows="3"
            class="field resize-none"
            placeholder="记录该项目的约定、模型偏好与注意事项"
          ></textarea>
        </div>

        <div class="mt-4 flex items-center gap-2 border-t border-base-600/60 pt-4">
          <button class="btn-primary" :disabled="!dirty" @click="save">
            <Save :size="15" />
            保存项目
          </button>
          <button class="btn-danger" @click="remove">
            <Trash2 :size="15" />
            删除
          </button>
          <span v-if="message" class="text-xs text-ok">{{ message }}</span>
        </div>
      </template>
      <EmptyState
        v-else
        title="尚未选择项目"
        hint="从左侧新建一个项目，绑定工作区与智能体"
      />
    </div>
    </div>
    </template>
  </div>
</template>