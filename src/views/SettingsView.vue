<script setup lang="ts">
import { ref, onMounted } from "vue";
import { DbrCard, DbrCheckbox } from "dobruniaui-vue";
import { invoke } from "@tauri-apps/api/core";

const paused = ref(false);
const trackWindowTitles = ref(true);
const trackInput = ref(true);
const autostart = ref(false);

async function saveSettings() {
  try {
    await invoke("set_settings", {
      paused,
      sampleIntervalSeconds: 10,
      idleThresholdSeconds: 120,
      trackWindowTitles,
      trackInput,
      autostart,
    });
  } catch (error) {
    console.error("Failed to save settings:", error);
  }
}

async function loadSettings() {
  try {
    const settings: {
      paused: boolean;
      track_window_titles: boolean;
      track_input: boolean;
      autostart: boolean;
    } = await invoke("get_settings");
    paused.value = settings.paused;
    trackWindowTitles.value = settings.track_window_titles;
    trackInput.value = settings.track_input;
    autostart.value = settings.autostart;
  } catch (error) {
    console.error("Failed to load settings:", error);
  }
}

onMounted(() => {
  loadSettings();
});
</script>

<template>
  <DbrCard class="settings-view">
    <header class="view-header">
      <h2 class="dbru-text-lg dbru-text-main">Настройки</h2>
    </header>

    <section class="settings-group">
      <div class="setting-item">
        <span class="setting-label dbru-text-base dbru-text-main">Пауза трекинга</span>
        <DbrCheckbox v-model="paused" @change="saveSettings" />
      </div>

      <div class="setting-item">
        <span class="setting-label dbru-text-base dbru-text-main">Трекинг заголовков окон</span>
        <DbrCheckbox v-model="trackWindowTitles" @change="saveSettings" />
      </div>

      <div class="setting-item">
        <span class="setting-label dbru-text-base dbru-text-main">Трекинг ввода</span>
        <DbrCheckbox v-model="trackInput" @change="saveSettings" />
      </div>

      <div class="setting-item">
        <span class="setting-label dbru-text-base dbru-text-main">Автозапуск</span>
        <DbrCheckbox v-model="autostart" @change="saveSettings" />
      </div>
    </section>
  </DbrCard>
</template>