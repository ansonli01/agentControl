import { defineStore } from "pinia";

import { api } from "../api/ipc";
import type { TaskRecord } from "../types/config";

export const useTasksStore = defineStore("tasks", {
  state: () => ({
    records: [] as TaskRecord[],
    kindFilter: "",
    loading: false,
  }),
  getters: {
    successCount: (s) => s.records.filter((r) => r.result === "success").length,
    failedCount: (s) => s.records.filter((r) => r.result === "failed").length,
    filtered: (s) =>
      s.kindFilter ? s.records.filter((r) => r.kind === s.kindFilter) : s.records,
  },
  actions: {
    async load() {
      this.loading = true;
      try {
        this.records = await api.listTasks(300, this.kindFilter || undefined);
      } finally {
        this.loading = false;
      }
    },
    async clear() {
      await api.clearTasks();
      this.records = [];
    },
  },
});