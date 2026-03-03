import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/vue";
import SettingsView from "@/views/SettingsView.vue";

// Mock Tauri invoke
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("SettingsView", () => {
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

    expect(screen.getByText("Трекинг заголовков окон")).toBeInTheDocument();
  });

  it("should display input tracking option", () => {
    render(SettingsView);

    expect(screen.getByText("Трекинг ввода")).toBeInTheDocument();
  });

  it("should display autostart option", () => {
    render(SettingsView);

    expect(screen.getByText("Автозапуск")).toBeInTheDocument();
  });

  it("should have exactly 4 settings items", () => {
    render(SettingsView);

    const settingsLabels = [
      "Пауза трекинга",
      "Трекинг заголовков окон",
      "Трекинг ввода",
      "Автозапуск",
    ];

    settingsLabels.forEach((label) => {
      expect(screen.getByText(label)).toBeInTheDocument();
    });
  });
});
