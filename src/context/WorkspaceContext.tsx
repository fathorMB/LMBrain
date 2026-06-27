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
  SessionMode,
  SessionWindowGeometry,
  SessionWindowState,
  Spec,
  WikiPage,
  WikiTree,
  WorkspaceInfo,
  WorkspaceSummary,
  AgentProfile,
} from "../types";
import * as commands from "../lib/commands";

const DEFAULT_SESSION_GEOMETRY = {
  width: 760,
  height: 460,
};

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
  mcpRecords: McpRecord[];
  mcpProposals: McpProposal[];
  handoffs: Handoff[];
  diagnostics: KitDiagnostic[];
  wikiTree: WikiTree | null;
  wikiPage: WikiPage | null;
  selectedSpec: Spec | null;
  sessions: SessionWindowState[];
  cmdkOpen: boolean;
  watcherActive: boolean;
  loading: boolean;
  error: string | null;
  detailArtifact: DetailArtifact | null;
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
  | { type: "SET_MCP_RECORDS"; records: McpRecord[] }
  | { type: "SET_MCP_PROPOSALS"; proposals: McpProposal[] }
  | { type: "SET_HANDOFFS"; handoffs: Handoff[] }
  | { type: "SET_WIKI_TREE"; tree: WikiTree }
  | { type: "SET_WIKI_PAGE"; page: WikiPage | null }
  | { type: "SET_SELECTED_SPEC"; spec: Spec | null }
  | { type: "SET_CMDK"; open: boolean }
  | { type: "SET_WATCHER"; active: boolean }
  | { type: "SET_LOADING"; loading: boolean }
  | { type: "SET_ERROR"; error: string | null }
  | { type: "SET_DETAIL_ARTIFACT"; artifact: DetailArtifact | null }
  | { type: "SET_SESSIONS"; sessions: SessionWindowState[] }
  | { type: "ADD_SESSION"; session: SessionWindowState }
  | { type: "UPDATE_SESSION"; id: string; patch: Partial<SessionWindowState> }
  | { type: "REMOVE_SESSION"; id: string }
  | { type: "UPDATE_SESSION_GEOMETRY"; id: string; geometry: SessionWindowGeometry }
  | { type: "BRING_SESSION_TO_FRONT"; id: string }
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
  mcpRecords: [],
  mcpProposals: [],
  handoffs: [],
  diagnostics: [],
  wikiTree: null,
  wikiPage: null,
  selectedSpec: null,
  sessions: [],
  cmdkOpen: false,
  watcherActive: false,
  loading: false,
  error: null,
  detailArtifact: null,
};

function nextZIndex(sessions: SessionWindowState[]) {
  return sessions.reduce((max, session) => Math.max(max, session.zIndex), 0) + 1;
}

function defaultGeometry(index: number): SessionWindowGeometry {
  return {
    x: 48 + (index % 6) * 28,
    y: 36 + (index % 6) * 22,
    width: DEFAULT_SESSION_GEOMETRY.width,
    height: DEFAULT_SESSION_GEOMETRY.height,
  };
}

function mergeSessionInfo(
  info: SessionInfo,
  existing: SessionWindowState | undefined,
  index: number,
  zIndex: number
): SessionWindowState {
  return {
    ...info,
    geometry: existing?.geometry ?? defaultGeometry(index),
    zIndex: existing?.zIndex ?? zIndex,
  };
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
    case "SET_MCP_RECORDS":
      return { ...state, mcpRecords: action.records };
    case "SET_MCP_PROPOSALS":
      return { ...state, mcpProposals: action.proposals };
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
      return { ...state, loading: action.loading };
    case "SET_ERROR":
      return { ...state, error: action.error };
    case "SET_DETAIL_ARTIFACT":
      return { ...state, detailArtifact: action.artifact };
    case "SET_SESSIONS":
      return { ...state, sessions: action.sessions };
    case "ADD_SESSION":
      return { ...state, sessions: [...state.sessions, action.session] };
    case "UPDATE_SESSION":
      return {
        ...state,
        sessions: state.sessions.map((session) =>
          session.id === action.id ? { ...session, ...action.patch } : session
        ),
      };
    case "REMOVE_SESSION":
      return {
        ...state,
        sessions: state.sessions.filter((session) => session.id !== action.id),
      };
    case "UPDATE_SESSION_GEOMETRY":
      return {
        ...state,
        sessions: state.sessions.map((session) =>
          session.id === action.id
            ? { ...session, geometry: action.geometry }
            : session
        ),
      };
    case "BRING_SESSION_TO_FRONT": {
      const zIndex = nextZIndex(state.sessions);
      return {
        ...state,
        sessions: state.sessions.map((session) =>
          session.id === action.id ? { ...session, zIndex } : session
        ),
      };
    }
    case "CLEAR_SESSIONS":
      return { ...state, sessions: [] };
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
  navigateTo: (view: AppView) => void;
  openSpec: (spec: Spec) => void;
  toggleCmdk: () => void;
  closeCmdk: () => void;
  goToPicker: () => Promise<void>;
  openDetailArtifact: (artifact: DetailArtifact | null) => void;
  createSession: (request: { mode: SessionMode; model?: string; label?: string; codex_bin?: string }) => Promise<string>;
  closeSession: (id: string) => Promise<void>;
  refreshSessions: () => Promise<void>;
  updateSessionGeometry: (id: string, geometry: SessionWindowGeometry) => void;
  bringSessionToFront: (id: string) => void;
}

