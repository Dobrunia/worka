<script setup lang="ts">
import { DbrCard } from "dobruniaui-vue";

defineProps<{
  label: string;
  value: string;
}>();
</script>

<template>
  <DbrCard class="kpi-card">
    <span class="kpi-label dbru-text-xs dbru-text-muted">{{ label }}</span>
    <span class="kpi-value dbru-text-lg dbru-text-main">{{ value }}</span>
  </DbrCard>
</template>

<style scoped>
.kpi-card {
  padding: var(--dbru-space-4) var(--dbru-space-5);
  display: flex;
  flex-direction: column;
  gap: var(--dbru-space-2);
}

.kpi-value {
  font-weight: var(--dbru-font-weight-semibold);
}
</style>
