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
  type Plugin,
} from "chart.js";
import KpiCard from "@/components/ui/KpiCard.vue";
import TopAppsList from "@/components/ui/TopAppsList.vue";
import { useAllTimeData } from "@/composables/useAllTimeData";

ChartJS.register(Title, Tooltip, Legend, BarElement, CategoryScale, LinearScale);

const { allTimeSummary, formatTime, startPolling } = useAllTimeData();

onMounted(() => startPolling());

const iconCache = new Map<string, HTMLImageElement>();

function getCachedIcon(src: string): HTMLImageElement {
  const cached = iconCache.get(src);
  if (cached) return cached;

  const image = new Image();
  image.src = src;
  iconCache.set(src, image);
  return image;
}

const appIconAxisPlugin: Plugin<"bar"> = {
  id: "app-icon-axis-plugin",
  afterDraw(chart) {
    const labels = chart.data.labels as string[] | undefined;
    const firstDataset = chart.data.datasets[0] as
      | { appIcons?: Array<string | null | undefined> }
      | undefined;
    const appIcons = firstDataset?.appIcons;
    const xScale = chart.scales.x;
    const yScale = chart.scales.y;

    if (!labels || !appIcons || !xScale || !yScale) return;

    const ctx = chart.ctx;
    const iconSize = 16;
    const yOffset = yScale.bottom + 8;

    ctx.save();
    ctx.textAlign = "center";
    ctx.textBaseline = "top";
    ctx.font = "11px Segoe UI, sans-serif";
    ctx.fillStyle = "#64748b";

    appIcons.forEach((iconDataUrl, index) => {
      const x = xScale.getPixelForTick(index);
      if (x < xScale.left || x > xScale.right) return;

      if (iconDataUrl) {
        const image = getCachedIcon(iconDataUrl);
        if (image.complete && image.naturalWidth > 0) {
          ctx.drawImage(
            image,
            x - iconSize / 2,
            yOffset,
            iconSize,
            iconSize
          );
          return;
        }
        image.onload = () => chart.draw();
      }

      const fallbackLabel = labels[index] ?? "";
      const shortLabel =
        fallbackLabel.length > 8
          ? `${fallbackLabel.slice(0, 8)}…`
          : fallbackLabel;
      ctx.fillText(shortLabel, x, yOffset);
    });

    ctx.restore();
  },
};

const topApps = computed(() =>
  allTimeSummary.value.top_apps.map((a) => ({
    name: a.name,
    iconDataUrl: a.icon_data_url ?? null,
    timeSeconds: a.time_seconds,
    percentage: a.percentage,
  }))
);

const hasData = computed(
  () =>
    allTimeSummary.value.active_time_seconds > 0 ||
    allTimeSummary.value.idle_time_seconds > 0
);

const totalTrackedSeconds = computed(
  () =>
    allTimeSummary.value.active_time_seconds +
    allTimeSummary.value.idle_time_seconds
);

const activeSharePercent = computed(() => {
  if (totalTrackedSeconds.value <= 0) return 0;
  return Math.round(
    (allTimeSummary.value.active_time_seconds / totalTrackedSeconds.value) * 100
  );
});

const avgActivePerDay = computed(() => {
  if (allTimeSummary.value.days_tracked <= 0) return "0м";
  const perDay = Math.round(
    allTimeSummary.value.active_time_seconds / allTimeSummary.value.days_tracked
  );
  return formatTime(perDay);
});

const topAppsForChart = computed(() => allTimeSummary.value.top_apps.slice(0, 7));

const appsChartData = computed(() => ({
  labels: topAppsForChart.value.map((a) => a.name),
  datasets: [
    {
      label: "Время",
      data: topAppsForChart.value.map((a) =>
        Math.round(a.time_seconds / 3600 * 10) / 10
      ),
      appIcons: topAppsForChart.value.map((a) => a.icon_data_url ?? null),
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
        label: (ctx: any) => `${ctx.label}: ${ctx.raw} ч.`,
      },
    },
  },
  layout: {
    padding: {
      bottom: 26,
    },
  },
  scales: {
    x: {
      ticks: {
        display: false,
      },
    },
    y: {
      beginAtZero: true,
      title: { display: true, text: "Часы" },
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

      <DbrCard class="chart-card">
        <h3 class="chart-title dbru-text-sm dbru-text-main">
          Топ приложений
        </h3>
        <div class="chart-container">
          <Bar
            v-if="appsChartData.labels.length > 0"
            :data="appsChartData"
            :options="appsChartOptions"
            :plugins="[appIconAxisPlugin]"
          />
          <p v-else class="dbru-text-sm dbru-text-muted">Нет данных</p>
        </div>
      </DbrCard>

      <div class="insights-row">
        <DbrCard class="insight-card">
          <span class="dbru-text-xs dbru-text-muted">Доля активного времени</span>
          <span class="dbru-text-lg dbru-text-main">{{ activeSharePercent }}%</span>
        </DbrCard>
        <DbrCard class="insight-card">
          <span class="dbru-text-xs dbru-text-muted">Среднее активное в день</span>
          <span class="dbru-text-lg dbru-text-main">{{ avgActivePerDay }}</span>
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

.insights-row {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: var(--dbru-space-4);
}

.chart-card {
  padding: var(--dbru-space-4) var(--dbru-space-5);
}

.insight-card {
  padding: var(--dbru-space-3) var(--dbru-space-4);
  display: flex;
  flex-direction: column;
  gap: var(--dbru-space-1);
}

.insight-card span:last-child {
  font-weight: var(--dbru-font-weight-semibold);
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

  .insights-row {
    grid-template-columns: 1fr;
  }
}
</style>
