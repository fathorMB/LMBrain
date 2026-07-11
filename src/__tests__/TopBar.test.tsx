import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { TopBar } from "../components/Layout/TopBar";

const workspace = vi.hoisted(() => ({
  state: {
    view: "insights",
    currentWorkspace: { name: "LMBrain" },
    gitInfo: { branch: "main" },
    watcherActive: true,
  },
  toggleCmdk: vi.fn(),
  refreshWorkspaceData: vi.fn(),
  refreshSessions: vi.fn(),
}));

vi.mock("../hooks/useWorkspace", () => ({
  useWorkspace: () => workspace,
}));

describe("TopBar current-view refresh", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    workspace.state.view = "insights";
    workspace.refreshWorkspaceData.mockResolvedValue(undefined);
    workspace.refreshSessions.mockResolvedValue(undefined);
  });

  it("refreshes shared data before signaling a non-session view remount", async () => {
    let resolveRefresh: (() => void) | undefined;
    workspace.refreshWorkspaceData.mockImplementation(
      () => new Promise<void>((resolve) => { resolveRefresh = resolve; })
    );
    const onViewReload = vi.fn();
    render(<TopBar onViewReload={onViewReload} />);

    const button = screen.getByRole("button", { name: "Refresh current view" });
    fireEvent.click(button);
    fireEvent.click(button);

    expect(workspace.refreshWorkspaceData).toHaveBeenCalledTimes(1);
    expect(button).toHaveProperty("disabled", true);
    expect(onViewReload).not.toHaveBeenCalled();

    resolveRefresh?.();

    await waitFor(() => expect(onViewReload).toHaveBeenCalledTimes(1));
    expect(screen.getByRole("status").textContent).toBe("Updated");
  });

  it("refreshes session metadata without remounting terminal views", async () => {
    workspace.state.view = "sessions";
    const onViewReload = vi.fn();
    render(<TopBar onViewReload={onViewReload} />);

    fireEvent.click(screen.getByRole("button", { name: "Refresh current view" }));

    await waitFor(() => expect(workspace.refreshSessions).toHaveBeenCalledTimes(1));
    expect(onViewReload).not.toHaveBeenCalled();
    expect(screen.getByRole("status").textContent).toBe("Updated");
  });

  it("reports refresh failures without presenting stale data as updated", async () => {
    workspace.refreshWorkspaceData.mockRejectedValue(new Error("backend unavailable"));
    const onViewReload = vi.fn();
    render(<TopBar onViewReload={onViewReload} />);

    fireEvent.click(screen.getByRole("button", { name: "Refresh current view" }));

    await waitFor(() => expect(screen.getByRole("alert").textContent).toBe("Refresh failed"));
    expect(onViewReload).not.toHaveBeenCalled();
    expect(screen.getByRole("button", { name: "Refresh failed. Retry current view" })).toBeDefined();
  });
});
