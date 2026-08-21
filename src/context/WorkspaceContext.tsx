import {
  createContext,
  useReducer,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type Dispatch,
  type ReactNode,
} from "react";
import { listen } from "@tauri-apps/api/event";
import type {
  Adr,
  AppView,
  DetailArtifact,
  FileEvent,
  GitInfo,
  Handoff,
  Debt,
  Dream,
  KitDiagnostic,
  KitFeedbackNote,
  McpProposal,
  McpRecord,
  PulseData,
  ProjectStatistics,
  Review,
  SessionInfo,
  AgentHost,
  ModelRoute,
  Skill,
  Spec,
  WikiPage,
  WikiTree,
  WorkspaceInfo,
  WorkspaceSummary,
  AgentProfile,
  AgentProposal,
} from "../types";
import * as commands from "../lib/commands";
import { createTrailingRefreshCoordinator } from "../lib/refreshCoordinator";
import {
  collectPageItems,
  countAllUnread,
  isUnreadPage,
  loadReadState,
  markItemRead,
  markItemsRead,
  saveReadState,
  seedAllRead,
  toUnreadItems,
  type PageItems,
  type ReadState,
  type UnreadPage,
  type UnreadSource,
} from "../lib/unreadState";

export interface WorkspaceState {
  screen: "picker" | "app";
  view: AppView;
  currentWorkspace: WorkspaceInfo | null;
  recentWorkspaces: WorkspaceSummary[];
  gitInfo: GitInfo | null;
  pulseData: PulseData | null;
  specs: Spec[];
  reviews: Review[];
  debts: Debt[];
  dreams: Dream[];
  adrs: Adr[];
  agents: AgentProfile[];
  agentProposals: AgentProposal[];
  mcpRecords: McpRecord[];
  mcpProposals: McpProposal[];
  skills: Skill[];
  handoffs: Handoff[];
  diagnostics: KitDiagnostic[];
  kitFeedbackNotes: KitFeedbackNote[];
  projectStatistics: ProjectStatistics | null;
  wikiTree: WikiTree | null;
  wikiPage: WikiPage | null;
  selectedSpec: Spec | null;
  sessions: SessionInfo[];
  activeSessionId: string | null;
  cmdkOpen: boolean;
  watcherActive: boolean;
  dataRefreshing: boolean;
  loading: boolean;
  loadingMessage: string;
  loadingPath: string | null;
  workspaceNotice: string | null;
  error: string | null;
  detailArtifact: DetailArtifact | null;
  showExitConfirm: boolean;
  showWindowCloseConfirm: boolean;
}

export type Action =
  | { type: "MERGE_DATA"; data: Partial<WorkspaceState> }
  | { type: "SET_SCREEN"; screen: "picker" | "app" }
  | { type: "SET_VIEW"; view: AppView }
  | { type: "SET_WORKSPACE"; info: WorkspaceInfo }
  | { type: "SET_RECENT"; workspaces: WorkspaceSummary[] }
  | { type: "SET_GIT_INFO"; info: GitInfo }
  | { type: "SET_PULSE"; data: PulseData }
  | { type: "SET_SPECS"; specs: Spec[] }
  | { type: "SET_REVIEWS"; reviews: Review[] }
  | { type: "SET_DEBTS"; debts: Debt[] }
  | { type: "SET_ADRS"; adrs: Adr[] }
  | { type: "SET_AGENTS"; agents: AgentProfile[] }
  | { type: "SET_AGENT_PROPOSALS"; proposals: AgentProposal[] }
  | { type: "SET_MCP_RECORDS"; records: McpRecord[] }
  | { type: "SET_MCP_PROPOSALS"; proposals: McpProposal[] }
  | { type: "SET_SKILLS"; skills: Skill[] }
  | { type: "SET_HANDOFFS"; handoffs: Handoff[] }
  | { type: "SET_WIKI_TREE"; tree: WikiTree }
  | { type: "SET_WIKI_PAGE"; page: WikiPage | null }
  | { type: "SET_SELECTED_SPEC"; spec: Spec | null }
  | { type: "CLOSE_SPEC_DETAIL" }
  | { type: "SET_CMDK"; open: boolean }
  | { type: "SET_WATCHER"; active: boolean }
  | { type: "SET_DATA_REFRESHING"; refreshing: boolean }
  | { type: "SET_LOADING"; loading: boolean; message?: string; path?: string | null }
  | { type: "SET_WORKSPACE_NOTICE"; notice: string | null }
  | { type: "SET_ERROR"; error: string | null }
  | { type: "SET_DETAIL_ARTIFACT"; artifact: DetailArtifact | null }
  | { type: "SET_EXIT_CONFIRM"; show: boolean }
  | { type: "SET_WINDOW_CLOSE_CONFIRM"; show: boolean }
  | { type: "SET_SESSIONS"; sessions: SessionInfo[] }
  | { type: "ADD_SESSION"; session: SessionInfo }
  | { type: "UPDATE_SESSION"; id: string; patch: Partial<SessionInfo> }
  | { type: "REMOVE_SESSION"; id: string }
  | { type: "SET_ACTIVE_SESSION"; id: string | null }
  | { type: "CLEAR_SESSIONS" };

