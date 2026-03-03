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
  (e: 'update:activeTab', value: string): void;
}>();

function switchTab(tabId: string) {
  emit('update:activeTab', tabId);
}
</script>

<template>
  <nav class="app-navigation dbru-surface">
    <div class="tabs" role="tablist">
      <DbrButton
        v-for="tab in tabs"
        :key="tab.id"
        :variant="activeTab === tab.id ? 'primary' : 'ghost'"
        size="sm"
        :class="{ active: activeTab === tab.id }"
        @click="switchTab(tab.id)"
      >
        {{ tab.label }}
      </DbrButton>
    </div>
  </nav>
</template>

<style scoped>
.app-navigation {
  padding: 0 var(--dbru-space-6);
  border-bottom: 1px solid var(--dbru-color-border);
}

.tabs {
  display: flex;
  gap: var(--dbru-space-2);
}

.active {
  font-weight: 600;
}
</style>
