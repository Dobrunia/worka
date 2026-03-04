import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/vue";
import AppHeader from "@/components/layout/AppHeader.vue";

describe("AppHeader", () => {
  it("should render Worka logo text", () => {
    render(AppHeader, { props: { isPaused: false } });
    expect(screen.getByText("Worka")).toBeInTheDocument();
  });

  it("should render logo SVG icon", () => {
    render(AppHeader, { props: { isPaused: false } });
    expect(document.querySelector("svg.logo-icon")).toBeInTheDocument();
  });

  it("should display active tracking text when not paused", () => {
    render(AppHeader, { props: { isPaused: false } });
    const badge = document.querySelector("[data-testid='status-badge']");
    expect(badge?.textContent?.trim()).toContain("Трекинг активен");
  });

  it("should display paused text when paused", () => {
    render(AppHeader, { props: { isPaused: true } });
    const badge = document.querySelector("[data-testid='status-badge']");
    expect(badge?.textContent?.trim()).toContain("На паузе");
  });

  it("should have paused modifier class when paused", () => {
    render(AppHeader, { props: { isPaused: true } });
    const badge = document.querySelector("[data-testid='status-badge']");
    expect(badge).toHaveClass("status--paused");
  });

  it("should not have paused modifier class when active", () => {
    render(AppHeader, { props: { isPaused: false } });
    const badge = document.querySelector("[data-testid='status-badge']");
    expect(badge).not.toHaveClass("status--paused");
  });
});
