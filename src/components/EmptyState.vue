<script setup lang="ts">
import { Inbox } from "lucide-vue-next";

defineProps<{
  title: string;
  hint?: string;
}>();
</script>

<template>
  <div
    class="flex flex-col items-center justify-center rounded-card border border-dashed border-base-600 bg-base-800/40 px-6 py-12 text-center"
  >
    <Inbox :size="28" class="mb-3 text-ink-mute" />
    <p class="text-sm font-medium text-ink-soft">{{ title }}</p>
    <p v-if="hint" class="mt-1 max-w-md text-xs text-ink-mute">{{ hint }}</p>
  </div>
</template>