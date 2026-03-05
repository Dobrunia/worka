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

const totalActiveTime = computed(() =>
  weekSummary.value.days.reduce((sum, d) => sum + d.active_time_seconds, 0)
);

const totalKeyboard = computed(() =>
  weekSummary.value.days.reduce((sum, d) => sum + d.keyboard_presses, 0)
);

const totalMouse = computed(() =>
  weekSummary.value.days.reduce((sum, d) => sum + d.mouse_clicks, 0)
);

const dailyBreakdown = computed(() =>
  [...weekSummary.value.days]
    .map((day) => {
      const total = day.active_time_seconds + day.idle_time_seconds;
      return {
        ...day,
        focus_percent: total > 0 ? Math.round((day.active_time_seconds / total) * 100) : 0,
      };
    })
    .sort((a, b) => b.active_time_seconds - a.active_time_seconds)
);

const bestDay = computed(() => dailyBreakdown.value[0] ?? null);
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

      <DbrCard class="insights-card">
        <h3 class="chart-title dbru-text-sm dbru-text-main">
          Ключевые дни
        </h3>
        <p v-if="bestDay" class="dbru-text-sm dbru-text-main best-day">
          Лучший день: <strong>{{ bestDay.day_name }}</strong> ·
          {{ formatTime(bestDay.active_time_seconds) }} активного времени
        </p>
        <div class="day-list">
          <div
            v-for="day in dailyBreakdown"
            :key="day.date"
            class="day-row"
          >
            <span class="dbru-text-sm dbru-text-main">{{ day.day_name }}</span>
            <span class="dbru-text-xs dbru-text-muted">{{ day.focus_percent }}% фокуса</span>
            <span class="dbru-text-xs dbru-text-muted">{{ formatTime(day.active_time_seconds) }}</span>
          </div>
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

.insights-card {
  padding: var(--dbru-space-4) var(--dbru-space-5);
  display: flex;
  flex-direction: column;
  gap: var(--dbru-space-3);
}

.chart-title {
  margin: 0 0 var(--dbru-space-3) 0;
  font-weight: var(--dbru-font-weight-medium);
}

.chart-container {
  height: 250px;
}

.best-day {
  margin: 0;
}

.day-list {
  display: flex;
  flex-direction: column;
  gap: var(--dbru-space-2);
}

.day-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: var(--dbru-space-2);
  border: 1px solid var(--dbru-color-border);
  border-radius: var(--dbru-radius-sm);
  padding: var(--dbru-space-2) var(--dbru-space-3);
}

.empty-card {
  padding: var(--dbru-space-6);
  text-align: center;
}

.empty-card p {
  margin: 0;
}
</style>
