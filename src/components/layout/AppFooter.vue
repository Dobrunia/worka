<script setup lang="ts">
const currentYear = new Date().getFullYear();
const version = "0.1.0";
</script>

<template>
  <footer class="app-footer dbru-surface">
    <div class="footer-content">
      <span class="version dbru-text-xs dbru-text-muted">Worka v{{ version }}</span>
      <span class="copyright dbru-text-xs dbru-text-muted">© {{ currentYear }} Добрыня</span>
    </div>
  </footer>
</template>

<style scoped>
.app-footer {
  padding: var(--dbru-space-3) var(--dbru-space-6);
  border-top: 1px solid var(--dbru-color-border);
  font-size: var(--dbru-font-size-xs);
}

.footer-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
</style>
