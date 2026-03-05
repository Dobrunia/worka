import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { formatTime } from "@/utils/time";

export interface TodaySummary {
  active_time_seconds: number;
  idle_time_seconds: number;
  keyboard_presses: number;
  mouse_clicks: number;
  top_apps: Array<{
    name: string;
    icon_data_url?: string | null;
    time_seconds: number;
    percentage: number;
  }>;
  is_paused: boolean;
}

export interface AppSettings {
  paused: boolean;
  sample_interval_seconds: number;
  idle_threshold_seconds: number;
  track_window_titles: boolean;
  track_input: boolean;
  autostart: boolean;
}

// ─── Singleton state ──────────────────────────────────────────────────────────
// Module-level refs shared across all component instances.
// App.vue owns the polling lifecycle via startPolling / stopPolling.

const summary = ref<TodaySummary>({
  active_time_seconds: 0,
  idle_time_seconds: 0,
  keyboard_presses: 0,
  mouse_clicks: 0,
  top_apps: [],
  is_paused: false,
});

const isPaused = ref(false);
const isLoading = ref(false);

let pollHandle: ReturnType<typeof setInterval> | null = null;

// ─── Actions ──────────────────────────────────────────────────────────────────

async function loadSummary(): Promise<void> {
  isLoading.value = true;
  try {
    const data: TodaySummary = await invoke("get_today_summary");
    summary.value = data;
    isPaused.value = data.is_paused;
  } catch (error) {
    console.error("Failed to load summary:", error);
  } finally {
    isLoading.value = false;
  }
}

function startPolling(intervalMs = 5000): void {
  if (pollHandle !== null) return; // already running
  loadSummary();
  pollHandle = setInterval(loadSummary, intervalMs);
}

function stopPolling(): void {
  if (pollHandle !== null) {
    clearInterval(pollHandle);
    pollHandle = null;
  }
}

// ─── Composable ───────────────────────────────────────────────────────────────

export function useTodayData() {
  return {
    summary,
    isPaused,
    isLoading,
    loadSummary,
    startPolling,
    stopPolling,
    formatTime,
  };
}
