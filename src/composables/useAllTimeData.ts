import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { formatTime } from "@/utils/time";

export interface AllTimeSummary {
  active_time_seconds: number;
  idle_time_seconds: number;
  keyboard_presses: number;
  mouse_clicks: number;
  top_apps: Array<{ name: string; time_seconds: number; percentage: number }>;
  days_tracked: number;
}

const allTimeSummary = ref<AllTimeSummary>({
  active_time_seconds: 0,
  idle_time_seconds: 0,
  keyboard_presses: 0,
  mouse_clicks: 0,
  top_apps: [],
  days_tracked: 0,
});
const isLoading = ref(false);

async function loadAllTimeSummary(): Promise<void> {
  isLoading.value = true;
  try {
    const data: AllTimeSummary = await invoke("get_all_time_summary");
    allTimeSummary.value = data;
  } catch (error) {
    console.error("Failed to load all-time summary:", error);
  } finally {
    isLoading.value = false;
  }
}

function startPolling(intervalMs = 60000): void {
  loadAllTimeSummary();
  setInterval(loadAllTimeSummary, intervalMs);
}

export function useAllTimeData() {
  return {
    allTimeSummary,
    isLoading,
    loadAllTimeSummary,
    startPolling,
    formatTime,
  };
}
