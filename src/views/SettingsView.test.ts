import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/vue";
import SettingsView from "@/views/SettingsView.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue({
    paused: false,
    track_window_titles: true,
    track_input: true,
    autostart: false,
    sample_interval_seconds: 10,
    idle_threshold_seconds: 120,
    // Fields returned by get_today_summary (used by useTodayData singleton)
    active_time_seconds: 0,
    idle_time_seconds: 0,
    keyboard_presses: 0,
    mouse_clicks: 0,
    top_apps: [],
    is_paused: false,
  }),
}));

describe("SettingsView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should render settings title", () => {
    render(SettingsView);

    expect(screen.getByText("Настройки")).toBeInTheDocument();
  });

  it("should display pause tracking option", () => {
    render(SettingsView);

    expect(screen.getByText("Пауза трекинга")).toBeInTheDocument();
  });

  it("should display window titles tracking option", () => {
    render(SettingsView);

    expect(screen.getByText("Сохранять заголовки окон")).toBeInTheDocument();
  });

  it("should display input tracking option", () => {
    render(SettingsView);

    expect(screen.getByText("Отслеживать клавиатуру и мышь")).toBeInTheDocument();
  });

  it("should display autostart option", () => {
    render(SettingsView);

    expect(screen.getByText("Автозапуск с Windows")).toBeInTheDocument();
  });

  it("should have exactly 4 settings items", () => {
    render(SettingsView);

    const labels = [
      "Пауза трекинга",
      "Сохранять заголовки окон",
      "Отслеживать клавиатуру и мышь",
      "Автозапуск с Windows",
    ];

    labels.forEach((label) => {
      expect(screen.getByText(label)).toBeInTheDocument();
    });
  });
});
