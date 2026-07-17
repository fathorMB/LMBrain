import {
  createContext,
  useReducer,
  useCallback,
  useEffect,
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
  KitDiagnostic,
  McpProposal,
  McpRecord,
  PulseData,
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

export interface WorkspaceState {
  screen: "picker" | "app";
  view: AppView;
  currentWorkspace: WorkspaceInfo | null;
  recentWorkspaces: WorkspaceSummary[];
  gitInfo: GitInfo | null;
  pulseData: PulseData | null;
  specs: Spec[];
  reviews: Review[];
  adrs: Adr[];
  agents: AgentProfile[];
  agentProposals: AgentProposal[];
  mcpRecords: McpRecord[];
  mcpProposals: McpProposal[];
  skills: Skill[];
  handoffs: Handoff[];
  diagnostics: KitDiagnostic[];
  wikiTree: WikiTree | null;
  wikiPage: WikiPage | null;
  selectedSpec: Spec | null;
  sessions: SessionInfo[];
  activeSessionId: string | null;
  cmdkOpen: boolean;
  watcherActive: boolean;
  loading: boolean;
  loadingMessage: string;
  loadingPath: string | null;
  workspaceNotice: string | null;
  error: string | null;
  detailArtifact: DetailArtifact | null;
  showExitConfirm: boolean;
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
  | { type: "SET_CMDK"; open: boolean }
  | { type: "SET_WATCHER"; active: boolean }
  | { type: "SET_LOADING"; loading: boolean; message?: string; path?: string | null }
  | { type: "SET_WORKSPACE_NOTICE"; notice: string | null }
  | { type: "SET_ERROR"; error: string | null }
  | { type: "SET_DETAIL_ARTIFACT"; artifact: DetailArtifact | null }
  | { type: "SET_EXIT_CONFIRM"; show: boolean }
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
  adrs: [],
  agents: [],
  agentProposals: [],
  mcpRecords: [],
  mcpProposals: [],
  skills: [],
  handoffs: [],
  diagnostics: [],
  wikiTree: null,
  wikiPage: null,
  selectedSpec: null,
  sessions: [],
  activeSessionId: null,
  cmdkOpen: false,
  watcherActive: false,
  loading: false,
  loadingMessage: "Preparing workspace...",
  loadingPath: null,
  workspaceNotice: null,
  error: null,
  detailArtifact: null,
  showExitConfirm: false,
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
    case "SET_CMDK":
      return { ...state, cmdkOpen: action.open };
    case "SET_WATCHER":
      return { ...state, watcherActive: action.active };
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
  dispatch: Dispatch<Action>;
  openWorkspace: (path: string) => Promise<void>;
  initializeWorkspaceKit: (path: string) => Promise<void>;
  loadAllData: () => Promise<void>;
  refreshWorkspaceData: () => Promise<void>;
  navigateTo: (view: AppView) => void;
  openSpec: (spec: Spec) => void;
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
}

// eslint-disable-next-line react-refresh/only-export-components
export const WorkspaceContext = createContext<WorkspaceContextValue | null>(null);

export function WorkspaceProvider({ children }: { children: ReactNode }) {
  const [state, dispatch] = useReducer(reducer, initialState);

  const refreshSessions = useCallback(async () => {
    const infos = await commands.sessionList();
    dispatch({ type: "SET_SESSIONS", sessions: infos });
  }, []);

  const fetchWorkspaceData = useCallback(async (): Promise<Partial<WorkspaceState>> => {
    const [
        pulseData,
        specs,
        reviews,
        adrs,
        agents,
        agentProposals,
        mcpRecords,
        mcpProposals,
        skills,
        handoffs,
        diagnostics,
    ] = await Promise.all([
        commands.getPulseData(),
        commands.getSpecs(),
        commands.getReviews(),
        commands.getAdrs(),
        commands.getAgents(),
        commands.getAgentProposals(),
        commands.getMcpRecords(),
        commands.getMcpProposals(),
        commands.getSkills(),
        commands.getHandoffs(),
        commands.getDiagnostics(),
    ]);

    return {
      pulseData,
      specs,
      reviews,
      adrs,
      agents,
      agentProposals,
      mcpRecords,
      mcpProposals,
      skills,
      handoffs,
      diagnostics,
    };
  }, []);

  const loadAllDataInternal = useCallback(async () => {
    try {
      dispatch({ type: "MERGE_DATA", data: await fetchWorkspaceData() });
    } catch (err) {
      console.error("Failed to load data:", err);
    }
  }, [fetchWorkspaceData]);

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
  }, [fetchWorkspaceData, state.selectedSpec, state.wikiPage]);

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
    [loadAllDataInternal, refreshSessions]
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

  const navigateTo = useCallback((view: AppView) => {
    dispatch({ type: "SET_VIEW", view });
    dispatch({ type: "SET_CMDK", open: false });
  }, []);

  const openSpec = useCallback((spec: Spec) => {
    dispatch({ type: "SET_SELECTED_SPEC", spec });
    dispatch({ type: "SET_VIEW", view: "spec" });
    dispatch({ type: "SET_CMDK", open: false });
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
  }, [state.sessions]);

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

  return (
    <WorkspaceContext.Provider
      value={{
        state,
        dispatch,
        openWorkspace,
        initializeWorkspaceKit,
        loadAllData,
        refreshWorkspaceData,
        navigateTo,
        openSpec,
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
      }}
    >
      {children}
    </WorkspaceContext.Provider>
  );
}
