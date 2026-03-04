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

    <p v-if="!apps || apps.length === 0" class="dbru-text-sm dbru-text-muted empty">
      Нет данных
    </p>

    <div v-else class="apps-container">
      <div v-for="app in apps" :key="app.name" class="app-item">
        <div class="app-info">
          <span class="app-name dbru-text-sm dbru-text-main">{{ app.name }}</span>
          <span class="dbru-text-xs dbru-text-muted">{{ formatTime(app.timeSeconds) }}</span>
        </div>
        <div class="progress-bar">
          <div class="progress-fill" :style="{ width: `${app.percentage}%` }" />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.top-apps-list {
  display: flex;
  flex-direction: column;
  gap: var(--dbru-space-4);
}

.list-title {
  font-weight: var(--dbru-font-weight-semibold);
  margin: 0;
}

.empty {
  margin: 0;
  text-align: center;
  padding: var(--dbru-space-4) 0;
}

.apps-container {
  display: flex;
  flex-direction: column;
  gap: var(--dbru-space-3);
}

.app-item {
  display: flex;
  flex-direction: column;
  gap: var(--dbru-space-1);
}

.app-info {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
}

.app-name {
  font-weight: 500;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.progress-bar {
  height: 4px;
  background: var(--dbru-color-border);
  border-radius: 2px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--dbru-color-primary);
  border-radius: 2px;
  transition: width var(--dbru-duration-base) var(--dbru-ease-standard);
}
</style>