const initialState: WorkspaceState = {
  screen: "picker",
  view: "pulse",
  currentWorkspace: null,
  recentWorkspaces: [],
  gitInfo: null,
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
};

// ─── Session reducer (exported for testing) ───────────────────────
export interface SessionState {
  sessions: SessionInfo[];
  activeSessionId: string | null;
}

export type SessionAction =
  | { type: "SET_SESSIONS"; sessions: SessionInfo[] }
  | { type: "ADD_SESSION"; session: SessionInfo }
  | { type: "UPDATE_SESSION"; id: string; patch: Partial<SessionInfo> }
  | { type: "REMOVE_SESSION"; id: string }
  | { type: "SET_ACTIVE_SESSION"; id: string | null }
  | { type: "CLEAR_SESSIONS" };

// eslint-disable-next-line react-refresh/only-export-components
export function sessionReducer(state: SessionState, action: SessionAction): SessionState {
  switch (action.type) {
    case "SET_SESSIONS": {
      const activeExists = state.activeSessionId && action.sessions.some((s) => s.id === state.activeSessionId);
      return {
        ...state,
        sessions: action.sessions,
        activeSessionId: activeExists ? state.activeSessionId : (action.sessions[0]?.id ?? null),
      };
    }
    case "ADD_SESSION":
      return {
        ...state,
        sessions: [...state.sessions, action.session],
        activeSessionId: action.session.id,
      };
    case "UPDATE_SESSION":
      return {
        ...state,
        sessions: state.sessions.map((session) =>
          session.id === action.id ? { ...session, ...action.patch } : session
        ),
      };
    case "REMOVE_SESSION": {
      const remaining = state.sessions.filter((s) => s.id !== action.id);
      const idx = state.sessions.findIndex((s) => s.id === action.id);
      const nextActive = state.activeSessionId === action.id
        ? (remaining.length > 0 ? remaining[Math.max(0, idx - 1)]?.id ?? null : null)
        : state.activeSessionId;
      return { ...state, sessions: remaining, activeSessionId: nextActive };
    }
    case "SET_ACTIVE_SESSION":
      return { ...state, activeSessionId: action.id };
    case "CLEAR_SESSIONS":
      return { ...state, sessions: [], activeSessionId: null };
  }
}

/**
 * Kit feedback lives outside the workspace snapshot. It is read alongside it so
 * the sidebar can count unread notes, and it never fails the refresh: an
 * unavailable or malformed report simply contributes no items.
 */
async function fetchKitFeedbackNotes(): Promise<KitFeedbackNote[]> {
  try {
    const report = await commands.getKitFeedback();
    return Array.isArray(report?.notes) ? report.notes : [];
  } catch {
    return [];
  }
}

