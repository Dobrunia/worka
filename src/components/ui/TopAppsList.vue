<script setup lang="ts">
import { formatTime } from "@/utils/time";

export interface AppUsage {
  name: string;
  timeSeconds: number;
  percentage: number;
}

defineProps<{
  apps?: AppUsage[];
}>();
</script>

<template>
  <div class="top-apps-list">
    <h3 class="list-title dbru-text-base dbru-text-main">Топ приложений</h3>
    
    <div v-if="!apps || apps.length === 0" class="empty-state">
      <p class="dbru-text-sm dbru-text-muted">Нет данных</p>
    </div>
    
    <div v-else class="apps-container">
      <div
        v-for="app in apps"
        :key="app.name"
        class="app-item"
      >
        <div class="app-info">
          <span class="app-name dbru-text-sm dbru-text-main">{{ app.name }}</span>
          <span class="app-time dbru-text-xs dbru-text-muted">{{ formatTime(app.timeSeconds) }}</span>
        </div>
        <div class="progress-bar">
          <div
            class="progress-fill"
            :style="{ width: `${app.percentage}%` }"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.top-apps-list {
  margin-top: var(--dbru-space-4);
}

.list-title {
  font-weight: 600;
  margin: 0 0 var(--dbru-space-4) 0;
}

.empty-state {
  text-align: center;
  padding: var(--dbru-space-6);
}

.apps-container {
  display: flex;
  flex-direction: column;
  gap: var(--dbru-space-4);
}

.app-item {
  display: flex;
  flex-direction: column;
  gap: var(--dbru-space-2);
}

.app-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.app-name {
  font-weight: 500;
}

.app-time {
  font-size: var(--dbru-font-size-xs);
}

.progress-bar {
  height: 6px;
  background: var(--dbru-color-border);
  border-radius: var(--dbru-radius-sm);
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--dbru-color-primary);
  border-radius: var(--dbru-radius-sm);
  transition: width var(--dbru-duration-base) var(--dbru-ease-standard);
}
</style>
