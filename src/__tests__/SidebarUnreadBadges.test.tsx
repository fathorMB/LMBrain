import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { Sidebar } from "../components/Layout/Sidebar";

const workspace = vi.hoisted(() => ({
  state: { view: "pulse", findings: [] },
  unreadCounts: {
    taskboard: 0,
    reviews: 0,
    findings: 0,
    feedback: 0,
    decisions: 0,
    agents: 0,
    mcp: 0,
    skills: 0,
  } as Record<string, number>,
  navigateTo: vi.fn(),
  triggerLeaveWorkspace: vi.fn(),
  toggleCmdk: vi.fn(),
}));

vi.mock("../hooks/useWorkspace", () => ({
  useWorkspace: () => workspace,
}));

function resetCounts() {
  for (const key of Object.keys(workspace.unreadCounts)) {
    workspace.unreadCounts[key] = 0;
  }
}

describe("Sidebar unread badges", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    workspace.state.view = "pulse";
    resetCounts();
  });

  it("shows a numeric badge only for pages with unread items", () => {
    workspace.unreadCounts.reviews = 3;
    render(<Sidebar />);

    expect(screen.getByRole("link", { name: "Reviews, 3 unread items" })).toBeTruthy();
    expect(screen.getByRole("link", { name: "Findings" })).toBeTruthy();
    expect(screen.getByTitle("3 unread").textContent).toBe("3");
    expect(screen.queryByTitle("0 unread")).toBeNull();
  });

  it("badges every eligible workspace page", () => {
    for (const key of Object.keys(workspace.unreadCounts)) {
      workspace.unreadCounts[key] = 1;
    }
    render(<Sidebar />);

    for (const label of ["Board", "Reviews", "Findings", "Kit Feedback", "Decisions", "Agents", "MCP", "Skills"]) {
      expect(screen.getByRole("link", { name: `${label}, 1 unread item` })).toBeTruthy();
    }
  });

  it("never badges Wiki, Design or Repository", () => {
    for (const key of Object.keys(workspace.unreadCounts)) {
      workspace.unreadCounts[key] = 5;
    }
    render(<Sidebar />);

    for (const label of ["Wiki", "Design", "Repository"]) {
      const item = screen.getByRole("link", { name: label });
      expect(item.textContent?.endsWith(label)).toBe(true);
      expect(item.querySelector("[title]")).toBeNull();
    }
  });

  it("marks the active page and navigates from mouse and keyboard", () => {
    workspace.state.view = "findings";
    render(<Sidebar />);

    const findings = screen.getByRole("link", { name: "Findings" });
    expect(findings.getAttribute("aria-current")).toBe("page");

    fireEvent.click(screen.getByRole("link", { name: "Decisions" }));
    expect(workspace.navigateTo).toHaveBeenCalledWith("decisions");

    fireEvent.keyDown(screen.getByRole("link", { name: "Skills" }), { key: "Enter" });
    expect(workspace.navigateTo).toHaveBeenCalledWith("skills");
  });

  it("does not break when counts are unavailable", () => {
    const counts = workspace.unreadCounts;
    (workspace as { unreadCounts: Record<string, number> }).unreadCounts = {};
    try {
      render(<Sidebar />);
      expect(screen.getByRole("link", { name: "Reviews" })).toBeTruthy();
    } finally {
      (workspace as { unreadCounts: Record<string, number> }).unreadCounts = counts;
    }
  });
});