function reducer(state: WorkspaceState, action: Action): WorkspaceState {
  switch (action.type) {
    case "MERGE_DATA":
      return { ...state, ...action.data };
    case "SET_SCREEN":
      return { ...state, screen: action.screen };
    case "SET_VIEW":
      return { ...state, view: action.view };
    case "SET_WORKSPACE":
      return { ...state, currentWorkspace: action.info };
    case "SET_RECENT":
      return { ...state, recentWorkspaces: action.workspaces };
    case "SET_GIT_INFO":
      return { ...state, gitInfo: action.info };
    case "SET_PULSE":
      return { ...state, pulseData: action.data };
    case "SET_SPECS":
      return { ...state, specs: action.specs };
    case "SET_REVIEWS":
      return { ...state, reviews: action.reviews };
    case "SET_DEBTS":
      return { ...state, debts: action.debts };
    case "SET_ADRS":
      return { ...state, adrs: action.adrs };
    case "SET_AGENTS":
      return { ...state, agents: action.agents };
    case "SET_AGENT_PROPOSALS":
      return { ...state, agentProposals: action.proposals };
    case "SET_MCP_RECORDS":
      return { ...state, mcpRecords: action.records };
    case "SET_MCP_PROPOSALS":
      return { ...state, mcpProposals: action.proposals };
    case "SET_SKILLS":
      return { ...state, skills: action.skills };
    case "SET_HANDOFFS":
      return { ...state, handoffs: action.handoffs };
    case "SET_WIKI_TREE":
      return { ...state, wikiTree: action.tree };
    case "SET_WIKI_PAGE":
      return { ...state, wikiPage: action.page };
    case "SET_SELECTED_SPEC":
      return { ...state, selectedSpec: action.spec };
    case "CLOSE_SPEC_DETAIL":
      return {
        ...state,
        view: "taskboard",
        selectedSpec: null,
        cmdkOpen: false,
      };
    case "SET_CMDK":
      return { ...state, cmdkOpen: action.open };
    case "SET_WATCHER":
      return { ...state, watcherActive: action.active };
    case "SET_DATA_REFRESHING":
      return { ...state, dataRefreshing: action.refreshing };
    case "SET_LOADING":
      return {
        ...state,
        loading: action.loading,
        loadingMessage: action.message ?? state.loadingMessage,
        loadingPath: action.loading ? (action.path ?? state.loadingPath) : null,
      };
    case "SET_WORKSPACE_NOTICE":
      return { ...state, workspaceNotice: action.notice };
    case "SET_ERROR":
      return { ...state, error: action.error };
    case "SET_DETAIL_ARTIFACT":
      return { ...state, detailArtifact: action.artifact };
    case "SET_EXIT_CONFIRM":
      return { ...state, showExitConfirm: action.show };
    case "SET_WINDOW_CLOSE_CONFIRM":
      return { ...state, showWindowCloseConfirm: action.show };
    case "SET_SESSIONS":
    case "ADD_SESSION":
    case "UPDATE_SESSION":
    case "REMOVE_SESSION":
    case "SET_ACTIVE_SESSION":
    case "CLEAR_SESSIONS":
      return { ...state, ...sessionReducer(state, action as unknown as SessionAction) };
    default:
      return state;
  }
}

