import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/vue";
import TopAppsList, { type AppUsage } from "@/components/ui/TopAppsList.vue";

describe("TopAppsList", () => {
  const ICON_DATA_URL =
    "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAusB9Wf6nfsAAAAASUVORK5CYII=";

  it("should show empty state when no apps provided", () => {
    render(TopAppsList, {
      props: { apps: [] },
    });

    expect(screen.getByText("Нет данных")).toBeInTheDocument();
  });

  it("should show empty state when apps is undefined", () => {
    render(TopAppsList, {
      props: {},
    });

    expect(screen.getByText("Нет данных")).toBeInTheDocument();
  });

  it("should render app list with data", () => {
    const apps: AppUsage[] = [
      { name: "VS Code", timeSeconds: 3600, percentage: 50 },
      { name: "Chrome", timeSeconds: 1800, percentage: 25 },
    ];

    render(TopAppsList, {
      props: { apps },
    });

    expect(screen.getByText("VS Code")).toBeInTheDocument();
    expect(screen.getByText("Chrome")).toBeInTheDocument();
  });

  it("should format time correctly for hours and minutes", () => {
    const apps: AppUsage[] = [
      { name: "App", timeSeconds: 3665, percentage: 100 },
    ];

    render(TopAppsList, {
      props: { apps },
    });

    expect(screen.getByText("1ч 1м")).toBeInTheDocument();
  });

  it("should prefer icon rendering and keep app name in tooltip", () => {
    const apps: AppUsage[] = [
      { name: "VS Code", iconDataUrl: ICON_DATA_URL, timeSeconds: 3600, percentage: 100 },
    ];

    render(TopAppsList, {
      props: { apps },
    });

    expect(screen.getByAltText("VS Code")).toBeInTheDocument();
    expect(screen.queryByText("VS Code")).not.toBeInTheDocument();
  });

  it("should format time correctly for minutes only", () => {
    const apps: AppUsage[] = [
      { name: "App", timeSeconds: 300, percentage: 100 },
    ];

    render(TopAppsList, {
      props: { apps },
    });

    expect(screen.getByText("5м")).toBeInTheDocument();
  });

  it("should render progress bar with correct width", () => {
    const apps: AppUsage[] = [
      { name: "App", timeSeconds: 100, percentage: 75 },
    ];

    render(TopAppsList, {
      props: { apps },
    });

    const progressFill = document.querySelector(".progress-fill") as HTMLElement;
    expect(progressFill).toHaveStyle("width: 75%");
  });

  it("should display list title", () => {
    render(TopAppsList, {
      props: { apps: [] },
    });

    expect(screen.getByText("Топ приложений")).toBeInTheDocument();
  });
});
