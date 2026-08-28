import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { WorkspaceProvider } from "../context/WorkspaceContext";
import { useWorkspace } from "../hooks/useWorkspace";
import type { WorkspaceSnapshot } from "../types";

const commandMocks = vi.hoisted(() => ({
  getWorkspaceSnapshot: vi.fn(),
  listRecentWorkspaces: vi.fn().mockResolvedValue([]),
}));

vi.mock("../lib/commands", () => commandMocks);
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => undefined),
}));

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((resolvePromise) => {
    resolve = resolvePromise;
  });
  return { promise, resolve };
}

function snapshot(totalSpecs: number): WorkspaceSnapshot {
  return {
    pulse_data: {
      focus: null,
      milestone: null,
      milestone_progress: null,
      milestone_due: null,
      metrics: [],
      actions: [],
      blockers: [],
      recent_activity: [],
      ready_handoffs: [],
      active_handoff: null,
    },
    specs: [],
    reviews: [],
    debts: [],
    adrs: [],
    agents: [],
    agent_proposals: [],
    mcp_records: [],
    mcp_proposals: [],
    skills: [],
    handoffs: [],
    diagnostics: [],
    project_statistics: {
      artifact_families: [],
      spec_flow: {
        total_specs: totalSpecs,
        done_specs: 0,
        open_specs: totalSpecs,
        done_ratio: 0,
        by_status: [],
        by_priority: [],
        by_area: [],
      },
      review_quality: {
        total_reviews: 0,
        total_review_passes: 0,
        remediation_cycles: 0,
        escalation_count: 0,
        takeover_count: 0,
        lifecycle_known_reviews: 0,
        lifecycle_coverage: 0,
        reviewed_specs: 0,
        accepted_reviews: 0,
        changes_requested_reviews: 0,
        blocked_reviews: 0,
        superseded_reviews: 0,
        reviews_without_spec: 0,
        reviews_without_created: 0,
        specs_with_changes_requested: 0,
        specs_with_multiple_changes_requested: 0,
        change_request_rate: 0,
        first_pass_eligible_specs: 0,
        first_pass_accepted_specs: 0,
        first_pass_acceptance_rate: 0,
        outcome_balance: {
          done_specs: 0, eligible_specs: 0, first_pass_specs: 0, remediation_required_specs: 0,
          excluded_specs: 0, excluded_no_review: 0, excluded_unknown_history: 0,
          excluded_inconsistent_history: 0, entries: [], entries_truncated: false,
        },
        average_reviews_per_reviewed_spec: 0,
        by_area: [],
        by_agent: [],
        trend: [],
      },
      diagnostics: { total: 0, warnings: 0, errors: 0, by_family: [] },
    },
  };
}

function RefreshProbe() {
  const { state, loadAllData } = useWorkspace();
  return (
    <>
      <button type="button" onClick={() => void loadAllData()}>
        refresh
      </button>
      <output>{state.projectStatistics?.spec_flow.total_specs ?? "none"}</output>
    </>
  );
}

describe("WorkspaceProvider refresh pipeline", () => {
  it("uses one snapshot command and commits only the coalesced trailing result", async () => {
    const first = deferred<WorkspaceSnapshot>();
    const trailing = deferred<WorkspaceSnapshot>();
    commandMocks.getWorkspaceSnapshot
      .mockReset()
      .mockReturnValueOnce(first.promise)
      .mockReturnValueOnce(trailing.promise);

    render(
      <WorkspaceProvider>
        <RefreshProbe />
      </WorkspaceProvider>,
    );

    fireEvent.click(screen.getByText("refresh"));
    fireEvent.click(screen.getByText("refresh"));
    fireEvent.click(screen.getByText("refresh"));
    expect(commandMocks.getWorkspaceSnapshot).toHaveBeenCalledTimes(1);

    first.resolve(snapshot(1));
    await waitFor(() =>
      expect(commandMocks.getWorkspaceSnapshot).toHaveBeenCalledTimes(2),
    );
    expect(screen.getByText("none")).toBeDefined();

    trailing.resolve(snapshot(2));
    await waitFor(() => expect(screen.getByText("2")).toBeDefined());
    expect(commandMocks.getWorkspaceSnapshot).toHaveBeenCalledTimes(2);
  });
});
