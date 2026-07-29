import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { SpecDetail } from "../components/Spec/SpecDetail";
import type { Spec } from "../types";

const closeSpecDetail = vi.fn();

const spec: Spec = {
  id: "SPEC-011",
  title: "Breadcrumb regression",
  status: "ready",
  priority: "high",
  area: "desktop-navigation",
  milestone: "M-07",
  recommended_agent: "AGENT-FULLSTACK-DESKTOP",
  body: "## Objective\nReturn to the Board.",
  path: "C:/workspace/.lmbrain/specs/ready/SPEC-011.md",
  created: "2026-07-29",
  updated: "2026-07-29",
  tags: [],
  links: [],
  related_tasks: [],
  related_decisions: [],
};

vi.mock("../hooks/useWorkspace", () => ({
  useWorkspace: () => ({
    state: {
      specs: [spec],
      selectedSpec: spec,
    },
    closeSpecDetail,
  }),
}));

describe("SpecDetail breadcrumb", () => {
  beforeEach(() => {
    closeSpecDetail.mockClear();
    spec.status = "ready";
    spec.depends_on = [];
    spec.parking_events = [];
  });

  it("uses an accessible native button and returns through the dedicated Board action", () => {
    render(<SpecDetail />);

    const breadcrumb = screen.getByRole("button", {
      name: "Back to specification board",
    });
    expect(breadcrumb.getAttribute("type")).toBe("button");

    fireEvent.click(breadcrumb);

    expect(closeSpecDetail).toHaveBeenCalledTimes(1);
  });

  it("is keyboard-focusable through native button semantics", () => {
    render(<SpecDetail />);

    const breadcrumb = screen.getByRole("button", {
      name: "Back to specification board",
    });
    breadcrumb.focus();

    expect(document.activeElement).toBe(breadcrumb);
    expect(closeSpecDetail).not.toHaveBeenCalled();
  });

  it("shows dependency blockers without lifecycle mutation controls", () => {
    spec.depends_on = ["SPEC-010"];
    render(<SpecDetail />);

    expect(screen.getByRole("region", { name: "Hard spec dependencies" })).toBeTruthy();
    expect(screen.getByText("SPEC-010 · missing")).toBeTruthy();
    expect(screen.getByText(/lifecycle changes are intentionally unavailable/i)).toBeTruthy();
    expect(screen.queryByRole("button", { name: /approve|park|change status/i })).toBeNull();
  });

  it("shows preserved parking evidence but no re-approval or status action", () => {
    spec.status = "backlog";
    spec.parking_events = [{
      timestamp: "2026-07-29T12:00:00+02:00",
      actor: "AGENT-LEAD",
      reason: "Milestone order changed",
      revisit_condition: "After SPEC-010",
    }];
    render(<SpecDetail />);

    expect(screen.getByRole("region", { name: "Parking history" })).toBeTruthy();
    expect(screen.getByText("Milestone order changed")).toBeTruthy();
    expect(screen.getByText(/Re-approval and lifecycle actions are intentionally unavailable/i)).toBeTruthy();
    expect(screen.queryByRole("button", { name: /re-approve|park|status/i })).toBeNull();
  });
});
