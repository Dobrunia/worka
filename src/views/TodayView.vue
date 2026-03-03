<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import { DbrBadge, DbrCard } from 'dobruniaui-vue';
import KpiCard from '@/components/ui/KpiCard.vue';
import { useTodayData } from '@/composables/useTodayData';

const { summary, isPaused, loadSummary, loadSettings, formatTime } = useTodayData();

let pollInterval: ReturnType<typeof setInterval> | null = null;

onMounted(async () => {
  await loadSettings();
  await loadSummary();
  // Опрос каждые 5 секунд для обновления данных
  pollInterval = setInterval(loadSummary, 5000);
});

onUnmounted(() => {
  if (pollInterval) {
    clearInterval(pollInterval);
  }
});
</script>

<template>
  <DbrCard class="today-view">
    <header class="view-header">
      <h2 class="dbru-text-lg dbru-text-main">Сегодня</h2>
      <DbrBadge :variant="isPaused ? undefined : 'primary'" :label="isPaused ? 'На паузе' : 'Трекинг активен'" />
    </header>

    <section class="kpi-grid">
      <KpiCard label="Активное время" :value="formatTime(summary.active_time_seconds)" />
      <KpiCard label="Время простоя" :value="formatTime(summary.idle_time_seconds)" />
      <KpiCard label="Нажатия клавиш" :value="summary.keyboard_presses.toString()" />
      <KpiCard label="Клики мыши" :value="summary.mouse_clicks.toString()" />
    </section>

    <section v-if="summary.active_time_seconds === 0 && summary.idle_time_seconds === 0" class="placeholder-section">
      <p class="placeholder-text dbru-text-sm dbru-text-muted">
        Данные появятся после начала трекинга
      </p>
    </section>
  </DbrCard>
</template>