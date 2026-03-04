<script setup lang="ts">
import { computed } from "vue";
import { DbrThemeToggle } from "dobruniaui-vue";

const props = defineProps<{ isPaused: boolean }>();

const statusText = computed(() =>
  props.isPaused ? "На паузе" : "Трекинг активен"
);
</script>

<template>
  <header class="app-header dbru-surface">
    <div class="logo">
      <svg
        class="logo-icon"
        width="18"
        height="18"
        viewBox="0 0 18 18"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
        aria-hidden="true"
      >
        <circle cx="9" cy="9" r="7.5" stroke="currentColor" stroke-width="1.5" />
        <path
          d="M9 5.25V9L11.25 11.25"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
      <span class="logo-text dbru-text-lg dbru-text-main">Worka</span>
    </div>

    <div class="right-side">
      <DbrThemeToggle size="sm" shape="rounded" />
      <div
        class="status"
        :class="{ 'status--paused': isPaused }"
        data-testid="status-badge"
      >
        <span class="status__dot" />
        <span class="status__text dbru-text-xs dbru-text-main">{{ statusText }}</span>
      </div>
    </div>
  </header>
</template>

<style scoped>
.app-header {
  height: 48px;
  padding: 0 var(--dbru-space-6);
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid var(--dbru-color-border);
  flex-shrink: 0;
}

.logo {
  display: flex;
  align-items: center;
  gap: var(--dbru-space-2);
  color: var(--dbru-color-text);
}

.logo-text {
  font-weight: var(--dbru-font-weight-semibold);
  letter-spacing: -0.01em;
}

.right-side {
  display: flex;
  align-items: center;
  gap: var(--dbru-space-3);
}

.status {
  display: flex;
  align-items: center;
  gap: var(--dbru-space-2);
  padding: var(--dbru-space-1) var(--dbru-space-3);
  border-radius: var(--dbru-radius-sm);
  background: color-mix(in oklab, var(--dbru-color-primary) 10%, transparent);
}

.status--paused {
  background: color-mix(in oklab, #f59e0b 12%, transparent);
}

.status__dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--dbru-color-primary);
  flex-shrink: 0;
  animation: pulse 2.4s ease-in-out infinite;
}

.status--paused .status__dot {
  background: #f59e0b;
  animation: none;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50%       { opacity: 0.35; }
}
</style>
