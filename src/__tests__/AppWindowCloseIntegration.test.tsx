import { cleanup, render, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ReactNode } from "react";
import type { SessionInfo } from "../types";
import App from "../App";

type CloseEvent = { preventDefault: () => void };
type CloseHandler = (event: CloseEvent) => void | Promise<void>;

const mocks = vi.hoisted(() => ({
  closeHandler: undefined as CloseHandler | undefined,
  destroy: vi.fn().mockResolvedValue(undefined),
  setSessions: vi.fn(),
  setShowWindowCloseConfirm: vi.fn(),
  sessionList: vi.fn<() => Promise<SessionInfo[]>>(),
  unlisten: vi.fn(),
}));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    destroy: mocks.destroy,
    onCloseRequested: vi.fn(async (handler: CloseHandler) => {
      mocks.closeHandler = handler;
      return mocks.unlisten;
    }),
  }),
}));

vi.mock("../lib/commands", () => ({
  sessionList: mocks.sessionList,
}));

vi.mock("../context/WorkspaceContext", () => ({
  WorkspaceProvider: ({ children }: { children: ReactNode }) => children,
}));

vi.mock("../hooks/useWorkspace", () => ({
  useWorkspace: () => ({
    toggleCmdk: vi.fn(),
    closeCmdk: vi.fn(),
    setSessions: mocks.setSessions,
    setShowWindowCloseConfirm: mocks.setShowWindowCloseConfirm,
    state: { sessions: [] },
  }),
}));

vi.mock("../components/Layout/AppShell", () => ({
  AppShell: () => null,
}));

const runningSession: SessionInfo = {
  id: "running-1",
  label: "Running session",
  host: "codex",
  route: "native",
  model: null,
  status: "running",
  exit_code: null,
};

describe("App window close integration", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.closeHandler = undefined;
  });

  afterEach(() => {
    cleanup();
  });

  it("prevents native destruction before awaiting the backend session list", async () => {
    let resolveSessions: (sessions: SessionInfo[]) => void = () => {};
    mocks.sessionList.mockReturnValue(
      new Promise((resolve) => {
        resolveSessions = resolve;
      }),
    );
    render(<App />);
    await waitFor(() => expect(mocks.closeHandler).toBeDefined());

    const preventDefault = vi.fn();
    const request = mocks.closeHandler?.({ preventDefault });

    expect(preventDefault).toHaveBeenCalledTimes(1);
    expect(mocks.destroy).not.toHaveBeenCalled();

    resolveSessions([runningSession]);
    await request;

    expect(mocks.setSessions).toHaveBeenCalledWith([runningSession]);
    expect(mocks.setShowWindowCloseConfirm).toHaveBeenCalledWith(true);
    expect(mocks.destroy).not.toHaveBeenCalled();
  });

  it("explicitly destroys only after confirming the backend has no sessions", async () => {
    mocks.sessionList.mockResolvedValue([]);
    render(<App />);
    await waitFor(() => expect(mocks.closeHandler).toBeDefined());

    const preventDefault = vi.fn();
    await mocks.closeHandler?.({ preventDefault });

    expect(preventDefault).toHaveBeenCalledTimes(1);
    expect(mocks.destroy).toHaveBeenCalledTimes(1);
    expect(mocks.setShowWindowCloseConfirm).not.toHaveBeenCalledWith(true);
  });
});
