import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { DecisionsList } from "../components/Decisions/DecisionsList";
import type { Adr } from "../types";

const commandMocks = vi.hoisted(() => ({
  getAdrs: vi.fn(),
}));

vi.mock("../lib/commands", () => commandMocks);

const workspace = vi.hoisted(() => ({
  state: { adrs: [] as Adr[], specs: [] as unknown[], findings: [] as unknown[] },
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
    supersedes: [],
    superseded_by: [],
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

describe("DecisionsList lifecycle", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    workspace.state.specs = [];
    workspace.state.findings = [];
    commandMocks.getAdrs.mockResolvedValue([]);
  });

  it("groups by authority and collapses history behind a disclosure", () => {
    workspace.state.adrs = [
      adr({ id: "ADR-001", title: "Live" }),
      adr({ id: "ADR-002", title: "Pending", status: "proposed" }),
      adr({ id: "ADR-003", title: "Retired", status: "superseded" }),
    ];
    render(<DecisionsList />);

    expect(screen.getByRole("heading", { name: "Authoritative · 1" })).toBeTruthy();
    expect(screen.getByRole("heading", { name: "Awaiting decision · 1" })).toBeTruthy();

    const history = screen.getByRole("button", { name: /Historical · 1/ });
    expect(history.getAttribute("aria-expanded")).toBe("false");
    expect(screen.queryByText("Retired")).toBeNull();

    fireEvent.click(history);
    expect(history.getAttribute("aria-expanded")).toBe("true");
    expect(screen.getByText("Retired")).toBeTruthy();
  });

  it("reports a supersession the retired decision has not acknowledged", () => {
    workspace.state.adrs = [
      adr({ id: "ADR-010", title: "Successor", supersedes: ["ADR-009"] }),
      adr({ id: "ADR-009", title: "Predecessor" }),
    ];
    render(<DecisionsList />);

    const band = screen.getByRole("region", { name: "Needs attention" });
    expect(band.textContent).toContain("ADR-009 is still accepted although ADR-010 supersedes it");
  });

  it("stays silent about a proposal's pending supersession claim", () => {
    workspace.state.adrs = [
      adr({ id: "ADR-014", title: "Proposal", status: "proposed", supersedes: ["ADR-013"] }),
      adr({ id: "ADR-013", title: "Predecessor" }),
    ];
    render(<DecisionsList />);

    const band = screen.getByRole("region", { name: "Needs attention" });
    expect(band.textContent).not.toContain("supersedes it");
    expect(band.textContent).toContain("ADR-014 awaits an accept or reject decision");
  });

  it("renders rejected with its own treatment rather than the pending grey", () => {
    workspace.state.adrs = [adr({ id: "ADR-020", title: "Refused", status: "rejected" })];
    render(<DecisionsList />);

    fireEvent.click(screen.getByRole("button", { name: /Historical/ }));
    const badge = screen.getByText("REJECTED");
    expect(badge.style.color).toBe("rgb(224, 88, 74)");
  });

  it("counts inbound references from specs and findings", () => {
    workspace.state.adrs = [adr({ id: "ADR-001", title: "Cited" })];
    workspace.state.specs = [{ id: "SPEC-001", title: "A spec", related_decisions: ["ADR-001"] }];
    workspace.state.findings = [
      { id: "FINDING-001", title: "A finding", related_decisions: ["ADR-001"] },
    ];
    render(<DecisionsList />);

    expect(screen.getByText("2 references")).toBeTruthy();
  });

  it("filters by search and offers a way back", () => {
    workspace.state.adrs = [
      adr({ id: "ADR-001", title: "Layout system" }),
      adr({ id: "ADR-002", title: "Branching strategy" }),
    ];
    render(<DecisionsList />);

    fireEvent.change(screen.getByLabelText("Search decisions"), { target: { value: "branch" } });
    expect(screen.queryByText("Layout system")).toBeNull();
    expect(screen.getByText("Branching strategy")).toBeTruthy();

    fireEvent.click(screen.getByRole("button", { name: "Clear filters" }));
    expect(screen.getByText("Layout system")).toBeTruthy();
  });

  it("explains an empty result rather than showing nothing", () => {
    workspace.state.adrs = [adr({ id: "ADR-001", title: "Layout system" })];
    const { container } = render(<DecisionsList />);

    fireEvent.change(screen.getByLabelText("Search decisions"), { target: { value: "zzz" } });
    expect(container.querySelector(".lm-empty-state")?.textContent).toBe(
      "No decisions match these filters.",
    );
  });
});
