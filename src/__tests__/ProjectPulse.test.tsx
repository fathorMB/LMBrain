import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { ProjectPulse } from "../components/Pulse/ProjectPulse";

const mockDispatch = vi.fn();

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
          message: "YAML frontmatter is malformed: missing key",
          severity: "error",
          path: ".lmbrain/tasks/task-001.md",
        },
      ],
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
    dispatch: mockDispatch,
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

  it("renders diagnostics and expands fix prompt when clicking Fix button", async () => {
    render(<ProjectPulse />);

    await waitFor(() => {
      expect(
        screen.getByText("YAML frontmatter is malformed: missing key"),
      ).toBeDefined();
    });

    fireEvent.click(screen.getByText("Fix"));

    await waitFor(() => {
      expect(screen.getByText("Copy fix prompt")).toBeDefined();
    });

    fireEvent.click(screen.getByText("Copy fix prompt"));

    expect(writeTextMock).toHaveBeenCalled();
    expect(writeTextMock.mock.calls[0][0]).toContain(
      "Please fix the malformed frontmatter",
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
      expect(screen.getByRole("status").textContent).toBe("Copied to clipboard."),
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
    mockDispatch.mockClear();
    fireEvent.click(screen.getByLabelText("Open STATUS.md"));
    expect(mockDispatch).toHaveBeenCalledWith({
      type: "SET_DETAIL_ARTIFACT",
      artifact: { title: "STATUS.md", path: "E:/workspace/.lmbrain/STATUS.md" },
    });

    fireEvent.click(screen.getByLabelText("Open ROADMAP.md"));
    expect(mockDispatch).toHaveBeenCalledWith({
      type: "SET_DETAIL_ARTIFACT",
      artifact: { title: "ROADMAP.md", path: "E:/workspace/.lmbrain/ROADMAP.md" },
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
