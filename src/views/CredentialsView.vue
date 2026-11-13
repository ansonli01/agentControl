<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { KeyRound, Save, Trash2 } from "lucide-vue-next";

import EmptyState from "../components/EmptyState.vue";
import { api } from "../api/ipc";
import { useAgentsStore } from "../stores/agents";
import { useConfigStore } from "../stores/config";
import type { McpDoc } from "../types/config";

const config = useConfigStore();
const agents = useAgentsStore();

const tab = ref<"credentials" | "mcp">("credentials");

const provider = ref("");
const label = ref("");
const secret = ref("");
const message = ref("");

const mcpAgents = computed(() =>
  agents.agents.filter((a) => a.mcpConfigPath && a.mcpConfigFormat === "json"),
);

const mcpAgentId = ref("");
const mcpDoc = ref<McpDoc | null>(null);
const mcpContent = ref("");
const mcpError = ref("");

async function addCredential() {
  if (!provider.value) {
    message.value = "请选择服务商标识";
    return;
  }
  try {
    await config.addCredential(provider.value, label.value || "默认", secret.value);
    secret.value = "";
    label.value = "";
    message.value = "密钥已写入系统凭据库";
  } catch (e) {
    message.value = String(e);
  }
  window.setTimeout(() => (message.value = ""), 2500);
}

async function removeCredential(id: string) {
  await config.removeCredential(id);
}

async function loadMcp() {
  if (!mcpAgentId.value) return;
  mcpError.value = "";
  try {
    const doc = await api.loadMcp(mcpAgentId.value);
    mcpDoc.value = doc;
    mcpContent.value = doc.content;
  } catch (e) {
    mcpError.value = String(e);
  }
}

async function saveMcp() {
  if (!mcpAgentId.value) return;
  try {
    const doc = await api.saveMcp(mcpAgentId.value, mcpContent.value);
    mcpDoc.value = doc;
    mcpContent.value = doc.content;
    mcpError.value = "";
  } catch (e) {
    mcpError.value = String(e);
  }
}

onMounted(async () => {
  if (config.providers.length > 0) provider.value = config.providers[0].id;
  if (mcpAgents.value.length > 0) mcpAgentId.value = mcpAgents.value[0].id;
});
</script>
<template>
  <div class="space-y-4">
    <header class="mb-1">
      <h1 class="text-h1 font-semibold text-ink">资料库</h1>
      <p class="mt-1 text-[13px] text-ink-mute">管理 API Key 与 MCP 配置，密钥只写入系统凭据库</p>
    </header>

    <div class="panel flex items-center gap-1 p-1">
      <button
        class="rounded-md px-4 py-1.5 text-xs transition-colors duration-150"
        :class="tab === 'credentials' ? 'bg-brand text-white' : 'text-ink-soft hover:text-ink'"
        @click="tab = 'credentials'"
      >
        凭据管理
      </button>
      <button
        class="rounded-md px-4 py-1.5 text-xs transition-colors duration-150"
        :class="tab === 'mcp' ? 'bg-brand text-white' : 'text-ink-soft hover:text-ink'"
        @click="tab = 'mcp'"
      >
        MCP 配置
      </button>
    </div>

    <div v-if="tab === 'credentials'" class="grid grid-cols-2 gap-4">
      <div class="panel p-4">
        <h3 class="text-h2 text-ink">新增密钥</h3>
        <p class="mt-1 text-xs text-ink-mute">
          密钥保存在 Windows 凭据管理器，配置文件中仅保留掩码。
        </p>
        <div class="mt-3 space-y-3">
          <div>
            <label class="field-label">服务商</label>
            <select v-model="provider" class="field">
              <option v-for="p in config.providers" :key="p.id" :value="p.id">
                {{ p.name }}（{{ p.vendor }}）
              </option>
            </select>
          </div>
          <div>
            <label class="field-label">备注名称</label>
            <input v-model="label" class="field" placeholder="例如 主力账号" />
          </div>
          <div>
            <label class="field-label">密钥</label>
            <input
              v-model="secret"
              type="password"
              class="field"
              placeholder="粘贴 API Key"
            />
          </div>
          <button class="btn-primary w-full justify-center" @click="addCredential">
            <KeyRound :size="15" />
            保存到凭据库
          </button>
          <p v-if="message" class="text-xs text-ok">{{ message }}</p>
        </div>
      </div>

      <div class="panel p-4">
        <h3 class="text-h2 text-ink">已保存的凭据</h3>
        <ul class="mt-3 space-y-2">
          <li
            v-for="c in config.credentials"
            :key="c.id"
            class="flex items-center gap-3 rounded-lg bg-base-700/50 px-3 py-2"
          >
            <div class="min-w-0">
              <p class="truncate text-sm text-ink">{{ c.label }}</p>
              <p class="truncate font-mono text-[11px] text-ink-mute">{{ c.masked }}</p>
            </div>
            <span class="chip bg-base-600/60 text-ink-mute ml-auto">{{ c.provider }}</span>
            <button class="btn-danger !px-2" @click="removeCredential(c.id)">
              <Trash2 :size="14" />
            </button>
          </li>
        </ul>
        <EmptyState
          v-if="config.credentials.length === 0"
          title="尚未保存任何密钥"
          hint="左侧新增后即可在各智能体配置中引用"
        />
      </div>
    </div>

    <div v-else class="grid grid-cols-3 gap-4">
      <div class="panel p-3">
        <h3 class="text-h2 text-ink">目标智能体</h3>
        <ul class="mt-2 space-y-1">
          <li v-for="a in mcpAgents" :key="a.id">
            <button
              class="w-full rounded-lg px-3 py-2 text-left text-sm transition-colors duration-150"
              :class="
                mcpAgentId === a.id
                  ? 'bg-brand/10 text-ink'
                  : 'text-ink-soft hover:bg-base-700 hover:text-ink'
              "
              @click="mcpAgentId = a.id; loadMcp()"
            >
              {{ a.name }}
              <p class="truncate text-[11px] text-ink-mute">{{ a.mcpConfigPath }}</p>
            </button>
          </li>
        </ul>
        <EmptyState
          v-if="mcpAgents.length === 0"
          title="暂无可编辑的 MCP 配置"
          hint="仅有 JSON 格式配置的智能体支持在线编辑"
        />
      </div>

      <div class="panel col-span-2 p-4">
        <div class="flex items-center justify-between">
          <h3 class="text-h2 text-ink">{{ mcpDoc?.path ?? "未加载" }}</h3>
          <div class="flex gap-2">
            <button class="btn-ghost" @click="loadMcp">重新加载</button>
            <button class="btn-primary" @click="saveMcp">
              <Save :size="15" />
              保存
            </button>
          </div>
        </div>
        <textarea
          v-model="mcpContent"
          rows="20"
          spellcheck="false"
          class="field mt-3 resize-none font-mono text-xs leading-5"
          placeholder='{ "mcpServers": {} }'
        ></textarea>
        <p v-if="mcpError" class="mt-2 rounded-md bg-err/10 px-3 py-2 text-xs text-err">
          {{ mcpError }}
        </p>
      </div>
    </div>
  </div>
</template>