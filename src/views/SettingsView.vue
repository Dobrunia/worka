<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { DbrCard, DbrCheckbox } from 'dobruniaui-vue';
import { invoke } from '@tauri-apps/api/core';
import { useTodayData } from '@/composables/useTodayData';
import type { AppSettings } from '@/composables/useTodayData';

const { loadSummary } = useTodayData();

const paused = ref(false);
const trackWindowTitles = ref(true);
const trackInput = ref(true);
const autostart = ref(false);
const sampleIntervalSeconds = ref(10);
const idleThresholdSeconds = ref(120);

async function loadSettings() {
  try {
    const s: AppSettings = await invoke('get_settings');
    paused.value = s.paused;
    trackWindowTitles.value = s.track_window_titles;
    trackInput.value = s.track_input;
    autostart.value = s.autostart;
    sampleIntervalSeconds.value = s.sample_interval_seconds;
    idleThresholdSeconds.value = s.idle_threshold_seconds;
  } catch (error) {
    console.error('Failed to load settings:', error);
  }
}

async function saveSettings() {
  try {
    await invoke('set_settings', {
      paused: paused.value,
      sampleIntervalSeconds: sampleIntervalSeconds.value,
      idleThresholdSeconds: idleThresholdSeconds.value,
      trackWindowTitles: trackWindowTitles.value,
      trackInput: trackInput.value,
      autostart: autostart.value,
    });
    // Refresh shared state so AppHeader badge updates immediately.
    await loadSummary();
  } catch (error) {
    console.error('Failed to save settings:', error);
  }
}

onMounted(loadSettings);
</script>

<template>
  <DbrCard>
    <header>
      <h2 class="dbru-text-lg dbru-text-main">Настройки</h2>
    </header>

    <section>
      <DbrCheckbox v-model="paused" @change="saveSettings" label="Пауза трекинга" />
      <DbrCheckbox
        v-model="trackWindowTitles"
        @change="saveSettings"
        label="Трекинг заголовков окон"
      />
      <DbrCheckbox v-model="trackInput" @change="saveSettings" label="Трекинг ввода" />
      <DbrCheckbox v-model="autostart" @change="saveSettings" label="Автозапуск" />
    </section>
  </DbrCard>
</template>
