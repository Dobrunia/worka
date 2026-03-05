import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { formatTime } from "@/utils/time";

export interface TimelineHourApp {
  name: string;
  icon_data_url?: string | null;
  time_seconds: number;
  percentage: number;
}

export interface TimelineHour {
  hour: string;
  hour_index: number;
  active_time_seconds: number;
  idle_time_seconds: number;
  keyboard_presses: number;
  mouse_clicks: number;
  top_app?: {
    name: string;
    icon_data_url?: string | null;
    time_seconds: number;
  } | null;
  apps: TimelineHourApp[];
}

export interface TimelineSummary {
  date: string;
  hours: TimelineHour[];
  has_data: boolean;
}

function getLocalTodayIsoDate(): string {
  const now = new Date();
  const year = now.getFullYear();
  const month = String(now.getMonth() + 1).padStart(2, "0");
  const day = String(now.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

const selectedDate = ref(getLocalTodayIsoDate());
const timeline = ref<TimelineSummary>({
  date: selectedDate.value,
  hours: Array.from({ length: 24 }, (_, hourIndex) => ({
    hour: `${String(hourIndex).padStart(2, "0")}:00`,
    hour_index: hourIndex,
    active_time_seconds: 0,
    idle_time_seconds: 0,
    keyboard_presses: 0,
    mouse_clicks: 0,
    top_app: null,
    apps: [],
  })),
  has_data: false,
});
const isLoading = ref(false);

async function loadTimeline(date = selectedDate.value): Promise<void> {
  isLoading.value = true;
  try {
    const data: TimelineSummary = await invoke("get_timeline", { date });
    timeline.value = data;
  } catch (error) {
    console.error("Failed to load timeline:", error);
  } finally {
    isLoading.value = false;
  }
}

export function useTimelineData() {
  return {
    selectedDate,
    timeline,
    isLoading,
    loadTimeline,
    formatTime,
  };
}
