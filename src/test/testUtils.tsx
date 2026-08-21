import type { ReactElement } from "react";
import { render, type RenderOptions } from "@testing-library/react";
import { vi } from "vitest";
import { WorkspaceContext, type WorkspaceContextValue, type WorkspaceState } from "../context/WorkspaceContext";

export function createMockWorkspaceState(overrides?: Partial<WorkspaceState>): WorkspaceState {
  return {
    screen: "app",
    view: "pulse",
    currentWorkspace: {
      path: "/test/workspace",
      name: "test-workspace",
      kit_version: "5.0.0",
      health: "ok",
      diagnostics: [],
      branch: "main",
      is_clean: true,
      spec_count: 0,
      debt_count: 0,
      task_count: 0,
      decision_count: 0,
      agent_count: 0,
      project_kit_version: "5.0.0",
      bundled_kit_version: "5.0.0",
      bundled_kit_path: "/bundled/kit",
      kit_migration_status: "up-to-date",
    },
    recentWorkspaces: [],
    gitInfo: {
      branch: "main",
      is_clean: true,
      current_commit: "abcdef1",
    },
    pulseData: null,
    specs: [],
    reviews: [],
    debts: [],
    dreams: [],
    adrs: [],
    agents: [],
    agentProposals: [],
    mcpRecords: [],
    mcpProposals: [],
    skills: [],
    handoffs: [],
    diagnostics: [],
    kitFeedbackNotes: [],
    projectStatistics: null,
    wikiTree: null,
    wikiPage: null,
    selectedSpec: null,
    sessions: [],
    activeSessionId: null,
    cmdkOpen: false,
    watcherActive: false,
    dataRefreshing: false,
    loading: false,
    loadingMessage: "Preparing workspace...",
    loadingPath: null,
    workspaceNotice: null,
    error: null,
    detailArtifact: null,
    showExitConfirm: false,
    showWindowCloseConfirm: false,
    ...overrides,
  };
}

export function createMockWorkspaceContext(
  overrides?: {
    state?: Partial<WorkspaceState>;
    actions?: Partial<Omit<WorkspaceContextValue, "state" | "unreadCounts">>;
    unreadCounts?: Record<string, number>;
  },
): WorkspaceContextValue {
  return {
    state: createMockWorkspaceState(overrides?.state),
    unreadCounts: overrides?.unreadCounts ?? {},
    openWorkspace: overrides?.actions?.openWorkspace ?? vi.fn().mockResolvedValue(undefined),
    initializeWorkspaceKit: overrides?.actions?.initializeWorkspaceKit ?? vi.fn().mockResolvedValue(undefined),
    loadAllData: overrides?.actions?.loadAllData ?? vi.fn().mockResolvedValue(undefined),
    refreshWorkspaceData: overrides?.actions?.refreshWorkspaceData ?? vi.fn().mockResolvedValue(undefined),
    navigateTo: overrides?.actions?.navigateTo ?? vi.fn(),
    openSpec: overrides?.actions?.openSpec ?? vi.fn(),
    closeSpecDetail: overrides?.actions?.closeSpecDetail ?? vi.fn(),
    toggleCmdk: overrides?.actions?.toggleCmdk ?? vi.fn(),
    closeCmdk: overrides?.actions?.closeCmdk ?? vi.fn(),
    goToPicker: overrides?.actions?.goToPicker ?? vi.fn().mockResolvedValue(undefined),
    openDetailArtifact: overrides?.actions?.openDetailArtifact ?? vi.fn(),
    triggerLeaveWorkspace: overrides?.actions?.triggerLeaveWorkspace ?? vi.fn(),
    cancelLeaveWorkspace: overrides?.actions?.cancelLeaveWorkspace ?? vi.fn(),
    createSession: overrides?.actions?.createSession ?? vi.fn().mockResolvedValue("test-session-id"),
    closeSession: overrides?.actions?.closeSession ?? vi.fn().mockResolvedValue(undefined),
    refreshSessions: overrides?.actions?.refreshSessions ?? vi.fn().mockResolvedValue(undefined),
    setActiveSession: overrides?.actions?.setActiveSession ?? vi.fn(),
    setShowWindowCloseConfirm: overrides?.actions?.setShowWindowCloseConfirm ?? vi.fn(),
    setWorkspaceNotice: overrides?.actions?.setWorkspaceNotice ?? vi.fn(),
    setWikiPage: overrides?.actions?.setWikiPage ?? vi.fn(),
    setWikiTree: overrides?.actions?.setWikiTree ?? vi.fn(),
    setSessions: overrides?.actions?.setSessions ?? vi.fn(),
  };
}

export function renderWithWorkspace(
  ui: ReactElement,
  options?: {
    context?: Partial<WorkspaceContextValue>;
    state?: Partial<WorkspaceState>;
    renderOptions?: Omit<RenderOptions, "wrapper">;
  },
) {
  const value: WorkspaceContextValue = {
    ...createMockWorkspaceContext({ state: options?.state }),
    ...options?.context,
  };

  return {
    ...render(
      <WorkspaceContext.Provider value={value}>
        {ui}
      </WorkspaceContext.Provider>,
      options?.renderOptions,
    ),
    context: value,
  };
}
