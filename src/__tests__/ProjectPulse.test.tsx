import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { ProjectPulse } from "../components/Pulse/ProjectPulse";

const mockOpenDetailArtifact = vi.fn();

vi.mock("../hooks/useWorkspace", () => ({
  useWorkspace: () => ({
    state: {
      pulseData: {
        focus: "M-01",
        milestone: "M-01",
        milestone_progress: 50,
        milestone_due: "2026-09-30",
        metrics: [],
        actions: [
          {
            title: "Start AGENT-FULLSTACK-DESKTOP on SPEC-016",
            description:
              "Spec is ready — copy the handoff prompt and launch the agent manually.",
            action_type: "handoff",
            spec_id: "SPEC-016",
            agent: "AGENT-FULLSTACK-DESKTOP",
          },
        ],
        blockers: [],
        recent_activity: [],
        ready_handoffs: [],
        active_handoff: null,
      },
      handoffs: [],
      adrs: [],
      agents: [],
      diagnostics: [
        {
          id: "DIAG-0123456789abcdef",
          code: "frontmatter-malformed",
          message: "YAML frontmatter is malformed: missing key",
          severity: "error",
          artifact_id: "SPEC-001",
          path: ".lmbrain/tasks/task-001.md",
          next_action: "Repair the YAML frontmatter before retrying.",
          fixability: "manual",
        },
      ],
      projectStatistics: {
        spec_flow: { total_specs: 1, done_specs: 0, open_specs: 1, done_ratio: 0, by_status: [], by_priority: [], by_area: [] },
        review_quality: {
          total_reviews: 0,
          total_review_passes: 0,
          reviewed_specs: 0,
          remediation_cycles: 0,
          escalation_count: 0,
          takeover_count: 0,
          lifecycle_known_reviews: 0,
          specs_with_changes_requested: 0,
          first_pass_accepted_specs: 0,
          first_pass_eligible_specs: 0,
          average_reviews_per_reviewed_spec: 0,
          lifecycle_coverage: 0,
          reviews_without_spec: 0,
          reviews_without_created: 0,
          accepted_reviews: 0,
          changes_requested_reviews: 0,
          blocked_reviews: 0,
          superseded_reviews: 0,
          specs_with_multiple_changes_requested: 0,
          change_request_rate: 0,
          first_pass_acceptance_rate: 0,
          by_area: [],
          by_agent: [],
          trend: [],
        },
        artifact_families: [],
        diagnostics: { total: 1, errors: 1, warnings: 0, by_family: [] },
      },
      currentWorkspace: {
        path: "E:/workspace",
        name: "workspace",
        kit_version: "2.1.2",
        project_kit_version: "2.1.2",
        bundled_kit_version: "2.2.7",
        bundled_kit_path: "E:/Git/LMBrain/kit/.lmbrain",
        kit_migration_status: "migration-available",
        health: "ok",
        diagnostics: [],
        branch: null,
        is_clean: null,
        spec_count: 1,
        task_count: 0,
        decision_count: 0,
        agent_count: 0,
      },
      gitInfo: null,
      watcherActive: false,
      specs: [
        {
          id: "SPEC-016",
          title: "Spec 16",
          status: "ready",
          priority: null,
          area: null,
          milestone: null,
          recommended_agent: "AGENT-FULLSTACK-DESKTOP",
          body: "",
          path: ".lmbrain/specs/ready/SPEC-016.md",
          created: "",
          updated: "",
          tags: [],
          links: [],
          related_tasks: [],
          related_decisions: [],
        },
      ],
    },
    openDetailArtifact: mockOpenDetailArtifact,
  }),
}));

const writeTextMock = vi.fn();
Object.assign(navigator, {
  clipboard: {
    writeText: writeTextMock,
  },
});

describe("ProjectPulse Diagnostics Fix Prompt", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders diagnostics via InsightReliability and expands fix prompt", async () => {
    render(<ProjectPulse />);

    await waitFor(() => {
      expect(screen.getByText("Needs attention")).toBeDefined();
    });

    const summary = screen.getByText(/Diagnostic details/i);
    fireEvent.click(summary);

    await waitFor(() => {
      expect(
        screen.getByText("YAML frontmatter is malformed: missing key"),
      ).toBeDefined();
      expect(screen.getByText("Copy fix prompt")).toBeDefined();
    });

    fireEvent.click(screen.getByText("Copy fix prompt"));

    expect(writeTextMock).toHaveBeenCalled();
    expect(writeTextMock.mock.calls[0][0]).toContain(
      "Please address DIAG-0123456789abcdef (frontmatter-malformed)",
    );
  });

  it("reveals and copies a manual handoff prompt without writing project state", async () => {
    render(<ProjectPulse />);

    await waitFor(() => expect(screen.getByText("View prompt")).toBeDefined());
    fireEvent.click(screen.getByText("View prompt"));
    expect(
      (screen.getByLabelText("Handoff prompt for SPEC-016") as HTMLTextAreaElement)
        .value,
    ).toContain(".lmbrain/specs/ready/SPEC-016.md");

    fireEvent.click(screen.getByText("Copy prompt"));
    await waitFor(() =>
      expect(screen.getByText("Copied to clipboard.")).toBeDefined(),
    );
    expect(writeTextMock).toHaveBeenCalledWith(
      expect.stringContaining("AGENT-FULLSTACK-DESKTOP"),
    );
  });

  it("opens STATUS.md and ROADMAP.md in the detail modal", async () => {
    render(<ProjectPulse />);

    await waitFor(() =>
      expect(screen.getByLabelText("Open STATUS.md")).toBeDefined(),
    );
    mockOpenDetailArtifact.mockClear();
    fireEvent.click(screen.getByLabelText("Open STATUS.md"));
    expect(mockOpenDetailArtifact).toHaveBeenCalledWith({
      title: "STATUS.md",
      path: "E:/workspace/.lmbrain/STATUS.md",
    });

    fireEvent.click(screen.getByLabelText("Open ROADMAP.md"));
    expect(mockOpenDetailArtifact).toHaveBeenCalledWith({
      title: "ROADMAP.md",
      path: "E:/workspace/.lmbrain/ROADMAP.md",
    });
  });

  it("keeps status available as a quick link without rendering its focus inline", async () => {
    render(<ProjectPulse />);

    expect(screen.queryByText("Current focus:")).toBeNull();
    expect(screen.queryByText("M-01", { selector: "p *" })).toBeNull();
    await waitFor(() =>
      expect(screen.getByLabelText("Open STATUS.md")).toBeDefined(),
    );
  });

  it("renders kit version metadata and handles Copy migration prompt click", async () => {
    render(<ProjectPulse />);

    await waitFor(() => {
      expect(screen.getByText("Bundled kit")).toBeDefined();
      expect(screen.getByText("Kit status")).toBeDefined();
      expect(screen.getByText("Migration available")).toBeDefined();
    });

    const copyBtn = screen.getByText("Copy migration prompt");
    expect(copyBtn).toBeDefined();

    fireEvent.click(copyBtn);
    expect(writeTextMock).toHaveBeenCalled();
    expect(writeTextMock.mock.calls[writeTextMock.mock.calls.length - 1][0]).toContain(
      "You are the Project Lead. The LMBrain application detected that this project's kit version is older"
    );
    expect(writeTextMock.mock.calls[writeTextMock.mock.calls.length - 1][0]).toContain(
      "Bundled kit source path: E:/Git/LMBrain/kit/.lmbrain"
    );
    expect(writeTextMock.mock.calls[writeTextMock.mock.calls.length - 1][0]).toContain(
      "authoritative source"
    );
  });
});
