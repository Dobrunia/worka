<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from "vue";
import { DbrBadge } from "dobruniaui-vue";
import { invoke } from "@tauri-apps/api/core";

const isPaused = ref(false);

async function loadStatus() {
  try {
    const settings: { paused: boolean } = await invoke("get_settings");
    isPaused.value = settings.paused;
  } catch (error) {
    console.error("Failed to load status:", error);
  }
}

const badgeText = computed(() => isPaused.value ? 'На паузе' : 'Трекинг активен');

// Обновляем статус каждые 3 секунды
let pollInterval: ReturnType<typeof setInterval> | null = null;

onMounted(() => {
  loadStatus();
  pollInterval = setInterval(loadStatus, 3000);
});

onUnmounted(() => {
  if (pollInterval) {
    clearInterval(pollInterval);
  }
});
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
      <DbrBadge :variant="isPaused ? undefined : 'primary'" :label="badgeText" />
    </div>
  </header>
</template>