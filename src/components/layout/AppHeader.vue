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
