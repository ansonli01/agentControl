<script setup lang="ts">
import { onMounted, reactive, ref } from "vue";
import { Save } from "lucide-vue-next";

import { useAgentsStore } from "../stores/agents";
import { useConfigStore } from "../stores/config";

const config = useConfigStore();
const agents = useAgentsStore();

const form = reactive({
  npmRegistry: "",
  proxy: "",
  downloadDir: "",
  detectTimeoutSecs: 8,
  installTimeoutMinutes: 30,
});

const message = ref("");
const sourceEdits = reactive<Record<string, string>>({});

async function save() {
  const base = config.config;
  if (!base) return;
  await config.save({
    ...base,
    npmRegistry: form.npmRegistry.trim(),
    proxy: form.proxy.trim() ? form.proxy.trim() : null,
    downloadDir: form.downloadDir.trim() ? form.downloadDir.trim() : null,
    detectTimeoutSecs: Number(form.detectTimeoutSecs) || 8,
    installTimeoutMinutes: Number(form.installTimeoutMinutes) || 30,
  });
  message.value = "设置已保存";
  window.setTimeout(() => (message.value = ""), 2000);
}

async function saveSource(agentId: string) {
  const value = sourceEdits[agentId] ?? "";
  await config.saveOverride({
    agentId,
    officialDownload: value.trim() ? value.trim() : null,
    officialPage: null,
    npmPackage: null,
    npmRegistry: null,
    silentArgs: null,
    enabled: null,
  });
  message.value = "安装源已更新";
  window.setTimeout(() => (message.value = ""), 2000);
}

onMounted(() => {
  if (!config.config) return;
  form.npmRegistry = config.config.npmRegistry;
  form.proxy = config.config.proxy ?? "";
  form.downloadDir = config.config.downloadDir ?? "";
  form.detectTimeoutSecs = config.config.detectTimeoutSecs;
  form.installTimeoutMinutes = config.config.installTimeoutMinutes;
  agents.agents.forEach((a) => {
    const ov = config.overrides.find((o) => o.agentId === a.id);
    sourceEdits[a.id] = ov?.officialDownload ?? a.officialDownload ?? "";
  });
});
</script>
<template>
  <div class="space-y-4">
    <header class="mb-1">
      <h1 class="text-h1 font-semibold text-ink">更多</h1>
      <p class="mt-1 text-[13px] text-ink-mute">全局参数、镜像源与自定义智能体安装源</p>
    </header>

    <div class="grid grid-cols-2 gap-4">
    <div class="panel p-4">
      <h3 class="text-h2 text-ink">全局设置</h3>
      <div class="mt-3 space-y-3">
        <div>
          <label class="field-label">npm 镜像源</label>
          <input v-model="form.npmRegistry" class="field" placeholder="https://registry.npmmirror.com" />
        </div>
        <div>
          <label class="field-label">HTTP 代理（可选）</label>
          <input v-model="form.proxy" class="field" placeholder="http://107.0.0.1:7890" />
        </div>
        <div>
          <label class="field-label">下载目录（可选）</label>
          <input v-model="form.downloadDir" class="field" placeholder="留空则使用系统下载目录下的 AgentControl" />
        </div>
        <div class="grid grid-cols-2 gap-3">
          <div>
            <label class="field-label">探测超时（秒）</label>
            <input v-model.number="form.detectTimeoutSecs" type="number" min="3" max="60" class="field" />
          </div>
          <div>
            <label class="field-label">安装超时（分钟）</label>
            <input v-model.number="form.installTimeoutMinutes" type="number" min="1" max="180" class="field" />
          </div>
        </div>
        <button class="btn-primary w-full justify-center" @click="save">
          <Save :size="15" />
          保存设置
        </button>
        <p v-if="message" class="text-xs text-ok">{{ message }}</p>
      </div>
    </div>

    <div class="panel p-4">
      <h3 class="text-h2 text-ink">自定义智能体安装源</h3>
      <p class="mt-1 text-xs text-ink-mute">
        豆包、WorkBuddy 等官方客户端未提供公开直链，可在此填写从官网获取的安装包地址。
      </p>
      <ul class="mt-3 space-y-3">
        <li
          v-for="a in agents.agents.filter((x) => x.installMethod !== 'npm')"
          :key="a.id"
          class="rounded-lg bg-base-700/40 p-3"
        >
          <div class="flex items-center gap-2">
            <span
              class="flex h-6 w-6 items-center justify-center rounded-md text-[10px] font-bold text-white"
              :style="{ background: a.accent }"
            >
              {{ a.iconLetter }}
            </span>
            <span class="text-sm text-ink">{{ a.name }}</span>
            <span class="chip bg-base-600/60 text-ink-mute">{{ a.vendor }}</span>
          </div>
          <div class="mt-2 flex gap-2">
            <input
              v-model="sourceEdits[a.id]"
              class="field font-mono text-xs"
              placeholder="https://example.com/installer.exe"
            />
            <button class="btn-ghost shrink-0" @click="saveSource(a.id)">保存</button>
          </div>
          <p v-if="a.officialPage" class="mt-1 truncate text-[11px] text-ink-mute">
            官网：{{ a.officialPage }}
          </p>
        </li>
      </ul>
    </div>
  </div>
  </div>
</template>