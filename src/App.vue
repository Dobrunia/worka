<script setup lang="ts">
import { ref, computed } from "vue";
import AppHeader from "@/components/layout/AppHeader.vue";
import AppNavigation from "@/components/layout/AppNavigation.vue";
import AppFooter from "@/components/layout/AppFooter.vue";
import TodayView from "@/views/TodayView.vue";
import WeekView from "@/views/WeekView.vue";
import TimelineView from "@/views/TimelineView.vue";
import SettingsView from "@/views/SettingsView.vue";

const activeTab = ref("today");

const tabs = [
  { id: "today", label: "Сегодня" },
  { id: "week", label: "Неделя" },
  { id: "timeline", label: "Таймлайн" },
  { id: "settings", label: "Настройки" },
];

const isTracking = computed(() => activeTab.value !== "settings");

const currentView = computed(() => {
  switch (activeTab.value) {
    case "today":
      return TodayView;
    case "week":
      return WeekView;
    case "timeline":
      return TimelineView;
    case "settings":
      return SettingsView;
    default:
      return TodayView;
  }
});
</script>

<template>
  <div class="app-container dbru-bg">
    <AppHeader :is-tracking="isTracking" />
    
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
