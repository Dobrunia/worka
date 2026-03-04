<script setup lang="ts">
import { DbrButton } from "dobruniaui-vue";

export interface Tab {
  id: string;
  label: string;
}

defineProps<{
  tabs: Tab[];
  activeTab: string;
}>();

const emit = defineEmits<{
  (e: "update:activeTab", value: string): void;
}>();

function switchTab(tabId: string) {
  emit("update:activeTab", tabId);
}
</script>

<template>
  <nav class="app-navigation dbru-surface" role="tablist">
    <DbrButton
      v-for="tab in tabs"
      :key="tab.id"
      :variant="activeTab === tab.id ? 'primary' : 'ghost'"
      size="sm"
      @click="switchTab(tab.id)"
    >
      {{ tab.label }}
    </DbrButton>
  </nav>
</template>

<style scoped>
.app-navigation {
  display: flex;
  gap: var(--dbru-space-2);
  padding: var(--dbru-space-2) var(--dbru-space-6);
  border-bottom: 1px solid var(--dbru-color-border);
  flex-shrink: 0;
}
</style>
