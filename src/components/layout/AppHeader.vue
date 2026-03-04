<script setup lang="ts">
import { computed } from 'vue';
import { DbrBadge, DbrThemeToggle } from 'dobruniaui-vue';

const props = defineProps<{
  isPaused: boolean;
}>();

const badgeText = computed(() => (props.isPaused ? 'На паузе' : 'Трекинг активен'));
</script>

<template>
  <header class="app-header dbru-surface">
    <div class="logo">
      <h1 class="logo-text dbru-text-lg dbru-text-main">Worka</h1>
    </div>
    <div class="right-side">
      <DbrThemeToggle size="sm" shape="rounded" />
      <DbrBadge
        :variant="isPaused ? undefined : 'primary'"
        :text="badgeText"
        data-testid="status-badge"
      />
    </div>
  </header>
</template>

<style scoped>
.app-header {
  padding: 0 var(--dbru-space-6);
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.right-side {
  display: flex;
  align-items: center;
  gap: var(--dbru-space-4);
}

.logo {
}

.logo-text {
  font-weight: 600;
}
</style>
