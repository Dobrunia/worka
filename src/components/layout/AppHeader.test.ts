import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen } from "@testing-library/vue";
import AppHeader from "@/components/layout/AppHeader.vue";

// Mock Tauri invoke
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("AppHeader", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.clearAllTimers();
  });

  it("should render Worka logo and title", () => {
    render(AppHeader);

    expect(screen.getByText("Worka")).toBeInTheDocument();
  });

  it("should display tracking active status by default", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockResolvedValue({ paused: false });

    render(AppHeader);

    // Ждём загрузки статуса и проверяем атрибут label
    await vi.waitFor(() => {
      const badge = document.querySelector("[data-v-6e76d7df][label]");
      expect(badge).toHaveAttribute("label", "Трекинг активен");
    });
  });

  it("should display paused status when tracking is paused", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockResolvedValue({ paused: true });

    render(AppHeader);

    await vi.waitFor(() => {
      const badge = document.querySelector("[data-v-6e76d7df][label]");
      expect(badge).toHaveAttribute("label", "На паузе");
    });
  });

  it("should use primary variant for active tracking badge", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockResolvedValue({ paused: false });

    render(AppHeader);

    await vi.waitFor(() => {
      const badge = document.querySelector("[data-v-6e76d7df][label]");
      expect(badge).toHaveClass("dbru-btn--primary");
    });
  });

  it("should render logo SVG icon", () => {
    render(AppHeader);

    const svg = document.querySelector("svg.logo-icon");
    expect(svg).toBeInTheDocument();
  });
});
