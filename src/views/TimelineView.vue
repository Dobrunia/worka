<script setup lang="ts">
import { computed, onMounted, watch } from "vue";
import { DbrCard, DbrButton } from "dobruniaui-vue";
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
import { useTimelineData } from "@/composables/useTimelineData";

ChartJS.register(Title, Tooltip, Legend, BarElement, CategoryScale, LinearScale);

const { selectedDate, timeline, isLoading, loadTimeline, formatTime } = useTimelineData();

onMounted(() => {
  loadTimeline(selectedDate.value);
});

watch(selectedDate, (date) => {
  loadTimeline(date);
});

function shiftDate(days: number) {
  const date = new Date(`${selectedDate.value}T00:00:00`);
  date.setDate(date.getDate() + days);
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  selectedDate.value = `${year}-${month}-${day}`;
}

const hasData = computed(() => timeline.value.has_data);

const hoursWithDetails = computed(() =>
  timeline.value.hours.filter(
    (hour) =>
      hour.active_time_seconds > 0 ||
      hour.idle_time_seconds > 0 ||
      hour.keyboard_presses > 0 ||
      hour.mouse_clicks > 0
  )
);

const hourlyChartData = computed(() => ({
  labels: timeline.value.hours.map((hour) => hour.hour),
  datasets: [
    {
      label: "Активное",
      data: timeline.value.hours.map((hour) =>
        Math.round((hour.active_time_seconds / 3600) * 10) / 10
      ),
      backgroundColor: "#22c55e",
      borderRadius: 4,
    },
    {
      label: "Простой",
      data: timeline.value.hours.map((hour) =>
        Math.round((hour.idle_time_seconds / 3600) * 10) / 10
      ),
      backgroundColor: "#f59e0b",
      borderRadius: 4,
    },
  ],
}));

const hourlyChartOptions = {
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
    x: {
      stacked: true,
      ticks: {
        autoSkip: true,
        maxTicksLimit: 12,
      },
    },
    y: {
      stacked: true,
      beginAtZero: true,
      title: { display: true, text: "Часы" },
    },
  },
};
</script>

<template>
  <div class="timeline-view">
    <div class="timeline-header">
      <h2 class="view-title dbru-text-lg dbru-text-main">Таймлайн</h2>
      <div class="date-controls">
        <DbrButton size="sm" variant="ghost" @click="shiftDate(-1)">←</DbrButton>
        <input v-model="selectedDate" type="date" class="date-input dbru-text-sm dbru-text-main" />
        <DbrButton size="sm" variant="ghost" @click="shiftDate(1)">→</DbrButton>
      </div>
    </div>

    <DbrCard v-if="isLoading" class="placeholder-card">
      <p class="dbru-text-sm dbru-text-muted">Загрузка таймлайна...</p>
    </DbrCard>

    <template v-else-if="hasData">
      <DbrCard class="chart-card">
        <h3 class="chart-title dbru-text-sm dbru-text-main">Активность по часам</h3>
        <div class="chart-container">
          <Bar :data="hourlyChartData" :options="hourlyChartOptions" />
        </div>
      </DbrCard>

      <DbrCard class="details-card">
        <h3 class="chart-title dbru-text-sm dbru-text-main">Что делали по часам</h3>
        <div class="hours-list">
          <div
            v-for="hour in hoursWithDetails"
            :key="hour.hour_index"
            class="hour-row"
          >
            <div class="hour-meta">
              <span class="hour-label dbru-text-sm dbru-text-main">{{ hour.hour }}</span>
              <span class="dbru-text-xs dbru-text-muted">
                Активно: {{ formatTime(hour.active_time_seconds) }} · Простой:
                {{ formatTime(hour.idle_time_seconds) }}
              </span>
              <span class="dbru-text-xs dbru-text-muted">
                ⌨ {{ hour.keyboard_presses }} · 🖱 {{ hour.mouse_clicks }}
              </span>
            </div>

            <div class="hour-apps">
              <template v-if="hour.apps.length > 0">
                <div
                  v-for="app in hour.apps.slice(0, 10)"
                  :key="`${hour.hour_index}-${app.name}`"
                  class="app-chip"
                  :title="`${app.name} • ${formatTime(app.time_seconds)}`"
                >
                  <img
                    v-if="app.icon_data_url"
                    :src="app.icon_data_url"
                    :alt="app.name"
                    class="app-icon"
                  />
                  <span v-else class="app-fallback dbru-text-xs dbru-text-main">
                    {{ app.name }}
                  </span>
                </div>
              </template>
              <span v-else class="dbru-text-xs dbru-text-muted">Нет активности</span>
            </div>
          </div>
        </div>
      </DbrCard>
    </template>

    <DbrCard v-else class="placeholder-card">
      <p class="dbru-text-sm dbru-text-muted">
        Для {{ selectedDate }} нет данных. Запустите трекинг и выберите день с активностью.
      </p>
    </DbrCard>
  </div>
</template>

<style scoped>
.timeline-view {
  display: flex;
  flex-direction: column;
  gap: var(--dbru-space-4);
  max-width: 1060px;
}

.timeline-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: var(--dbru-space-3);
  flex-wrap: wrap;
}

.view-title {
  margin: 0;
  font-weight: var(--dbru-font-weight-semibold);
}

.date-controls {
  display: flex;
  align-items: center;
  gap: var(--dbru-space-2);
}

.date-input {
  border: 1px solid var(--dbru-color-border);
  border-radius: var(--dbru-radius-sm);
  padding: var(--dbru-space-2) var(--dbru-space-3);
  background: var(--dbru-color-surface);
}

.chart-card,
.details-card {
  padding: var(--dbru-space-4) var(--dbru-space-5);
}

.chart-title {
  margin: 0 0 var(--dbru-space-3) 0;
  font-weight: var(--dbru-font-weight-medium);
}

.chart-container {
  height: 280px;
}

.hours-list {
  display: flex;
  flex-direction: column;
  gap: var(--dbru-space-3);
}

.hour-row {
  border: 1px solid var(--dbru-color-border);
  border-radius: var(--dbru-radius-md);
  padding: var(--dbru-space-3);
  display: flex;
  justify-content: space-between;
  gap: var(--dbru-space-3);
  flex-wrap: wrap;
}

.hour-meta {
  display: flex;
  flex-direction: column;
  gap: var(--dbru-space-1);
  min-width: 220px;
}

.hour-label {
  font-weight: var(--dbru-font-weight-semibold);
}

.hour-apps {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: var(--dbru-space-2);
  max-width: 600px;
}

.app-chip {
  min-height: 24px;
  min-width: 24px;
  border: 1px solid var(--dbru-color-border);
  border-radius: var(--dbru-radius-sm);
  padding: 2px;
  background: var(--dbru-color-surface-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
}

.app-icon {
  width: 20px;
  height: 20px;
  object-fit: contain;
}

.app-fallback {
  padding: 0 var(--dbru-space-1);
  white-space: nowrap;
  max-width: 150px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.placeholder-card {
  padding: var(--dbru-space-6);
  text-align: center;
}

.placeholder-card p {
  margin: 0;
}

@media (max-width: 800px) {
  .chart-container {
    height: 240px;
  }
}
</style>
