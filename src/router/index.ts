import { createRouter, createWebHashHistory } from "vue-router";

const routes = [
  { path: "/", name: "dashboard", component: () => import("../views/DashboardView.vue"), meta: { title: "总览" } },
  { path: "/agents", name: "agents", component: () => import("../views/AgentsView.vue"), meta: { title: "智能体" } },
  { path: "/projects", name: "projects", component: () => import("../views/ProjectsView.vue"), meta: { title: "项目" } },
  { path: "/credentials", name: "credentials", component: () => import("../views/CredentialsView.vue"), meta: { title: "凭据与 MCP" } },
  { path: "/tasks", name: "tasks", component: () => import("../views/TasksView.vue"), meta: { title: "历史任务" } },
  { path: "/workbuddy", name: "workbuddy", component: () => import("../views/WorkBuddyView.vue"), meta: { title: "WorkBuddy" } },
  { path: "/settings", name: "settings", component: () => import("../views/SettingsView.vue"), meta: { title: "设置" } },
];

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
});