export interface WorkspaceContextValue {
  state: WorkspaceState;
  /** Unread item count per workspace page, keyed by view. */
  unreadCounts: Record<UnreadPage, number>;
  openWorkspace: (path: string) => Promise<void>;
  initializeWorkspaceKit: (path: string) => Promise<void>;
  loadAllData: () => Promise<void>;
  refreshWorkspaceData: () => Promise<void>;
  navigateTo: (view: AppView) => void;
  openSpec: (spec: Spec) => void;
  closeSpecDetail: () => void;
  toggleCmdk: () => void;
  closeCmdk: () => void;
  goToPicker: () => Promise<void>;
  openDetailArtifact: (artifact: DetailArtifact | null) => void;
  triggerLeaveWorkspace: () => void;
  cancelLeaveWorkspace: () => void;
  createSession: (request: { host: AgentHost; route: ModelRoute; model?: string; label?: string; codex_bin?: string }) => Promise<string>;
  closeSession: (id: string) => Promise<void>;
  refreshSessions: () => Promise<void>;
  setActiveSession: (id: string | null) => void;
  setSessions: (sessions: SessionInfo[]) => void;
  setShowWindowCloseConfirm: (show: boolean) => void;
  setWorkspaceNotice: (notice: string | null) => void;
  setWikiPage: (page: WikiPage | null) => void;
  setWikiTree: (tree: WikiTree) => void;
}

// eslint-disable-next-line react-refresh/only-export-components
export const WorkspaceContext = createContext<WorkspaceContextValue | null>(null);

