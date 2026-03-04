<script setup lang="ts">
const currentYear = new Date().getFullYear();
const version = "0.1.0";
</script>

<template>
  <footer class="app-footer dbru-surface">
    <span class="dbru-text-xs dbru-text-muted">Worka v{{ version }}</span>
    <span class="dbru-text-xs dbru-text-muted">© {{ currentYear }} Добрыня</span>
  </footer>
</template>

<style scoped>
.app-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--dbru-space-3) var(--dbru-space-6);
  border-top: 1px solid var(--dbru-color-border);
  flex-shrink: 0;
}
</style>
