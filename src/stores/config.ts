import { defineStore } from "pinia";

import { api } from "../api/ipc";
import type { Provider } from "../types/agent";
import type { AppConfig, Credential, Project } from "../types/config";

export const useConfigStore = defineStore("config", {
  state: () => ({
    config: null as AppConfig | null,
    providers: [] as Provider[],
    error: "",
  }),
  getters: {
    projects: (s) => s.config?.projects ?? [],
    credentials: (s) => s.config?.credentials ?? [],
    overrides: (s) => s.config?.agentOverrides ?? [],
    npmRegistry: (s) => s.config?.npmRegistry ?? "https://registry.npmmirror.com",
  },
  actions: {
    async load() {
      this.config = await api.getConfig();
      this.providers = await api.listProviders();
    },
    async save(next: AppConfig) {
      this.config = await api.saveConfig(next);
    },
    async saveOverride(item: AppConfig["agentOverrides"][number]) {
      this.config = await api.saveAgentOverride(item);
    },
    async saveProject(project: Project) {
      this.config = await api.saveProject(project);
    },
    async removeProject(projectId: string) {
      this.config = await api.deleteProject(projectId);
    },
    async addCredential(provider: string, label: string, secret: string) {
      this.config = await api.saveCredential(provider, label, secret);
    },
    async removeCredential(id: string) {
      this.config = await api.deleteCredential(id);
    },
    credentialOf(provider: string): Credential | null {
      return this.credentials.find((c) => c.provider === provider) ?? null;
    },
  },
});