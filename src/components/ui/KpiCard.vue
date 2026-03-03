<script setup lang="ts">
defineProps<{
  label: string;
  value: string;
}>();
</script>

<template>
  <div class="kpi-card dbru-surface">
    <span class="kpi-label dbru-text-sm dbru-text-muted">{{ label }}</span>
    <span class="kpi-value dbru-text-lg dbru-text-main">{{ value }}</span>
  </div>
</template>

<style scoped>
.kpi-card {
  padding: var(--dbru-space-4);
  border-radius: var(--dbru-radius-md);
  display: flex;
  flex-direction: column;
  gap: var(--dbru-space-2);
}

.kpi-label {
  font-size: var(--dbru-font-size-sm);
}

.kpi-value {
  font-size: var(--dbru-font-size-lg);
  font-weight: 600;
}
</style>