// eslint-disable-next-line react-refresh/only-export-components
export const WorkspaceContext = createContext<WorkspaceContextValue | null>(null);

export function WorkspaceProvider({ children }: { children: ReactNode }) {
  const [state, dispatch] = useReducer(reducer, initialState);

  const refreshSessions = useCallback(async () => {
    const infos = await commands.sessionList();
    dispatch({
      type: "SET_SESSIONS",
      sessions: infos.map((info, index) =>
        mergeSessionInfo(
          info,
          state.sessions.find((session) => session.id === info.id),
          index,
          nextZIndex(state.sessions) + index
        )
      ),
    });
  }, [state.sessions]);

  const loadAllDataInternal = useCallback(async () => {
    try {
      const [
        pulseData,
        specs,
        reviews,
        adrs,
        agents,
        mcpRecords,
        mcpProposals,
        handoffs,
        diagnostics,
      ] = await Promise.all([
        commands.getPulseData(),
        commands.getSpecs(),
        commands.getReviews(),
        commands.getAdrs(),
        commands.getAgents(),
        commands.getMcpRecords(),
        commands.getMcpProposals(),
        commands.getHandoffs(),
        commands.getDiagnostics(),
      ]);

      dispatch({
        type: "MERGE_DATA",
        data: {
          pulseData,
          specs,
          reviews,
          adrs,
          agents,
          mcpRecords,
          mcpProposals,
          handoffs,
          diagnostics,
        },
      });
    } catch (err) {
      console.error("Failed to load data:", err);
    }
  }, []);

  const loadAllData = useCallback(async () => {
    await loadAllDataInternal();
  }, [loadAllDataInternal]);

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
      dispatch({ type: "SET_LOADING", loading: true });
      dispatch({ type: "SET_ERROR", error: null });
      try {
        const info = await commands.openWorkspace(path);
        dispatch({ type: "SET_WORKSPACE", info });

        if (info.health === "none") {
          return;
        }

        try {
          const gitInfo = await commands.getGitInfo();
          dispatch({ type: "SET_GIT_INFO", info: gitInfo });
        } catch {
          // Git info is optional
        }

        const workspaces = await commands.listRecentWorkspaces();
        dispatch({ type: "SET_RECENT", workspaces });

        await loadAllDataInternal();
        await refreshSessions();

        try {
          await commands.startWatcher();
          dispatch({ type: "SET_WATCHER", active: true });
        } catch {
          // Watcher is optional
        }

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
      dispatch({ type: "SET_LOADING", loading: true });
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
    async (request: { mode: SessionMode; model?: string; label?: string; codex_bin?: string }) => {
      const id = await commands.sessionStart(request);
      const info = (await commands.sessionList()).find((session) => session.id === id);
      const session: SessionWindowState = mergeSessionInfo(
        info ?? {
          id,
          label:
            request.label?.trim() ||
            (request.mode === "ollama" && request.model
              ? `Claude via ${request.model}`
              : request.mode === "codex"
                ? "Codex"
                : "Claude"),
          mode: request.mode,
          model: request.model ?? null,
          status: "running",
          exit_code: null,
        },
        undefined,
        state.sessions.length,
        nextZIndex(state.sessions)
      );
      dispatch({ type: "ADD_SESSION", session });
      dispatch({ type: "SET_VIEW", view: "sessions" });
      return id;
    },
    [state.sessions]
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
  }, [state.sessions]);

  const openDetailArtifact = useCallback((artifact: DetailArtifact | null) => {
    dispatch({ type: "SET_DETAIL_ARTIFACT", artifact });
  }, []);

  const updateSessionGeometry = useCallback((id: string, geometry: SessionWindowGeometry) => {
    dispatch({ type: "UPDATE_SESSION_GEOMETRY", id, geometry });
  }, []);

  const bringSessionToFront = useCallback((id: string) => {
    dispatch({ type: "BRING_SESSION_TO_FRONT", id });
  }, []);

  return (
    <WorkspaceContext.Provider
      value={{
        state,
        dispatch,
        openWorkspace,
        initializeWorkspaceKit,
        loadAllData,
        navigateTo,
        openSpec,
        toggleCmdk,
        closeCmdk,
        goToPicker,
        openDetailArtifact,
        createSession,
        closeSession,
        refreshSessions,
        updateSessionGeometry,
        bringSessionToFront,
      }}
    >
      {children}
    </WorkspaceContext.Provider>
  );
}
