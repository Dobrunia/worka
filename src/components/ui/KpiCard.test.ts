import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/vue";
import KpiCard from "@/components/ui/KpiCard.vue";

describe("KpiCard", () => {
  it("should render label and value correctly", () => {
    const label = "Активное время";
    const value = "5ч 30м";

    render(KpiCard, {
      props: { label, value },
    });

    expect(screen.getByText(label)).toBeInTheDocument();
    expect(screen.getByText(value)).toBeInTheDocument();
  });

  it("should apply correct CSS classes", () => {
    render(KpiCard, {
      props: { label: "Test", value: "100" },
    });

    const card = screen.getByText("Test").parentElement;
    expect(card).toHaveClass("kpi-card");
  });

  it("should render numeric values correctly", () => {
    render(KpiCard, {
      props: { label: "Нажатия", value: "1234" },
    });

    expect(screen.getByText("1234")).toBeInTheDocument();
  });

  it("should render time format correctly", () => {
    render(KpiCard, {
      props: { label: "Время", value: "2ч 15м" },
    });

    expect(screen.getByText("2ч 15м")).toBeInTheDocument();
  });

  it("should use dbru-text-sm class for label", () => {
    render(KpiCard, {
      props: { label: "Label", value: "Value" },
    });

    const labelElement = screen.getByText("Label");
    expect(labelElement).toHaveClass("dbru-text-sm");
  });

  it("should use dbru-text-lg class for value", () => {
    render(KpiCard, {
      props: { label: "Label", value: "Value" },
    });

    const valueElement = screen.getByText("Value");
    expect(valueElement).toHaveClass("dbru-text-lg");
  });
});
