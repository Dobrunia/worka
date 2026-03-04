import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { formatTime } from "@/utils/time";

export interface WeekDay {
  date: string;
  day_name: string;
  active_time_seconds: number;
  idle_time_seconds: number;
  keyboard_presses: number;
  mouse_clicks: number;
}

export interface WeekSummary {
  days: WeekDay[];
}

const weekSummary = ref<WeekSummary>({ days: [] });
const isLoading = ref(false);

async function loadWeekSummary(): Promise<void> {
  isLoading.value = true;
  try {
    const data: WeekSummary = await invoke("get_week_summary");
    weekSummary.value = data;
  } catch (error) {
    console.error("Failed to load week summary:", error);
  } finally {
    isLoading.value = false;
  }
}

function startPolling(intervalMs = 60000): void {
  loadWeekSummary().then(() => {
    setInterval(loadWeekSummary, intervalMs);
  });
}

export function useWeekData() {
  return {
    weekSummary,
    isLoading,
    loadWeekSummary,
    startPolling,
    formatTime,
  };
}
