import { describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { RoadmapView } from "../components/Roadmap/RoadmapView";
import * as commands from "../lib/commands";

const mockOpenDetailArtifact = vi.fn();

vi.mock("../lib/commands", () => ({
  getMilestoneOverview: vi.fn(),
}));

vi.mock("../hooks/useWorkspace", () => ({
  useWorkspace: () => ({
    state: {},
    dispatch: vi.fn(),
    openDetailArtifact: mockOpenDetailArtifact,
  }),
}));

describe("RoadmapView", () => {
  beforeEach(() => {
    mockOpenDetailArtifact.mockClear();
  });

  it("renders milestone intelligence with overview and detail", async () => {
    vi.mocked(commands.getMilestoneOverview).mockResolvedValue({
      title: "Roadmap",
      milestones: [{
        id: "M-01",
        title: "First milestone",
        status: "active",
        outcome: "Deliver the core.",
        depends_on: null,
        risks: ["API stability"],
        spec_count: 2,
        spec_counts_by_status: { done: 1, ready: 1 },
        specs: [
          { id: "SPEC-001", title: "Setup", status: "done", priority: "high", area: "core", recommended_agent: "AGENT-IMPL", path: ".lmbrain/specs/done/SPEC-001-setup.md" },
          { id: "SPEC-002", title: "Integration", status: "ready", priority: "medium", area: "core", recommended_agent: null, path: ".lmbrain/specs/ready/SPEC-002-integration.md" },
        ],
        reviews: [
          { id: "REVIEW-001", title: "Review of SPEC-001", status: "accepted", spec_id: "SPEC-001", path: ".lmbrain/reviews/accepted/REVIEW-001-spec-001.md" },
        ],
        decisions: [
          { id: "ADR-001", title: "Architecture decision", status: "accepted", path: ".lmbrain/decisions/ADR-001-arch.md" },
        ],
        unresolved_refs: [],
        next_action: "1 ready spec(s) ready for handoff",
        progress_pct: 50,
      }],
      unmapped_specs: [],
      warnings: [],
    });

    render(<RoadmapView />);

    await waitFor(() => {
      const matches = screen.getAllByText("First milestone");
      expect(matches.length).toBeGreaterThanOrEqual(1);
    });

    // Milestone list shows the ID and status
    expect(screen.getAllByText("M-01").length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText("ACTIVE").length).toBeGreaterThanOrEqual(1);

    // Detail shows specs with status chips
    expect(screen.getByText("SPEC-001")).toBeDefined();
    expect(screen.getByText("SPEC-002")).toBeDefined();
    expect(screen.getByText("Setup")).toBeDefined();
    expect(screen.getByText("Integration")).toBeDefined();

    // Shows spec counts by status
    expect(screen.getByText("done: 1")).toBeDefined();
    expect(screen.getByText("ready: 1")).toBeDefined();

    // Shows next action
    expect(screen.getByText(/ready spec/)).toBeDefined();

    // Shows reviews
    expect(screen.getByText("REVIEW-001")).toBeDefined();

    // Shows decisions
    expect(screen.getByText("ADR-001")).toBeDefined();

    // Shows risks
    expect(screen.getByText("API stability")).toBeDefined();
  });

  it("opens spec detail with real path on click", async () => {
    vi.mocked(commands.getMilestoneOverview).mockResolvedValue({
      title: "Roadmap",
      milestones: [{
        id: "M-01", title: "Test", status: "active", outcome: "",
        depends_on: null, risks: [], spec_count: 1,
        spec_counts_by_status: { ready: 1 },
        specs: [
          { id: "SPEC-001", title: "Setup", status: "ready", priority: null, area: null, recommended_agent: null, path: ".lmbrain/specs/ready/SPEC-001-setup.md" },
        ],
        reviews: [], decisions: [], unresolved_refs: [],
        next_action: null, progress_pct: 0,
      }],
      unmapped_specs: [],
      warnings: [],
    });

    render(<RoadmapView />);

    await waitFor(() => expect(screen.getByText("SPEC-001")).toBeDefined());
    fireEvent.click(screen.getByText("SPEC-001"));
    expect(mockOpenDetailArtifact).toHaveBeenCalledWith({
      title: "Setup",
      path: ".lmbrain/specs/ready/SPEC-001-setup.md",
    });
  });

  it("opens review detail with real path on click", async () => {
    vi.mocked(commands.getMilestoneOverview).mockResolvedValue({
      title: "Roadmap",
      milestones: [{
        id: "M-01", title: "Test", status: "active", outcome: "",
        depends_on: null, risks: [], spec_count: 0,
        spec_counts_by_status: {},
        specs: [],
        reviews: [
          { id: "REVIEW-001", title: "Review of work", status: "accepted", spec_id: "SPEC-001", path: ".lmbrain/reviews/accepted/REVIEW-001-work.md" },
        ],
        decisions: [], unresolved_refs: [],
        next_action: null, progress_pct: 0,
      }],
      unmapped_specs: [],
      warnings: [],
    });

    render(<RoadmapView />);

    await waitFor(() => expect(screen.getByText("REVIEW-001")).toBeDefined());
    fireEvent.click(screen.getByText("REVIEW-001"));
    expect(mockOpenDetailArtifact).toHaveBeenCalledWith({
      title: "Review of work",
      path: ".lmbrain/reviews/accepted/REVIEW-001-work.md",
    });
  });

  it("opens ADR detail with real path on click", async () => {
    vi.mocked(commands.getMilestoneOverview).mockResolvedValue({
      title: "Roadmap",
      milestones: [{
        id: "M-01", title: "Test", status: "active", outcome: "",
        depends_on: null, risks: [], spec_count: 0,
        spec_counts_by_status: {},
        specs: [],
        reviews: [],
        decisions: [
          { id: "ADR-001", title: "Architecture choice", status: "accepted", path: ".lmbrain/decisions/ADR-001-arch.md" },
        ],
        unresolved_refs: [],
        next_action: null, progress_pct: 0,
      }],
      unmapped_specs: [],
      warnings: [],
    });

    render(<RoadmapView />);

    await waitFor(() => expect(screen.getByText("ADR-001")).toBeDefined());
    fireEvent.click(screen.getByText("ADR-001"));
    expect(mockOpenDetailArtifact).toHaveBeenCalledWith({
      title: "Architecture choice",
      path: ".lmbrain/decisions/ADR-001-arch.md",
    });
  });

  it("shows empty state when no milestones exist", async () => {
    vi.mocked(commands.getMilestoneOverview).mockResolvedValue({
      title: "Roadmap",
      milestones: [],
      unmapped_specs: [],
      warnings: [],
    });

    render(<RoadmapView />);

    await waitFor(() => expect(screen.getByText(/No milestones defined/)).toBeDefined());
  });

  it("shows unresolved references as warnings", async () => {
    vi.mocked(commands.getMilestoneOverview).mockResolvedValue({
      title: "Roadmap",
      milestones: [{
        id: "M-01", title: "Test", status: "planned", outcome: "",
        depends_on: "M-99", risks: [], spec_count: 0,
        spec_counts_by_status: {},
        specs: [], reviews: [], decisions: [],
        unresolved_refs: ["ADR-999 referenced in milestone M-01 not found"],
        next_action: "No specs assigned", progress_pct: 0,
      }],
      unmapped_specs: [],
      warnings: [],
    });

    render(<RoadmapView />);

    await waitFor(() => expect(screen.getByText("Unresolved References")).toBeDefined());
    expect(screen.getByText(/ADR-999/)).toBeDefined();
  });
});
