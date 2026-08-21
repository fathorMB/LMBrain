import { fireEvent, render, screen, within } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { TaskboardView } from "../components/Taskboard/TaskboardView";
import type { Spec } from "../types";

vi.mock("../lib/commands", () => ({
  getSpecs: vi.fn().mockResolvedValue([]),
}));

function spec(overrides: Partial<Spec>): Spec {
  return {
    id: "SPEC-000",
    title: "Untitled",
    status: "ready",
    priority: null,
    area: null,
    milestone: null,
    recommended_agent: null,
    capability_tier: null,
    thinking_level: null,
    depends_on: [],
    skills: [],
    body: "",
    path: ".lmbrain/specs/ready/SPEC-000.md",
    created: "2026-07-01",
    updated: "2026-07-01",
    tags: [],
    links: [],
    related_tasks: [],
    related_decisions: [],
    ...overrides,
  };
}

const specs: Spec[] = [
  spec({
    id: "SPEC-001",
    title: "Wiki navigation",
    tags: ["wiki", "ux", "markdown", "documentation"],
    capability_tier: "luna",
    thinking_level: "minimal",
  }),
  spec({ id: "SPEC-002", title: "MCP verbs", tags: ["mcp"], capability_tier: "sol" }),
  spec({ id: "SPEC-003", title: "Untagged work" }),
];

const workspace = vi.hoisted(() => ({
  state: { specs: [] as Spec[], debts: [] as unknown[] },
  openSpec: vi.fn(),
}));

vi.mock("../hooks/useWorkspace", () => ({
  useWorkspace: () => workspace,
}));

function readyColumn(): HTMLElement {
  // The Ready column header and its cards share a column container.
  return screen.getByText("Ready").closest("div")!.parentElement as HTMLElement;
}

describe("Board tags and effort tiers", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    workspace.state.specs = specs;
  });

  it("shows at most three tag chips plus an overflow count", () => {
    render(<TaskboardView />);
    const card = screen.getByText("Wiki navigation").parentElement as HTMLElement;
    expect(within(card).getByText("wiki")).toBeTruthy();
    expect(within(card).getByText("ux")).toBeTruthy();
    expect(within(card).getByText("markdown")).toBeTruthy();
    expect(within(card).queryByText("documentation")).toBeNull();
    expect(within(card).getByText("+1")).toBeTruthy();
  });

  it("labels the capability tier accessibly", () => {
    render(<TaskboardView />);
    expect(
      screen.getByLabelText("Capability tier luna, minimal reasoning"),
    ).toBeTruthy();
    expect(screen.getByLabelText("Capability tier sol")).toBeTruthy();
  });

  it("filters by an included tag and reports shown over total", () => {
    render(<TaskboardView />);
    expect(screen.getByText("Wiki navigation")).toBeTruthy();
    expect(screen.getByText("MCP verbs")).toBeTruthy();

    fireEvent.change(screen.getByLabelText("Add tag filter"), { target: { value: "wiki" } });

    expect(screen.getByText("Wiki navigation")).toBeTruthy();
    expect(screen.queryByText("MCP verbs")).toBeNull();
    expect(within(readyColumn()).getByText("1/3")).toBeTruthy();
  });

  it("excludes a tag and clears filters again", () => {
    render(<TaskboardView />);
    fireEvent.change(screen.getByLabelText("Exclude tag filter"), { target: { value: "mcp" } });
    expect(screen.queryByText("MCP verbs")).toBeNull();
    expect(screen.getByText("Wiki navigation")).toBeTruthy();

    fireEvent.click(screen.getByRole("button", { name: "Clear filters" }));
    expect(screen.getByText("MCP verbs")).toBeTruthy();
  });

  it("removes an active tag filter from its chip", () => {
    render(<TaskboardView />);
    fireEvent.change(screen.getByLabelText("Add tag filter"), { target: { value: "wiki" } });
    fireEvent.click(screen.getByRole("button", { name: "Remove include filter wiki" }));
    expect(screen.getByText("MCP verbs")).toBeTruthy();
  });

  it("filters by capability tier", () => {
    render(<TaskboardView />);
    fireEvent.change(screen.getByLabelText("Capability tier"), { target: { value: "sol" } });
    expect(screen.getByText("MCP verbs")).toBeTruthy();
    expect(screen.queryByText("Wiki navigation")).toBeNull();
    expect(screen.queryByText("Untagged work")).toBeNull();
  });

  it("offers untagged-only as its own toggle", () => {
    render(<TaskboardView />);
    fireEvent.click(screen.getByLabelText("Untagged only"));
    expect(screen.getByText("Untagged work")).toBeTruthy();
    expect(screen.queryByText("Wiki navigation")).toBeNull();
  });

  it("hides the tag controls when no spec carries a tag", () => {
    workspace.state.specs = [spec({ id: "SPEC-009", title: "Plain" })];
    render(<TaskboardView />);
    expect(screen.queryByLabelText("Add tag filter")).toBeNull();
    expect(screen.getByLabelText("Capability tier")).toBeTruthy();
  });
});
