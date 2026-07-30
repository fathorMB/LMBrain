import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { WindowCloseConfirmModal } from "../components/Layout/WindowCloseConfirmModal";

const mocks = vi.hoisted(() => ({
  dispatch: vi.fn(),
  destroy: vi.fn().mockResolvedValue(undefined),
  stopWatcher: vi.fn().mockResolvedValue(undefined),
  sessionKill: vi.fn().mockResolvedValue(undefined),
  sessions: [
    { id: "running-1", label: "Codex fix", status: "running" },
    { id: "exited-1", label: "Old session", status: "exited" },
  ],
}));

vi.mock("../hooks/useWorkspace", () => ({
  useWorkspace: () => ({
    state: { sessions: mocks.sessions },
    dispatch: mocks.dispatch,
  }),
}));
vi.mock("../lib/commands", () => ({
  stopWatcher: mocks.stopWatcher,
  sessionKill: mocks.sessionKill,
}));
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ destroy: mocks.destroy }),
}));

describe("WindowCloseConfirmModal", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.stopWatcher.mockResolvedValue(undefined);
    mocks.sessionKill.mockResolvedValue(undefined);
    mocks.destroy.mockResolvedValue(undefined);
  });

  it("renders an application-styled confirmation for every open session tab", () => {
    render(<WindowCloseConfirmModal />);

    expect(screen.getByRole("dialog", { name: "Close LMBrain?" })).toBeDefined();
    expect(screen.getByText(/2 agent sessions are still open/)).toBeDefined();
    expect(document.activeElement).toBe(screen.getByText("Keep app open"));
  });

  it("cancels without touching sessions or the window", () => {
    render(<WindowCloseConfirmModal />);

    fireEvent.click(screen.getByText("Keep app open"));

    expect(mocks.dispatch).toHaveBeenCalledWith({
      type: "SET_WINDOW_CLOSE_CONFIRM",
      show: false,
    });
    expect(mocks.sessionKill).not.toHaveBeenCalled();
    expect(mocks.destroy).not.toHaveBeenCalled();
  });

  it("stops the watcher and running sessions before destroying the window", async () => {
    render(<WindowCloseConfirmModal />);

    fireEvent.click(screen.getByText("Close LMBrain"));

    await waitFor(() => expect(mocks.destroy).toHaveBeenCalledTimes(1));
    expect(mocks.stopWatcher).toHaveBeenCalledTimes(1);
    expect(mocks.sessionKill).toHaveBeenCalledWith("running-1");
    expect(mocks.sessionKill).not.toHaveBeenCalledWith("exited-1");
  });

  it("destroys the window without retrying cleanup after an explicit force close", async () => {
    mocks.sessionKill.mockRejectedValueOnce(new Error("kill failed"));
    render(<WindowCloseConfirmModal />);

    fireEvent.click(screen.getByText("Close LMBrain"));
    await screen.findByText("Close anyway");

    expect(mocks.destroy).not.toHaveBeenCalled();
    fireEvent.click(screen.getByText("Close anyway"));

    await waitFor(() => expect(mocks.destroy).toHaveBeenCalledTimes(1));
    expect(mocks.sessionKill).toHaveBeenCalledTimes(1);
  });
});