export function WorkspaceProvider({ children }: { children: ReactNode }) {
  const [state, dispatch] = useReducer(reducer, initialState);
  const dataRefreshRequestCount = useRef(0);

  // ─── Unread read-state (sidebar badges) ────────────────────────
  const [readState, setReadState] = useState<ReadState>({});
  const readStateRef = useRef<ReadState>({});
  const readPathRef = useRef<string | null>(null);
  const seedPendingRef = useRef(false);
  const viewRef = useRef<AppView>(initialState.view);
  const pageItemsRef = useRef<PageItems>(collectPageItems(null));

  const applyReadState = useCallback((next: ReadState, persist: boolean) => {
    if (next === readStateRef.current) return;
    readStateRef.current = next;
    setReadState(next);
    if (persist && readPathRef.current) saveReadState(readPathRef.current, next);
  }, []);

  /**
   * Restore the read state of a workspace being opened. When nothing is stored,
   * a baseline is scheduled for the first load so an existing project does not
   * report its entire backlog as unread.
   */
  const beginReadStateForWorkspace = useCallback(
    (path: string) => {
      const stored = loadReadState(path);
      readPathRef.current = path;
      seedPendingRef.current = stored === null;
      applyReadState(stored ?? {}, false);
    },
    [applyReadState],
  );

  /** Reconcile read state against freshly loaded workspace data. */
  const reconcileReadState = useCallback(
    (source: UnreadSource) => {
      if (!readPathRef.current) return;
      const items = collectPageItems(source);
      pageItemsRef.current = items;
      if (seedPendingRef.current) {
        seedPendingRef.current = false;
        applyReadState(seedAllRead(items), true);
        return;
      }
      // Items updated while their page is on screen are being looked at.
      const view = viewRef.current;
      if (isUnreadPage(view)) {
        applyReadState(markItemsRead(readStateRef.current, view, items[view]), true);
      }
    },
    [applyReadState],
  );

  const markPageRead = useCallback(
    (view: AppView) => {
      if (!readPathRef.current || seedPendingRef.current || !isUnreadPage(view)) return;
      const page: UnreadPage = view;
      applyReadState(markItemsRead(readStateRef.current, page, pageItemsRef.current[page]), true);
    },
    [applyReadState],
  );

  const refreshSessions = useCallback(async () => {
    const infos = await commands.sessionList();
    dispatch({ type: "SET_SESSIONS", sessions: infos });
  }, []);

  const fetchWorkspaceData = useMemo(
    () =>
      createTrailingRefreshCoordinator<Partial<WorkspaceState>>(async () => {
        const [snapshot, kitFeedbackNotes] = await Promise.all([
          commands.getWorkspaceSnapshot(),
          fetchKitFeedbackNotes(),
        ]);
        return {
          kitFeedbackNotes,
          pulseData: snapshot.pulse_data,
          specs: snapshot.specs,
          reviews: snapshot.reviews,
          debts: snapshot.debts,
          dreams: snapshot.dreams,
          adrs: snapshot.adrs,
          agents: snapshot.agents,
          agentProposals: snapshot.agent_proposals,
          mcpRecords: snapshot.mcp_records,
          mcpProposals: snapshot.mcp_proposals,
          skills: snapshot.skills,
          handoffs: snapshot.handoffs,
          diagnostics: snapshot.diagnostics,
          projectStatistics: snapshot.project_statistics,
        };
      }),
    [],
  );

  const loadAllDataInternal = useCallback(async () => {
    dataRefreshRequestCount.current += 1;
    dispatch({ type: "SET_DATA_REFRESHING", refreshing: true });
    try {
      const data = await fetchWorkspaceData();
      dispatch({ type: "MERGE_DATA", data });
      reconcileReadState(data);
    } catch (err) {
      console.error("Failed to load data:", err);
    } finally {
      dataRefreshRequestCount.current -= 1;
      if (dataRefreshRequestCount.current === 0) {
        dispatch({ type: "SET_DATA_REFRESHING", refreshing: false });
      }
    }
  }, [fetchWorkspaceData, reconcileReadState]);

  const loadAllData = useCallback(async () => {
    await loadAllDataInternal();
  }, [loadAllDataInternal]);

  const refreshWorkspaceData = useCallback(async () => {
    const [data, gitInfo, wikiPage] = await Promise.all([
      fetchWorkspaceData(),
      commands.getGitInfo(),
      state.wikiPage ? commands.getWikiPage(state.wikiPage.path) : Promise.resolve(null),
    ]);
    const selectedSpec = state.selectedSpec
      ? data.specs?.find((spec) => spec.id === state.selectedSpec?.id) ?? null
      : null;
    dispatch({
      type: "MERGE_DATA",
      data: {
        ...data,
        gitInfo,
        selectedSpec,
        wikiPage,
      },
    });
    reconcileReadState(data);
  }, [fetchWorkspaceData, reconcileReadState, state.selectedSpec, state.wikiPage]);

  const pageItems = useMemo(
    () =>
      collectPageItems({
        specs: state.specs,
        reviews: state.reviews,
        debts: state.debts,
        adrs: state.adrs,
        agents: state.agents,
        agentProposals: state.agentProposals,
        mcpRecords: state.mcpRecords,
        mcpProposals: state.mcpProposals,
        skills: state.skills,
        kitFeedbackNotes: state.kitFeedbackNotes,
      }),
    [
      state.specs,
      state.reviews,
      state.debts,
      state.adrs,
      state.agents,
      state.agentProposals,
      state.mcpRecords,
      state.mcpProposals,
      state.skills,
      state.kitFeedbackNotes,
    ],
  );

  // Keep the values the read-state callbacks need available outside of render.
  useEffect(() => {
    viewRef.current = state.view;
  }, [state.view]);

  useEffect(() => {
    pageItemsRef.current = pageItems;
  }, [pageItems]);

  const unreadCounts = useMemo(() => countAllUnread(pageItems, readState), [pageItems, readState]);

  useEffect(() => {
    commands.listRecentWorkspaces().then((workspaces) => {
      dispatch({ type: "SET_RECENT", workspaces });
    });
  }, []);

  useEffect(() => {
    const unlisten = listen<FileEvent>("file-changed", () => {
      loadAllData();
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [loadAllData, state.currentWorkspace]);

  useEffect(() => {
    const unlisten = listen<{ id: string; code: number | null }>("session-exit", (event) => {
      dispatch({
        type: "UPDATE_SESSION",
        id: event.payload.id,
        patch: {
          status: "exited",
          exit_code: event.payload.code,
        },
      });
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const openWorkspace = useCallback(
    async (path: string) => {
      dispatch({
        type: "SET_LOADING",
        loading: true,
        message: "Validating workspace...",
        path,
      });
      dispatch({ type: "SET_ERROR", error: null });
      dispatch({ type: "SET_WORKSPACE_NOTICE", notice: null });
      try {
        const info = await commands.openWorkspace(path);
        dispatch({ type: "SET_WORKSPACE", info });

        if (info.health === "none") {
          return;
        }

        beginReadStateForWorkspace(info.path ?? path);

        dispatch({
          type: "SET_LOADING",
          loading: true,
          message: "Preparing Pi agent integration...",
        });
        try {
          const preparation = await commands.preparePiIntegration();
          if (preparation.status === "unavailable") {
            dispatch({ type: "SET_WORKSPACE_NOTICE", notice: preparation.message });
          }
        } catch (error) {
          dispatch({
            type: "SET_WORKSPACE_NOTICE",
            notice:
              typeof error === "string"
                ? error
                : "Pi integration could not be prepared; the workspace remains available.",
          });
        }

        dispatch({ type: "SET_LOADING", loading: true, message: "Reading Git state..." });
        try {
          const gitInfo = await commands.getGitInfo();
          dispatch({ type: "SET_GIT_INFO", info: gitInfo });
        } catch {
          // Git info is optional
        }

        const workspaces = await commands.listRecentWorkspaces();
        dispatch({ type: "SET_RECENT", workspaces });

        dispatch({ type: "SET_LOADING", loading: true, message: "Loading project data..." });
        await loadAllDataInternal();
        dispatch({ type: "SET_LOADING", loading: true, message: "Restoring sessions..." });
        await refreshSessions();

        dispatch({ type: "SET_LOADING", loading: true, message: "Starting file watcher..." });
        try {
          await commands.startWatcher();
          dispatch({ type: "SET_WATCHER", active: true });
        } catch {
          // Watcher is optional
        }

        dispatch({ type: "SET_LOADING", loading: true, message: "Opening Project Pulse..." });
        dispatch({ type: "SET_SCREEN", screen: "app" });
      } catch (err) {
        dispatch({
          type: "SET_ERROR",
          error: typeof err === "string" ? err : "Failed to open workspace",
        });
      } finally {
        dispatch({ type: "SET_LOADING", loading: false });
      }
    },
    [beginReadStateForWorkspace, loadAllDataInternal, refreshSessions]
  );

  const initializeWorkspaceKit = useCallback(
    async (path: string) => {
      dispatch({
        type: "SET_LOADING",
        loading: true,
        message: "Initializing LMBrain kit...",
        path,
      });
      dispatch({ type: "SET_ERROR", error: null });
      try {
        await commands.initializeWorkspaceKit(path);
        await openWorkspace(path);
      } catch (err) {
        dispatch({
          type: "SET_ERROR",
          error: typeof err === "string" ? err : "Failed to initialize LMBrain kit",
        });
      } finally {
        dispatch({ type: "SET_LOADING", loading: false });
      }
    },
    [openWorkspace]
  );

  const navigateTo = useCallback(
    (view: AppView) => {
      dispatch({ type: "SET_VIEW", view });
      dispatch({ type: "SET_CMDK", open: false });
      viewRef.current = view;
      markPageRead(view);
    },
    [markPageRead],
  );

  const openSpec = useCallback(
    (spec: Spec) => {
      dispatch({ type: "SET_SELECTED_SPEC", spec });
      dispatch({ type: "SET_VIEW", view: "spec" });
      dispatch({ type: "SET_CMDK", open: false });
      viewRef.current = "spec";
      // Opening one spec marks that spec read without clearing the whole Board.
      const item = toUnreadItems("spec", [spec])[0] ?? null;
      applyReadState(markItemRead(readStateRef.current, "taskboard", item), true);
    },
    [applyReadState],
  );

  const closeSpecDetail = useCallback(() => {
    dispatch({ type: "CLOSE_SPEC_DETAIL" });
  }, []);

  const toggleCmdk = useCallback(() => {
    dispatch({ type: "SET_CMDK", open: !state.cmdkOpen });
  }, [state.cmdkOpen]);

  const closeCmdk = useCallback(() => {
    dispatch({ type: "SET_CMDK", open: false });
  }, []);

  const createSession = useCallback(
    async (request: { host: AgentHost; route: ModelRoute; model?: string; label?: string; codex_bin?: string }) => {
      const id = await commands.sessionStart(request);
      const info = (await commands.sessionList()).find((session) => session.id === id);
      const session: SessionInfo = info ?? {
        id,
        label:
          request.label?.trim() ||
          (request.host === "claude" && request.route === "ollama" && request.model
            ? `Claude via ${request.model}`
            : (request.host === "pi" || request.host === "opencode") && request.model
              ? `${request.host === "pi" ? "Pi" : "OpenCode"} via ${request.model}`
              : request.host === "codex"
              ? "Codex"
              : "Claude"),
        host: request.host,
        route: request.route,
        model: request.model ?? null,
        status: "running",
        exit_code: null,
      };
      dispatch({ type: "ADD_SESSION", session });
      dispatch({ type: "SET_VIEW", view: "sessions" });
      return id;
    },
    []
  );

  const closeSession = useCallback(async (id: string) => {
    try {
      await commands.sessionKill(id);
    } catch (error) {
      console.error("Failed to close session:", error);
    } finally {
      dispatch({ type: "REMOVE_SESSION", id });
    }
  }, []);

  const goToPicker = useCallback(async () => {
    commands.stopWatcher().catch(() => {});
    dispatch({ type: "SET_WATCHER", active: false });
    await Promise.all(state.sessions.map((session) => commands.sessionKill(session.id).catch(() => {})));
    dispatch({ type: "CLEAR_SESSIONS" });
    dispatch({ type: "SET_SCREEN", screen: "picker" });
    dispatch({ type: "SET_EXIT_CONFIRM", show: false });
    readPathRef.current = null;
    seedPendingRef.current = false;
    applyReadState({}, false);
  }, [applyReadState, state.sessions]);

  const triggerLeaveWorkspace = useCallback(() => {
    dispatch({ type: "SET_EXIT_CONFIRM", show: true });
  }, []);

  const cancelLeaveWorkspace = useCallback(() => {
    dispatch({ type: "SET_EXIT_CONFIRM", show: false });
  }, []);

  const openDetailArtifact = useCallback((artifact: DetailArtifact | null) => {
    dispatch({ type: "SET_DETAIL_ARTIFACT", artifact });
  }, []);

  const setActiveSession = useCallback((id: string | null) => {
    dispatch({ type: "SET_ACTIVE_SESSION", id });
  }, []);

  const setSessions = useCallback((sessions: SessionInfo[]) => {
    dispatch({ type: "SET_SESSIONS", sessions });
  }, []);

  const setShowWindowCloseConfirm = useCallback((show: boolean) => {
    dispatch({ type: "SET_WINDOW_CLOSE_CONFIRM", show });
  }, []);

  const setWorkspaceNotice = useCallback((notice: string | null) => {
    dispatch({ type: "SET_WORKSPACE_NOTICE", notice });
  }, []);

  const setWikiPage = useCallback((page: WikiPage | null) => {
    dispatch({ type: "SET_WIKI_PAGE", page });
  }, []);

  const setWikiTree = useCallback((tree: WikiTree) => {
    dispatch({ type: "SET_WIKI_TREE", tree });
  }, []);

  return (
    <WorkspaceContext.Provider
      value={{
        state,
        unreadCounts,
        openWorkspace,
        initializeWorkspaceKit,
        loadAllData,
        refreshWorkspaceData,
        navigateTo,
        openSpec,
        closeSpecDetail,
        toggleCmdk,
        closeCmdk,
        goToPicker,
        openDetailArtifact,
        triggerLeaveWorkspace,
        cancelLeaveWorkspace,
        createSession,
        closeSession,
        refreshSessions,
        setActiveSession,
        setSessions,
        setShowWindowCloseConfirm,
        setWorkspaceNotice,
        setWikiPage,
        setWikiTree,
      }}
    >
      {children}
    </WorkspaceContext.Provider>
  );
}
