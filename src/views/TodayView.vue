<script setup lang="ts">
import { computed } from "vue";
import { DbrCard } from "dobruniaui-vue";
import { Doughnut } from "vue-chartjs";
import {
  Chart as ChartJS,
  Title,
  Tooltip,
  Legend,
  ArcElement,
} from "chart.js";
import KpiCard from "@/components/ui/KpiCard.vue";
import TopAppsList from "@/components/ui/TopAppsList.vue";
import { useTodayData } from "@/composables/useTodayData";

ChartJS.register(
  Title,
  Tooltip,
  Legend,
  ArcElement
);

const { summary, isPaused, formatTime } = useTodayData();

const topApps = computed(() =>
  summary.value.top_apps.map((a) => ({
    name: a.name,
    iconDataUrl: a.icon_data_url ?? null,
    timeSeconds: a.time_seconds,
    percentage: a.percentage,
  }))
);

const hasData = computed(
  () =>
    summary.value.active_time_seconds > 0 ||
    summary.value.idle_time_seconds > 0
);

// Doughnut chart for active vs idle
const activityChartData = computed(() => ({
  labels: ["Активное", "Простой"],
  datasets: [
    {
      data: [
        summary.value.active_time_seconds,
        summary.value.idle_time_seconds,
      ],
      backgroundColor: ["#22c55e", "#f59e0b"],
      borderWidth: 0,
    },
  ],
}));

const doughnutOptions = {
  responsive: true,
  maintainAspectRatio: false,
  cutout: "60%",
  plugins: {
    legend: { position: "bottom" as const },
    tooltip: {
      callbacks: {
        label: (ctx: any) => {
          const total =
            summary.value.active_time_seconds +
            summary.value.idle_time_seconds;
          const pct =
            total > 0 ? Math.round((ctx.raw / total) * 100) : 0;
          return `${ctx.label}: ${pct}%`;
        },
      },
    },
  },
};
</script>

<template>
  <div class="today-view">
    <h2 class="view-title dbru-text-lg dbru-text-main">Сегодня</h2>

    <DbrCard v-if="!hasData" class="empty-card">
      <p class="dbru-text-sm dbru-text-muted">
        {{
          isPaused ? "Трекинг на паузе" : "Данные появятся после начала трекинга"
        }}
      </p>
    </DbrCard>

    <template v-else>
      <div class="kpi-grid">
        <KpiCard
          label="Активное время"
          :value="formatTime(summary.active_time_seconds)"
          icon="⏱️"
        />
        <KpiCard
          label="Время простоя"
          :value="formatTime(summary.idle_time_seconds)"
          icon="☕"
        />
        <KpiCard
          label="Нажатия клавиш"
          :value="summary.keyboard_presses.toString()"
          icon="⌨️"
        />
        <KpiCard
          label="Клики мыши"
          :value="summary.mouse_clicks.toString()"
          icon="🖱️"
        />
      </div>

      <DbrCard class="chart-card">
        <h3 class="chart-title dbru-text-sm dbru-text-main">
          Активность за день
        </h3>
        <div class="chart-container doughnut-container">
          <Doughnut :data="activityChartData" :options="doughnutOptions" />
        </div>
      </DbrCard>

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

.chart-card {
  padding: var(--dbru-space-4) var(--dbru-space-5);
}

.chart-title {
  margin: 0 0 var(--dbru-space-3) 0;
  font-weight: var(--dbru-font-weight-medium);
}

.chart-container {
  height: 200px;
}

.doughnut-container {
  display: flex;
  align-items: center;
  justify-content: center;
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

@media (max-width: 800px) {
  .kpi-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}
</style>
