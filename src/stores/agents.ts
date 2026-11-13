import { defineStore } from "pinia";

import { api, onDetectProgress, onInstallLog, onInstallStatus } from "../api/ipc";
import type { DetectProgress } from "../api/ipc";
import type { AgentDefinition, AgentStatus } from "../types/agent";
import type { InstallLogLine, InstallStatus } from "../types/config";

export type TaskView = {
  status: InstallStatus;
  logs: InstallLogLine[];
};

export const useAgentsStore = defineStore("agents", {
  state: () => ({
    agents: [] as AgentDefinition[],
    statuses: {} as Record<string, AgentStatus>,
    scanning: false,
    progress: null as DetectProgress | null,
    tasks: {} as Record<string, TaskView>,
    selectedId: "",
    eventsBound: false,
  }),
  getters: {
    installedCount: (s) => Object.values(s.statuses).filter((x) => x.installed).length,
    totalCount: (s) => s.agents.length,
    sortedAgents: (s) =>
      s.agents.map((a) => ({
        agent: a,
        status: s.statuses[a.id] ?? null,
      })),
    activeTasks: (s) => Object.values(s.tasks),
  },
  actions: {
    async loadAgents() {
      this.agents = await api.listAgents();
    },
    statusOf(id: string) {
      return this.statuses[id] ?? null;
    },
    agentOf(id: string) {
      return this.agents.find((a) => a.id === id) ?? null;
    },
    bindEvents() {
      if (this.eventsBound) return;
      this.eventsBound = true;
      onDetectProgress((p) => {
        this.progress = p;
      });
      onInstallStatus((s) => {
        const view = this.tasks[s.taskId] ?? { status: s, logs: [] };
        view.status = s;
        this.tasks[s.taskId] = view;
      });
      onInstallLog((l) => {
        const view = this.tasks[l.taskId] ?? {
          status: {
            taskId: l.taskId,
            agentId: l.agentId,
            agentName: "",
            action: "",
            phase: "queued",
            progress: null,
            message: null,
            ts: l.ts,
          },
          logs: [],
        };
        view.logs.push(l);
        if (view.logs.length > 500) view.logs.splice(0, view.logs.length - 500);
        this.tasks[l.taskId] = view;
      });
    },    async scan() {
      this.scanning = true;
      this.progress = null;
      try {
        const list = await api.detectAll();
        const map: Record<string, AgentStatus> = {};
        list.forEach((s) => {
          map[s.agentId] = s;
        });
        this.statuses = map;
      } finally {
        this.scanning = false;
      }
    },
    async install(agentId: string, action: string) {
      const taskId = await api.installStart(agentId, action);
      this.selectedId = agentId;
      return taskId;
    },
    async cancel(taskId: string) {
      await api.installCancel(taskId);
    },
    latestTaskFor(agentId: string) {
      const entries = Object.entries(this.tasks).filter(
        ([, v]) => v.status.agentId === agentId,
      );
      if (entries.length === 0) return null;
      entries.sort((a, b) => b[1].status.ts - a[1].status.ts);
      return entries[0][1];
    },
  },
});