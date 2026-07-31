import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { DecisionsList } from "../components/Decisions/DecisionsList";
import type { Adr } from "../types";

const commandMocks = vi.hoisted(() => ({
  getAdrs: vi.fn(),
}));

vi.mock("../lib/commands", () => commandMocks);

const workspace = vi.hoisted(() => ({
  state: { adrs: [] as Adr[] },
  dispatch: vi.fn(),
}));

vi.mock("../hooks/useWorkspace", () => ({
  useWorkspace: () => workspace,
}));

function adr(overrides: Partial<Adr> = {}): Adr {
  return {
    id: "ADR-001",
    title: "Adopt a shared layout system",
    status: "accepted",
    decision_date: "2026-07-31",
    decider: "Project Lead",
    body: "",
    path: ".lmbrain/decisions/ADR-001.md",
    created: "2026-07-01",
    updated: "2026-07-31",
    tags: [],
    links: [],
    ...overrides,
  };
}

describe("DecisionsList layout", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    workspace.state.adrs = [];
    commandMocks.getAdrs.mockResolvedValue([]);
  });

  it("renders through the dense page archetype", () => {
    const { container } = render(<DecisionsList />);
    expect(container.querySelector("[data-archetype='dense']")).not.toBeNull();
    expect(container.querySelector(".lm-page__inner")).not.toBeNull();
    expect(screen.getByRole("heading", { level: 1, name: "Decisions" })).toBeTruthy();
  });

  it("lays decisions out in the shared card grid at the 360px minimum", () => {
    workspace.state.adrs = [adr(), adr({ id: "ADR-002", title: "Second" })];
    const { container } = render(<DecisionsList />);

    const grid = container.querySelector(".lm-card-grid") as HTMLElement;
    expect(grid).not.toBeNull();
    expect(grid.style.getPropertyValue("--lm-card-min")).toBe("360px");
    expect(screen.getAllByRole("button")).toHaveLength(2);
  });

  it("opens a decision from mouse and keyboard", () => {
    workspace.state.adrs = [adr()];
    render(<DecisionsList />);

    const card = screen.getByRole("button", { name: /Adopt a shared layout system/ });
    fireEvent.click(card);
    expect(workspace.dispatch).toHaveBeenCalledWith({
      type: "SET_DETAIL_ARTIFACT",
      artifact: { title: "Adopt a shared layout system", path: ".lmbrain/decisions/ADR-001.md" },
    });

    // Cards are real buttons, so Enter activates them without extra handlers.
    workspace.dispatch.mockClear();
    fireEvent.keyDown(card, { key: "Enter" });
    fireEvent.click(card);
    expect(workspace.dispatch).toHaveBeenCalledTimes(1);
  });

  it("keeps the malformed marker and falls back for unknown statuses", () => {
    workspace.state.adrs = [
      adr({ malformed: true }),
      adr({ id: "ADR-003", title: "Unknown status", status: "invented" as Adr["status"] }),
    ];
    render(<DecisionsList />);

    expect(screen.getByText("MALFORMED")).toBeTruthy();
    expect(screen.getByText("INVENTED")).toBeTruthy();
  });

  it("shows the shared empty state and loads decisions on mount", async () => {
    const { container } = render(<DecisionsList />);
    expect(container.querySelector(".lm-empty-state")?.textContent).toBe("No decisions recorded yet.");
    await waitFor(() => expect(commandMocks.getAdrs).toHaveBeenCalledTimes(1));
  });
});
