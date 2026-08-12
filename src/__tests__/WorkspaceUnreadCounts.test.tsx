import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { WorkspaceProvider } from "../context/WorkspaceContext";
import { useWorkspace } from "../hooks/useWorkspace";
import type { AppView, WorkspaceSnapshot } from "../types";

const commandMocks = vi.hoisted(() => ({
  getWorkspaceSnapshot: vi.fn(),
  getKitFeedback: vi.fn(),
  listRecentWorkspaces: vi.fn().mockResolvedValue([]),
  openWorkspace: vi.fn(),
  preparePiIntegration: vi.fn(),
  getGitInfo: vi.fn(),
  startWatcher: vi.fn(),
  sessionList: vi.fn(),
}));

vi.mock("../lib/commands", () => commandMocks);
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => undefined),
}));

const WORKSPACE_PATH = "E:/projects/alpha";

interface SnapshotItems {
  specs?: { id: string; status: string; updated: string }[];
  reviews?: { id: string; status: string; updated: string }[];
}

function snapshot(items: SnapshotItems): WorkspaceSnapshot {
  return {
    pulse_data: null,
    specs: items.specs ?? [],
    reviews: items.reviews ?? [],
    debts: [],
    adrs: [],
    agents: [],
    agent_proposals: [],
    mcp_records: [],
    mcp_proposals: [],
    skills: [],
    handoffs: [],
    diagnostics: [],
    project_statistics: null,
  } as unknown as WorkspaceSnapshot;
}

function UnreadProbe() {
  const { state, unreadCounts, loadAllData, navigateTo, openWorkspace } = useWorkspace();
  return (
    <>
      <button type="button" onClick={() => void openWorkspace(WORKSPACE_PATH)}>
        open
      </button>
      <button type="button" onClick={() => void loadAllData()}>
        refresh
      </button>
      {(["taskboard", "reviews", "feedback"] as AppView[]).map((view) => (
        <button key={view} type="button" onClick={() => navigateTo(view)}>
          go-{view}
        </button>
      ))}
      <output data-testid="taskboard">{unreadCounts.taskboard}</output>
      <output data-testid="reviews">{unreadCounts.reviews}</output>
      <output data-testid="feedback">{unreadCounts.feedback}</output>
      <output data-testid="screen">{state.screen}</output>
    </>
  );
}

function count(page: "taskboard" | "reviews" | "feedback"): string {
  return screen.getByTestId(page).textContent ?? "";
}

async function openWorkspace() {
  fireEvent.click(screen.getByText("open"));
  await waitFor(() => expect(screen.getByTestId("screen").textContent).toBe("app"));
}

describe("WorkspaceProvider unread counts", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
    commandMocks.openWorkspace.mockResolvedValue({
      path: WORKSPACE_PATH,
      name: "alpha",
      health: "ok",
    });
    commandMocks.preparePiIntegration.mockResolvedValue({ status: "ready", message: "" });
    commandMocks.getGitInfo.mockResolvedValue({ branch: "main" });
    commandMocks.startWatcher.mockResolvedValue(undefined);
    commandMocks.sessionList.mockResolvedValue([]);
    commandMocks.listRecentWorkspaces.mockResolvedValue([]);
    commandMocks.getKitFeedback.mockResolvedValue({ notes: [] });
    commandMocks.getWorkspaceSnapshot.mockResolvedValue(
      snapshot({
        specs: [
          { id: "SPEC-1", status: "ready", updated: "2026-07-01" },
          { id: "SPEC-2", status: "working", updated: "2026-07-01" },
        ],
        reviews: [{ id: "REV-1", status: "pending", updated: "2026-07-01" }],
      }),
    );
  });

  it("treats a workspace opened for the first time as fully read", async () => {
    render(
      <WorkspaceProvider>
        <UnreadProbe />
      </WorkspaceProvider>,
    );
    await openWorkspace();

    await waitFor(() => expect(count("taskboard")).toBe("0"));
    expect(count("reviews")).toBe("0");
  });

  it("surfaces new and changed items after a watcher refresh, then clears them on visit", async () => {
    render(
      <WorkspaceProvider>
        <UnreadProbe />
      </WorkspaceProvider>,
    );
    await openWorkspace();
    await waitFor(() => expect(count("taskboard")).toBe("0"));

    commandMocks.getWorkspaceSnapshot.mockResolvedValue(
      snapshot({
        specs: [
          { id: "SPEC-1", status: "review", updated: "2026-07-02" },
          { id: "SPEC-2", status: "working", updated: "2026-07-01" },
          { id: "SPEC-3", status: "backlog", updated: "2026-07-02" },
        ],
        reviews: [{ id: "REV-1", status: "pending", updated: "2026-07-01" }],
      }),
    );
    fireEvent.click(screen.getByText("refresh"));

    // SPEC-1 changed status, SPEC-3 is new; the untouched review stays read.
    await waitFor(() => expect(count("taskboard")).toBe("2"));
    expect(count("reviews")).toBe("0");

    fireEvent.click(screen.getByText("go-taskboard"));
    await waitFor(() => expect(count("taskboard")).toBe("0"));
  });

  it("counts kit feedback notes and survives an unavailable feedback report", async () => {
    commandMocks.getKitFeedback.mockResolvedValue({ notes: [] });
    render(
      <WorkspaceProvider>
        <UnreadProbe />
      </WorkspaceProvider>,
    );
    await openWorkspace();
    await waitFor(() => expect(count("feedback")).toBe("0"));

    commandMocks.getKitFeedback.mockResolvedValue({
      notes: [{ id: "NOTE-1", timestamp: "2026-07-02T09:00:00Z", severity: "high" }],
    });
    fireEvent.click(screen.getByText("refresh"));
    await waitFor(() => expect(count("feedback")).toBe("1"));

    commandMocks.getKitFeedback.mockRejectedValue("feedback report unavailable");
    fireEvent.click(screen.getByText("refresh"));
    await waitFor(() => expect(count("feedback")).toBe("0"));
    expect(count("taskboard")).toBe("0");
  });

  it("keeps read state across application restarts", async () => {
    const first = render(
      <WorkspaceProvider>
        <UnreadProbe />
      </WorkspaceProvider>,
    );
    await openWorkspace();
    await waitFor(() => expect(count("taskboard")).toBe("0"));
    first.unmount();

    commandMocks.getWorkspaceSnapshot.mockResolvedValue(
      snapshot({
        specs: [
          { id: "SPEC-1", status: "ready", updated: "2026-07-01" },
          { id: "SPEC-2", status: "working", updated: "2026-07-01" },
        ],
        reviews: [
          { id: "REV-1", status: "pending", updated: "2026-07-01" },
          { id: "REV-2", status: "pending", updated: "2026-07-03" },
        ],
      }),
    );

    render(
      <WorkspaceProvider>
        <UnreadProbe />
      </WorkspaceProvider>,
    );
    await openWorkspace();

    // Previously seen items stay read; only the review added while the
    // application was closed is reported.
    await waitFor(() => expect(count("reviews")).toBe("1"));
    expect(count("taskboard")).toBe("0");
  });
});
