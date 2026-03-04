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
import KpiCard from "@/components/ui/KpiCard.vue";
import TopAppsList from "@/components/ui/TopAppsList.vue";
import { useAllTimeData } from "@/composables/useAllTimeData";

ChartJS.register(Title, Tooltip, Legend, BarElement, CategoryScale, LinearScale);

const { allTimeSummary, formatTime, startPolling } = useAllTimeData();

onMounted(() => startPolling());

const topApps = computed(() =>
  allTimeSummary.value.top_apps.map((a) => ({
    name: a.name,
    timeSeconds: a.time_seconds,
    percentage: a.percentage,
  }))
);

const hasData = computed(
  () =>
    allTimeSummary.value.active_time_seconds > 0 ||
    allTimeSummary.value.idle_time_seconds > 0
);

const appsChartData = computed(() => ({
  labels: allTimeSummary.value.top_apps.slice(0, 7).map((a) => a.name),
  datasets: [
    {
      label: "Время",
      data: allTimeSummary.value.top_apps.slice(0, 7).map((a) =>
        Math.round(a.time_seconds / 3600 * 10) / 10
      ),
      backgroundColor: "#3b82f6",
      borderRadius: 4,
    },
  ],
}));

const appsChartOptions = {
  responsive: true,
  maintainAspectRatio: false,
  plugins: {
    legend: { display: false },
    tooltip: {
      callbacks: {
        label: (ctx: any) => `${ctx.raw} ч.`,
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

const timeDistributionData = computed(() => ({
  labels: ["Активное", "Простой"],
  datasets: [
    {
      data: [
        allTimeSummary.value.active_time_seconds,
        allTimeSummary.value.idle_time_seconds,
      ],
      backgroundColor: ["#22c55e", "#f59e0b"],
    },
  ],
}));

const pieOptions = {
  responsive: true,
  maintainAspectRatio: false,
  plugins: {
    legend: { position: "bottom" as const },
    tooltip: {
      callbacks: {
        label: (ctx: any) => {
          const total =
            allTimeSummary.value.active_time_seconds +
            allTimeSummary.value.idle_time_seconds;
          const pct = total > 0 ? Math.round((ctx.raw / total) * 100) : 0;
          return `${ctx.label}: ${pct}%`;
        },
      },
    },
  },
};
</script>

<template>
  <div class="alltime-view">
    <h2 class="view-title dbru-text-lg dbru-text-main">За всё время</h2>

    <template v-if="hasData">
      <div class="kpi-grid">
        <KpiCard
          label="Активное время"
          :value="formatTime(allTimeSummary.active_time_seconds)"
          icon="⏱️"
        />
        <KpiCard
          label="Время простоя"
          :value="formatTime(allTimeSummary.idle_time_seconds)"
          icon="☕"
        />
        <KpiCard
          label="Нажатия клавиш"
          :value="allTimeSummary.keyboard_presses.toLocaleString()"
          icon="⌨️"
        />
        <KpiCard
          label="Клики мыши"
          :value="allTimeSummary.mouse_clicks.toLocaleString()"
          icon="🖱️"
        />
        <KpiCard
          label="Дней отслежено"
          :value="allTimeSummary.days_tracked.toString()"
          icon="📅"
        />
      </div>

      <div class="charts-row">
        <DbrCard class="chart-card">
          <h3 class="chart-title dbru-text-sm dbru-text-main">
            Топ приложений
          </h3>
          <div class="chart-container">
            <Bar
              v-if="appsChartData.labels.length > 0"
              :data="appsChartData"
              :options="appsChartOptions"
            />
            <p v-else class="dbru-text-sm dbru-text-muted">Нет данных</p>
          </div>
        </DbrCard>

        <DbrCard class="chart-card">
          <h3 class="chart-title dbru-text-sm dbru-text-main">
            Активность / Простой
          </h3>
          <div class="chart-container">
            <Bar
              :data="timeDistributionData"
              :options="{
                ...pieOptions,
                indexAxis: 'y' as const,
              }"
            />
          </div>
        </DbrCard>
      </div>

      <DbrCard class="apps-card">
        <TopAppsList :apps="topApps" />
      </DbrCard>
    </template>

    <DbrCard v-else class="empty-card">
      <p class="dbru-text-sm dbru-text-muted">
        Данные появятся после начала трекинга
      </p>
    </DbrCard>
  </div>
</template>

<style scoped>
.alltime-view {
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
  grid-template-columns: repeat(5, 1fr);
  gap: var(--dbru-space-4);
}

.charts-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
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
  height: 220px;
}

.apps-card {
  padding: var(--dbru-space-4) var(--dbru-space-5);
}

.empty-card {
  padding: var(--dbru-space-6);
  text-align: center;
}

.empty-card p {
  margin: 0;
}

@media (max-width: 800px) {
  .kpi-grid {
    grid-template-columns: repeat(3, 1fr);
  }

  .charts-row {
    grid-template-columns: 1fr;
  }
}
</style>
