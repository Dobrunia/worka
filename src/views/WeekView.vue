<script setup lang="ts">
import { computed, onMounted } from "vue";
import { DbrCard } from "dobruniaui-vue";
import { Bar } from "vue-chartjs";
import {
  Chart as ChartJS,
  Title,
  Tooltip,
  Legend,
  BarElement,
  CategoryScale,
  LinearScale,
} from "chart.js";
import { useWeekData } from "@/composables/useWeekData";

ChartJS.register(Title, Tooltip, Legend, BarElement, CategoryScale, LinearScale);

const { weekSummary, formatTime, startPolling } = useWeekData();

onMounted(() => {
  startPolling();
});

const hasData = computed(() =>
  weekSummary.value.days.some(
    (d) => d.active_time_seconds > 0 || d.idle_time_seconds > 0
  )
);

const activityChartData = computed(() => ({
  labels: weekSummary.value.days.map((d) => d.day_name),
  datasets: [
    {
      label: "Активное время",
      data: weekSummary.value.days.map((d) =>
        Math.round(d.active_time_seconds / 3600 * 10) / 10
      ),
      backgroundColor: "#22c55e",
      borderRadius: 4,
    },
    {
      label: "Время простоя",
      data: weekSummary.value.days.map((d) =>
        Math.round(d.idle_time_seconds / 3600 * 10) / 10
      ),
      backgroundColor: "#f59e0b",
      borderRadius: 4,
    },
  ],
}));

const activityChartOptions = {
  responsive: true,
  maintainAspectRatio: false,
  plugins: {
    legend: { position: "bottom" as const },
    tooltip: {
      callbacks: {
        label: (ctx: any) => `${ctx.dataset.label}: ${ctx.raw} ч.`,
      },
    },
  },
  scales: {
    y: {
      beginAtZero: true,
      title: { display: true, text: "Часы" },
    },
  },
};

const inputChartData = computed(() => ({
  labels: weekSummary.value.days.map((d) => d.day_name),
  datasets: [
    {
      label: "Нажатия клавиш",
      data: weekSummary.value.days.map((d) =>
        Math.round(d.keyboard_presses / 1000)
      ),
      backgroundColor: "#3b82f6",
      borderRadius: 4,
    },
    {
      label: "Клики мыши",
      data: weekSummary.value.days.map((d) =>
        Math.round(d.mouse_clicks / 100)
      ),
      backgroundColor: "#8b5cf6",
      borderRadius: 4,
    },
  ],
}));

const inputChartOptions = {
  responsive: true,
  maintainAspectRatio: false,
  plugins: {
    legend: { position: "bottom" as const },
    tooltip: {
      callbacks: {
        label: (ctx: any) => {
          const label = ctx.dataset.label;
          const val = ctx.raw;
          return label.includes("клавиш") ? `${label}: ${val}K` : `${label}: ${val * 100}`;
        },
      },
    },
  },
  scales: {
    y: {
      beginAtZero: true,
    },
  },
};

const totalActiveTime = computed(() =>
  weekSummary.value.days.reduce((sum, d) => sum + d.active_time_seconds, 0)
);

const totalKeyboard = computed(() =>
  weekSummary.value.days.reduce((sum, d) => sum + d.keyboard_presses, 0)
);

const totalMouse = computed(() =>
  weekSummary.value.days.reduce((sum, d) => sum + d.mouse_clicks, 0)
);
</script>

<template>
  <div class="week-view">
    <h2 class="view-title dbru-text-lg dbru-text-main">Неделя</h2>

    <template v-if="hasData">
      <div class="kpi-grid">
        <DbrCard class="mini-kpi">
          <span class="dbru-text-xs dbru-text-muted">Всего активного</span>
          <span class="dbru-text-lg dbru-text-main">{{ formatTime(totalActiveTime) }}</span>
        </DbrCard>
        <DbrCard class="mini-kpi">
          <span class="dbru-text-xs dbru-text-muted">Клавиш (тыс.)</span>
          <span class="dbru-text-lg dbru-text-main">{{ Math.round(totalKeyboard / 1000) }}</span>
        </DbrCard>
        <DbrCard class="mini-kpi">
          <span class="dbru-text-xs dbru-text-muted">Кликов (сотни)</span>
          <span class="dbru-text-lg dbru-text-main">{{ Math.round(totalMouse / 100) }}</span>
        </DbrCard>
      </div>

      <DbrCard class="chart-card">
        <h3 class="chart-title dbru-text-sm dbru-text-main">
          Время по дням недели
        </h3>
        <div class="chart-container">
          <Bar :data="activityChartData" :options="activityChartOptions" />
        </div>
      </DbrCard>

      <DbrCard class="chart-card">
        <h3 class="chart-title dbru-text-sm dbru-text-main">
          Ввод по дням недели
        </h3>
        <div class="chart-container">
          <Bar :data="inputChartData" :options="inputChartOptions" />
        </div>
      </DbrCard>
    </template>

    <DbrCard v-else class="empty-card">
      <p class="dbru-text-sm dbru-text-muted">
        Статистика за неделю появится здесь
      </p>
    </DbrCard>
  </div>
</template>

<style scoped>
.week-view {
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
  grid-template-columns: repeat(3, 1fr);
  gap: var(--dbru-space-4);
}

.mini-kpi {
  padding: var(--dbru-space-3) var(--dbru-space-4);
  display: flex;
  flex-direction: column;
  gap: var(--dbru-space-1);
}

.mini-kpi span:last-child {
  font-weight: var(--dbru-font-weight-semibold);
}

.chart-card {
  padding: var(--dbru-space-4) var(--dbru-space-5);
}

.chart-title {
  margin: 0 0 var(--dbru-space-3) 0;
  font-weight: var(--dbru-font-weight-medium);
}

.chart-container {
  height: 250px;
}

.empty-card {
  padding: var(--dbru-space-6);
  text-align: center;
}

.empty-card p {
  margin: 0;
}
</style>
