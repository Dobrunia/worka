<script setup lang="ts">
import { computed } from "vue";
import { DbrBadge } from "dobruniaui-vue";

const props = defineProps<{
  isPaused: boolean;
}>();

const badgeText = computed(() =>
  props.isPaused ? "На паузе" : "Трекинг активен"
);
</script>

<template>
  <header class="app-header dbru-surface">
    <div class="header-left">
      <div class="logo">
        <svg
          class="logo-icon"
          viewBox="0 0 24 24"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="2" />
          <path d="M12 6v6l4 2" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
        </svg>
        <h1 class="logo-text dbru-text-lg dbru-text-main">Worka</h1>
      </div>
      <DbrBadge
        :variant="isPaused ? undefined : 'primary'"
        :label="badgeText"
        data-testid="status-badge"
      />
    </div>
  </header>
</template>
