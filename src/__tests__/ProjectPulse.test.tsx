import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { ProjectPulse } from "../components/Pulse/ProjectPulse";
import * as commands from "../lib/commands";

vi.mock("../lib/commands", () => ({
  getPulseData: vi.fn(),
  getAdrs: vi.fn(),
  getHandoffs: vi.fn(),
  getAgents: vi.fn(),
  getDiagnostics: vi.fn(),
}));

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
        actions: [{
          title: "Start AGENT-FULLSTACK-DESKTOP on SPEC-016",
          description: "Spec is ready — copy the handoff prompt and launch the agent manually.",
          action_type: "handoff",
          spec_id: "SPEC-016",
          agent: "AGENT-FULLSTACK-DESKTOP",
        }],
        blockers: [],
        recent_activity: [],
        ready_handoffs: [],
        active_handoff: null,
      },
      handoffs: [],
      adrs: [],
      agents: [],
      currentWorkspace: { path: "E:/workspace" },
    },
    dispatch: mockDispatch,
  }),
}));

// Mock navigator.clipboard
const writeTextMock = vi.fn();
Object.assign(navigator, {
  clipboard: {
    writeText: writeTextMock,
  },
});

describe("ProjectPulse Diagnostics Fix Prompt", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(commands.getPulseData).mockResolvedValue({
      focus: "M-01",
      milestone: "M-01",
      milestone_progress: 50,
      milestone_due: "2026-09-30",
      metrics: [],
      actions: [],
      blockers: [],
      recent_activity: [],
      ready_handoffs: [],
      active_handoff: null,
    });
    vi.mocked(commands.getAdrs).mockResolvedValue([]);
    vi.mocked(commands.getHandoffs).mockResolvedValue([]);
    vi.mocked(commands.getAgents).mockResolvedValue([]);
  });

  it("renders diagnostics and expands fix prompt when clicking Fix button", async () => {
    vi.mocked(commands.getDiagnostics).mockResolvedValue([
      {
        message: "YAML frontmatter is malformed: missing key",
        severity: "error",
        path: ".lmbrain/tasks/task-001.md",
      },
    ]);

    render(<ProjectPulse />);

    await waitFor(() => {
      expect(screen.getByText("YAML frontmatter is malformed: missing key")).toBeDefined();
    });

    const fixButton = screen.getByText("Fix");
    expect(fixButton).toBeDefined();

    fireEvent.click(fixButton);

    // Prompt content should be visible
    await waitFor(() => {
      expect(screen.getByText("Copy fix prompt")).toBeDefined();
    });

    const copyButton = screen.getByText("Copy fix prompt");
    fireEvent.click(copyButton);

    expect(writeTextMock).toHaveBeenCalled();
    expect(writeTextMock.mock.calls[0][0]).toContain("Please fix the malformed frontmatter");
  });

  it("reveals and copies a manual handoff prompt without writing project state", async () => {
    vi.mocked(commands.getPulseData).mockResolvedValue({
      focus: "M-02",
      milestone: "M-02",
      milestone_progress: 0,
      milestone_due: null,
      metrics: [],
      actions: [{
        title: "Start AGENT-FULLSTACK-DESKTOP on SPEC-016",
        description: "Spec is ready — copy the handoff prompt and launch the agent manually.",
        action_type: "handoff",
        spec_id: "SPEC-016",
        agent: "AGENT-FULLSTACK-DESKTOP",
      }],
      blockers: [],
      recent_activity: [],
      ready_handoffs: [],
      active_handoff: null,
    });
    vi.mocked(commands.getDiagnostics).mockResolvedValue([]);

    render(<ProjectPulse />);

    await waitFor(() => expect(screen.getByText("View prompt")).toBeDefined());
    fireEvent.click(screen.getByText("View prompt"));
    expect((screen.getByLabelText("Handoff prompt for SPEC-016") as HTMLTextAreaElement).value)
      .toContain(".lmbrain/specs/ready/SPEC-016.md");

    fireEvent.click(screen.getByText("Copy prompt"));
    await waitFor(() => expect(screen.getByRole("status").textContent).toBe("Copied to clipboard."));
    expect(writeTextMock).toHaveBeenCalledWith(expect.stringContaining("AGENT-FULLSTACK-DESKTOP"));
  });

  it("opens STATUS.md and ROADMAP.md in the detail modal", async () => {
    vi.mocked(commands.getDiagnostics).mockResolvedValue([]);
    render(<ProjectPulse />);

    await waitFor(() => expect(screen.getByLabelText("Open STATUS.md")).toBeDefined());
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
});
