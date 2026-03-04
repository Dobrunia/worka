<script setup lang="ts">
import { computed } from "vue";
import { DbrCard } from "dobruniaui-vue";
import KpiCard from "@/components/ui/KpiCard.vue";
import TopAppsList from "@/components/ui/TopAppsList.vue";
import { useTodayData } from "@/composables/useTodayData";

const { summary, isPaused, formatTime } = useTodayData();

const topApps = computed(() =>
  summary.value.top_apps.map((a) => ({
    name: a.name,
    timeSeconds: a.time_seconds,
    percentage: a.percentage,
  }))
);

const hasData = computed(
  () =>
    summary.value.active_time_seconds > 0 ||
    summary.value.idle_time_seconds > 0
);
</script>

<template>
  <div class="today-view">
    <h2 class="view-title dbru-text-lg dbru-text-main">Сегодня</h2>

    <DbrCard v-if="!hasData" class="empty-card">
      <p class="dbru-text-sm dbru-text-muted">
        {{ isPaused ? "Трекинг на паузе" : "Данные появятся после начала трекинга" }}
      </p>
    </DbrCard>

    <template v-else>
      <div class="kpi-grid">
        <KpiCard label="Активное время" :value="formatTime(summary.active_time_seconds)" />
        <KpiCard label="Время простоя" :value="formatTime(summary.idle_time_seconds)" />
        <KpiCard label="Нажатия клавиш" :value="summary.keyboard_presses.toString()" />
        <KpiCard label="Клики мыши" :value="summary.mouse_clicks.toString()" />
      </div>

      <DbrCard class="apps-card">
        <TopAppsList :apps="topApps" />
      </DbrCard>
    </template>
  </div>
</template>

<style scoped>
.today-view {
  display: flex;
  flex-direction: column;
  gap: var(--dbru-space-4);
  max-width: 960px;
}

.view-title {
  margin: 0;
  font-weight: var(--dbru-font-weight-semibold);
}

.kpi-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: var(--dbru-space-4);
}

.empty-card {
  padding: var(--dbru-space-6);
  text-align: center;
}

.empty-card p {
  margin: 0;
}

.apps-card {
  padding: var(--dbru-space-4) var(--dbru-space-5);
}
</style>
