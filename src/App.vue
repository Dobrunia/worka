<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import AppHeader from "@/components/layout/AppHeader.vue";
import AppNavigation from "@/components/layout/AppNavigation.vue";
import AppFooter from "@/components/layout/AppFooter.vue";
import TodayView from "@/views/TodayView.vue";
import WeekView from "@/views/WeekView.vue";
import TimelineView from "@/views/TimelineView.vue";
import SettingsView from "@/views/SettingsView.vue";
import { useTodayData } from "@/composables/useTodayData";

const activeTab = ref("today");

const tabs = [
  { id: "today", label: "Сегодня" },
  { id: "week", label: "Неделя" },
  { id: "timeline", label: "Таймлайн" },
  { id: "settings", label: "Настройки" },
];

const currentView = computed(() => {
  switch (activeTab.value) {
    case "today": return TodayView;
    case "week": return WeekView;
    case "timeline": return TimelineView;
    case "settings": return SettingsView;
    default: return TodayView;
  }
});

// Polling lifecycle is owned here so it runs regardless of active tab.
const { isPaused, startPolling, stopPolling } = useTodayData();

onMounted(() => startPolling());
onUnmounted(() => stopPolling());
</script>

<template>
  <div class="app-container dbru-bg">
    <AppHeader :is-paused="isPaused" />

    <AppNavigation
      v-model:active-tab="activeTab"
      :tabs="tabs"
    />

    <main class="content dbru-surface">
      <component :is="currentView" />
    </main>

    <AppFooter />
  </div>
</template>

<style>
html,
body,
#app {
  margin: 0;
  padding: 0;
  height: 100%;
  overflow: hidden;
}
</style>

<style scoped>
.app-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
}

.content {
  flex: 1;
  overflow-y: auto;
  padding: var(--dbru-space-6);
}
</style>
