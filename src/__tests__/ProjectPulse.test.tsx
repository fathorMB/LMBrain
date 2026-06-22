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
        actions: [],
        blockers: [],
        recent_activity: [],
        ready_handoffs: [],
        active_handoff: null,
      },
      handoffs: [],
      adrs: [],
      agents: [],
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
});
