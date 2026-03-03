<script setup lang="ts">
import { computed } from "vue";
import { DbrCard } from "dobruniaui-vue";
import KpiCard from "@/components/ui/KpiCard.vue";
import TopAppsList from "@/components/ui/TopAppsList.vue";
import { useTodayData } from "@/composables/useTodayData";

const { summary, isPaused, formatTime } = useTodayData();

// Map backend snake_case to component camelCase.
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
  <DbrCard class="today-view">
    <header class="view-header">
      <h2 class="dbru-text-lg dbru-text-main">Сегодня</h2>
    </header>

    <section v-if="!hasData" class="placeholder-section">
      <p class="placeholder-text dbru-text-sm dbru-text-muted">
        {{ isPaused ? "Трекинг на паузе" : "Данные появятся после начала трекинга" }}
      </p>
    </section>

    <template v-else>
      <section class="kpi-grid">
        <KpiCard label="Активное время" :value="formatTime(summary.active_time_seconds)" />
        <KpiCard label="Время простоя" :value="formatTime(summary.idle_time_seconds)" />
        <KpiCard label="Нажатия клавиш" :value="summary.keyboard_presses.toString()" />
        <KpiCard label="Клики мыши" :value="summary.mouse_clicks.toString()" />
      </section>

      <TopAppsList :apps="topApps" />
    </template>
  </DbrCard>
</template>
