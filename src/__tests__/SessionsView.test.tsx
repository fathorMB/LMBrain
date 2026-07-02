import { describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { SessionsView } from "../components/Sessions/SessionsView";
import type { SessionInfo } from "../types";

const mockWorkspace = vi.hoisted(() => ({
  closeSession: vi.fn(),
  createSession: vi.fn(),
  setActiveSession: vi.fn(),
}));

const runningSession: SessionInfo = {
  id: "session-1",
  label: "Frontend debugging",
  mode: "claude",
  model: null,
  status: "running",
  exit_code: null,
};

const exitedSession: SessionInfo = {
  id: "session-2",
  label: "Backend work",
  mode: "codex",
  model: null,
  status: "exited",
  exit_code: 0,
};

const failedSession: SessionInfo = {
  id: "session-3",
  label: "Failed task",
  mode: "ollama",
  model: "llama3",
  status: "exited",
  exit_code: 1,
};

vi.mock("../lib/commands", () => ({
  listOllamaModels: vi.fn(),
}));

vi.mock("../hooks/useWorkspace", () => ({
  useWorkspace: () => ({
    state: {
      sessions: [runningSession, exitedSession, failedSession],
      activeSessionId: "session-1",
    },
    createSession: mockWorkspace.createSession,
    closeSession: mockWorkspace.closeSession,
    setActiveSession: mockWorkspace.setActiveSession,
  }),
}));

vi.mock("../components/Sessions/SessionTerminal", () => ({
  SessionTerminal: ({ sessionId }: { sessionId: string }) => (
    <div data-testid={`terminal-${sessionId}`} />
  ),
}));

describe("SessionsView", () => {
  it("renders tabs for each session", () => {
    render(<SessionsView active />);

    expect(screen.getByText("Frontend debugging")).toBeDefined();
    expect(screen.getByText("Backend work")).toBeDefined();
    expect(screen.getByText("Failed task")).toBeDefined();
  });

  it("shows the active tab with a terminal", () => {
    render(<SessionsView active />);

    // The active session's terminal should be rendered
    expect(screen.getByTestId("terminal-session-1")).toBeDefined();
  });

  it("switching tabs calls setActiveSession", () => {
    render(<SessionsView active />);

    fireEvent.click(screen.getByText("Backend work"));
    expect(mockWorkspace.setActiveSession).toHaveBeenCalledWith("session-2");
  });

  it("closing a tab calls closeSession", () => {
    render(<SessionsView active />);

    // Each tab has a close button — find the one for "Backend work"
    const backendLabel = screen.getByText("Backend work");
    const tabOuter = backendLabel.parentElement;
    expect(tabOuter).not.toBeNull();
    const closeButtons = tabOuter!.querySelectorAll("button");
    expect(closeButtons.length).toBeGreaterThanOrEqual(1);
    // The close button is the last button in the tab
    const closeButton = closeButtons[closeButtons.length - 1];
    fireEvent.click(closeButton);
    expect(mockWorkspace.closeSession).toHaveBeenCalledWith("session-2");
  });

  it("shows exit status for exited sessions", () => {
    render(<SessionsView active />);

    expect(screen.getByText("exit 0")).toBeDefined();
    expect(screen.getByText("exit 1")).toBeDefined();
  });

  it("shows mode for running sessions", () => {
    render(<SessionsView active />);

    // Running sessions show their mode
    expect(screen.getByText("claude")).toBeDefined();
  });

  it("opens the new-session modal when clicking New session", () => {
    render(<SessionsView active />);

    fireEvent.click(screen.getByText("New session"));
    expect(screen.getByRole("dialog", { name: "Start session" })).toBeDefined();
  });

  it("shows the new-session modal above the tab workspace", () => {
    render(<SessionsView active />);

    fireEvent.click(screen.getByText("New session"));
    const dialog = screen.getByRole("dialog", { name: "Start session" });
    const overlay = dialog.parentElement;

    expect(overlay).not.toBeNull();
    expect(Number(overlay?.style.zIndex)).toBe(100);
  });
});
