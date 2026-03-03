import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

export interface TodaySummary {
  active_time_seconds: number;
  idle_time_seconds: number;
  keyboard_presses: number;
  mouse_clicks: number;
  top_apps: Array<{ name: string; time_seconds: number; percentage: number }>;
}

export interface AppState {
  paused: boolean;
  sample_interval_seconds: number;
  idle_threshold_seconds: number;
  track_window_titles: boolean;
  track_input: boolean;
  autostart: boolean;
}

export async function getTodaySummary(): Promise<TodaySummary> {
  return await invoke("get_today_summary");
}

export async function getSettings(): Promise<AppState> {
  return await invoke("get_settings");
}

export async function togglePause(): Promise<boolean> {
  return await invoke("toggle_pause");
}

export function useTodayData() {
  const summary = ref<TodaySummary>({
    active_time_seconds: 0,
    idle_time_seconds: 0,
    keyboard_presses: 0,
    mouse_clicks: 0,
    top_apps: [],
  });

  const isPaused = ref(false);
  const isLoading = ref(false);

  async function loadSummary() {
    isLoading.value = true;
    try {
      const data = await getTodaySummary();
      summary.value = data;
    } catch (error) {
      console.error("Failed to load summary:", error);
    } finally {
      isLoading.value = false;
    }
  }

  async function loadSettings() {
    try {
      const settings = await getSettings();
      isPaused.value = settings.paused;
    } catch (error) {
      console.error("Failed to load settings:", error);
    }
  }

  async function togglePauseState() {
    try {
      isPaused.value = await togglePause();
    } catch (error) {
      console.error("Failed to toggle pause:", error);
    }
  }

  function formatTime(seconds: number): string {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    if (hours > 0) {
      return `${hours}ч ${minutes}м`;
    }
    return `${minutes}м`;
  }

  return {
    summary,
    isPaused,
    isLoading,
    loadSummary,
    loadSettings,
    togglePauseState,
    formatTime,
  };
}
