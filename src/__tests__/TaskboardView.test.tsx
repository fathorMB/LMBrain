import { describe, it, expect, vi, beforeEach } from "vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { TaskboardView } from "../components/Taskboard/TaskboardView";
import * as commands from "../lib/commands";
import type { Spec } from "../types";

vi.mock("../lib/commands", () => ({
  getSpecs: vi.fn(),
}));

const baseSpec: Spec = {
  id: "SPEC-001",
  title: "A working spec",
  status: "working",
  priority: null,
  area: null,
  milestone: null,
  recommended_agent: "AGENT-001",
  body: "## Acceptance criteria\n- [x] one\n- [ ] two\n\n## Evidence\nproof\n",
  path: "C:/ws/.lmbrain/specs/working/SPEC-001.md",
  created: "2026-06-23",
  updated: "2026-06-23",
  tags: [],
  links: [],
  related_tasks: [],
  related_decisions: [],
};

vi.mock("../hooks/useWorkspace", () => ({
  useWorkspace: () => ({
    state: { specs: [baseSpec] },
    openSpec: vi.fn(),
  }),
}));

describe("Board (spec board)", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    baseSpec.depends_on = [];
    baseSpec.parking_events = [];
    baseSpec.status = "working";
    vi.mocked(commands.getSpecs).mockResolvedValue([baseSpec]);
  });

  it("renders a spec card with its id and acceptance-criteria progress", async () => {
    render(<TaskboardView />);
    await waitFor(() => expect(screen.getByText("SPEC-001")).toBeDefined());
    expect(screen.getByText("A working spec")).toBeDefined();
    // 1 of 2 acceptance criteria checked
    expect(screen.getByText("1/2")).toBeDefined();
  });

  it("renders the spec board columns", async () => {
    render(<TaskboardView />);
    await waitFor(() => expect(screen.getByText("Working")).toBeDefined());
    expect(screen.getByText("Backlog")).toBeDefined();
    expect(screen.getByText("Discarded")).toBeDefined();
  });

  it("filters and labels dependency-blocked specs without exposing lifecycle actions", async () => {
    baseSpec.depends_on = ["SPEC-099"];
    render(<TaskboardView />);
    await waitFor(() => expect(screen.getByLabelText(/Blocked by hard dependencies/)).toBeDefined());
    fireEvent.change(screen.getByLabelText("Dependency view"), { target: { value: "ready-after" } });
    expect(screen.queryByText("SPEC-001")).toBeNull();
    fireEvent.change(screen.getByLabelText("Dependency view"), { target: { value: "blocked" } });
    expect(screen.getByText("SPEC-001")).toBeDefined();
    expect(screen.queryByRole("button", { name: /approve|park|status/i })).toBeNull();
  });

  it("distinguishes parked backlog work from ordinary backlog", async () => {
    baseSpec.status = "backlog";
    baseSpec.parking_events = [{
      timestamp: "2026-07-29T12:00:00+02:00",
      actor: "AGENT-LEAD",
      reason: "Milestone order changed",
      revisit_condition: null,
    }];
    render(<TaskboardView />);
    await waitFor(() => expect(screen.getByText("Parked · readiness expired")).toBeDefined());
  });
});
