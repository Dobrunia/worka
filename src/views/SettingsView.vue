<script setup lang="ts">
import { ref, onMounted } from "vue";
import { DbrCard, DbrCheckbox } from "dobruniaui-vue";
import { invoke } from "@tauri-apps/api/core";
import { useTodayData } from "@/composables/useTodayData";
import type { AppSettings } from "@/composables/useTodayData";

const { loadSummary } = useTodayData();

const paused = ref(false);
const trackWindowTitles = ref(true);
const trackInput = ref(true);
const autostart = ref(false);
const sampleIntervalSeconds = ref(10);
const idleThresholdSeconds = ref(120);

async function loadSettings() {
  try {
    const s: AppSettings = await invoke("get_settings");
    paused.value = s.paused;
    trackWindowTitles.value = s.track_window_titles;
    trackInput.value = s.track_input;
    autostart.value = s.autostart;
    sampleIntervalSeconds.value = s.sample_interval_seconds;
    idleThresholdSeconds.value = s.idle_threshold_seconds;
  } catch (error) {
    console.error("Failed to load settings:", error);
  }
}

async function saveSettings() {
  try {
    await invoke("set_settings", {
      paused: paused.value,
      sampleIntervalSeconds: sampleIntervalSeconds.value,
      idleThresholdSeconds: idleThresholdSeconds.value,
      trackWindowTitles: trackWindowTitles.value,
      trackInput: trackInput.value,
      autostart: autostart.value,
    });
    await loadSummary();
  } catch (error) {
    console.error("Failed to save settings:", error);
  }
}

onMounted(loadSettings);
</script>

<template>
  <div class="settings-view">
    <h2 class="view-title dbru-text-lg dbru-text-main">Настройки</h2>

    <DbrCard class="settings-card">
      <h3 class="section-title dbru-text-base dbru-text-main">Трекинг</h3>
      <div class="settings-group">
        <DbrCheckbox v-model="paused" @change="saveSettings" label="Пауза трекинга" />
        <DbrCheckbox
          v-model="trackWindowTitles"
          @change="saveSettings"
          label="Сохранять заголовки окон"
        />
        <DbrCheckbox
          v-model="trackInput"
          @change="saveSettings"
          label="Отслеживать клавиатуру и мышь"
        />
        <DbrCheckbox
          v-model="autostart"
          @change="saveSettings"
          label="Автозапуск с Windows"
        />
      </div>
    </DbrCard>
  </div>
</template>

<style scoped>
.settings-view {
  display: flex;
  flex-direction: column;
  gap: var(--dbru-space-4);
  max-width: 560px;
}

.view-title {
  margin: 0;
  font-weight: var(--dbru-font-weight-semibold);
}

.settings-card {
  padding: var(--dbru-space-5) var(--dbru-space-6);
  display: flex;
  flex-direction: column;
  gap: var(--dbru-space-4);
}

.section-title {
  margin: 0;
  font-weight: var(--dbru-font-weight-semibold);
}

.settings-group {
  display: flex;
  flex-direction: column;
  gap: var(--dbru-space-4);
}
</style>
