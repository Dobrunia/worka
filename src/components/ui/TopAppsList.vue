<script setup lang="ts">
import { ref } from "vue";
import { formatTime } from "@/utils/time";

export interface AppUsage {
  name: string;
  iconDataUrl?: string | null;
  timeSeconds: number;
  percentage: number;
}

defineProps<{
  apps?: AppUsage[];
}>();

const failedIcons = ref<Record<string, boolean>>({});

function markIconFailed(appName: string) {
  failedIcons.value[appName] = true;
}

function shouldShowIcon(app: AppUsage): boolean {
  return Boolean(app.iconDataUrl) && !failedIcons.value[app.name];
}
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
          <div class="app-identity">
            <div
              v-if="shouldShowIcon(app)"
              class="app-icon-wrap"
              :title="app.name"
              :aria-label="app.name"
            >
              <img
                :src="app.iconDataUrl ?? undefined"
                :alt="app.name"
                class="app-icon"
                @error="markIconFailed(app.name)"
              />
            </div>
            <span v-else class="app-name dbru-text-sm dbru-text-main">{{ app.name }}</span>
          </div>
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
  align-items: center;
}

.app-identity {
  display: flex;
  align-items: center;
  min-width: 0;
}

.app-icon-wrap {
  width: 20px;
  height: 20px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--dbru-color-border);
  background: var(--dbru-color-surface-secondary);
}

.app-icon {
  width: 16px;
  height: 16px;
  object-fit: contain;
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
