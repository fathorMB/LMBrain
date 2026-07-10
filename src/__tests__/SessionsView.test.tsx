import { beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { SessionsView } from "../components/Sessions/SessionsView";
import { listOllamaModels } from "../lib/commands";
import type { SessionInfo } from "../types";

const mockWorkspace = vi.hoisted(() => ({
  closeSession: vi.fn(),
  createSession: vi.fn(),
  setActiveSession: vi.fn(),
}));

const runningSession: SessionInfo = {
  id: "session-1",
  label: "Frontend debugging",
  host: "claude",
  route: "native",
  model: null,
  status: "running",
  exit_code: null,
};

const exitedSession: SessionInfo = {
  id: "session-2",
  label: "Backend work",
  host: "codex",
  route: "native",
  model: null,
  status: "exited",
  exit_code: 0,
};

const failedSession: SessionInfo = {
  id: "session-3",
  label: "Failed task",
  host: "claude",
  route: "ollama",
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
  SessionTerminal: ({ sessionId, active }: { sessionId: string; active: boolean }) => (
    <div data-testid={`terminal-${sessionId}`} data-active={String(active)} />
  ),
}));

describe("SessionsView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(listOllamaModels).mockResolvedValue([
      { name: "qwen3.5:cloud", cloud: true, capabilities: ["tools"] },
    ]);
    mockWorkspace.createSession.mockResolvedValue("session-pi");
  });

  it("renders tabs for each session", () => {
    render(<SessionsView active />);

    expect(screen.getByText("Frontend debugging")).toBeDefined();
    expect(screen.getByText("Backend work")).toBeDefined();
    expect(screen.getByText("Failed task")).toBeDefined();
  });

  it("shows the active tab with a terminal", () => {
    render(<SessionsView active />);

    expect(screen.getByTestId("terminal-session-1").dataset.active).toBe("true");
  });

  it("keeps inactive session terminals mounted to preserve scrollback", () => {
    render(<SessionsView active />);

    expect(screen.getByTestId("terminal-session-1").dataset.active).toBe("true");
    expect(screen.getByTestId("terminal-session-2").dataset.active).toBe("false");
    expect(screen.getByTestId("terminal-session-3").dataset.active).toBe("false");
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

  it("loads Ollama models when Pi is selected and submits the Pi route", async () => {
    render(<SessionsView active />);

    fireEvent.click(screen.getByRole("button", { name: "New session" }));
    fireEvent.click(screen.getByRole("button", { name: "Pi" }));

    await waitFor(() => expect(listOllamaModels).toHaveBeenCalledTimes(1));
    expect(await screen.findByRole("option", { name: "qwen3.5:cloud" })).toBeDefined();

    fireEvent.click(screen.getByRole("button", { name: "Start session" }));
    await waitFor(() =>
      expect(mockWorkspace.createSession).toHaveBeenCalledWith({
        host: "pi",
        route: "ollama",
        model: "qwen3.5:cloud",
        codex_bin: undefined,
        label: "",
      })
    );
  });

  it("keeps the modal open and shows Pi launch errors", async () => {
    mockWorkspace.createSession.mockRejectedValueOnce("Pi MCP prerequisite is missing");
    render(<SessionsView active />);

    fireEvent.click(screen.getByRole("button", { name: "New session" }));
    fireEvent.click(screen.getByRole("button", { name: "Pi" }));
    await screen.findByRole("option", { name: "qwen3.5:cloud" });
    fireEvent.click(screen.getByRole("button", { name: "Start session" }));

    expect(await screen.findByText("Pi MCP prerequisite is missing")).toBeDefined();
    expect(screen.getByRole("dialog", { name: "Start session" })).toBeDefined();
  });
});
