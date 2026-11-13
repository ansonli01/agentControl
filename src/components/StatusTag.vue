<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  status: "installed" | "missing" | "running" | "failed" | "unknown";
  label?: string;
}>();

const palette: Record<string, string> = {
  installed: "bg-ok/10 text-ok",
  missing: "bg-base-600/60 text-ink-mute",
  running: "bg-brand/20 text-brand",
  failed: "bg-err/10 text-err",
  unknown: "bg-warn/10 text-warn",
};

const text = computed(() => {
  if (props.label) return props.label;
  switch (props.status) {
    case "installed":
      return "已安装";
    case "missing":
      return "未安装";
    case "running":
      return "安装中";
    case "failed":
      return "失败";
    default:
      return "版本未知";
  }
});
</script>

<template>
  <span class="chip" :class="palette[props.status]">
    <span
      v-if="props.status === 'running'"
      class="h-1.5 w-1.5 rounded-full bg-brand animate-pulse-ring"
    ></span>
    {{ text }}
  </span>
</template>