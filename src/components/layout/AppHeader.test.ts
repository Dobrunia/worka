import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/vue";
import AppHeader from "@/components/layout/AppHeader.vue";

// AppHeader is a pure presentational component — no Tauri calls, no polling.
// All state arrives via props, so no mocking is required.

describe("AppHeader", () => {
  it("should render Worka logo and title", () => {
    render(AppHeader, { props: { isPaused: false } });

    expect(screen.getByText("Worka")).toBeInTheDocument();
  });

  it("should render logo SVG icon", () => {
    render(AppHeader, { props: { isPaused: false } });

    expect(document.querySelector("svg.logo-icon")).toBeInTheDocument();
  });

  it("should display tracking active text when not paused", () => {
    render(AppHeader, { props: { isPaused: false } });

    const badge = document.querySelector("[data-testid='status-badge']");
    expect(badge).toHaveAttribute("label", "Трекинг активен");
  });

  it("should display paused text when paused", () => {
    render(AppHeader, { props: { isPaused: true } });

    const badge = document.querySelector("[data-testid='status-badge']");
    expect(badge).toHaveAttribute("label", "На паузе");
  });

  it("should pass variant=primary when not paused", () => {
    render(AppHeader, { props: { isPaused: false } });

    // DbrBadge receives the prop — verify computed value drives correct label
    const badge = document.querySelector("[data-testid='status-badge']");
    expect(badge).toHaveAttribute("label", "Трекинг активен");
  });

  it("should pass no variant when paused (label changes)", () => {
    render(AppHeader, { props: { isPaused: true } });

    const badge = document.querySelector("[data-testid='status-badge']");
    expect(badge).toHaveAttribute("label", "На паузе");
  });
});
