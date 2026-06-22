import {
  createContext,
  useReducer,
  useCallback,
  useEffect,
  type ReactNode,
} from "react";
import { listen } from "@tauri-apps/api/event";
import type {
  AppView,
  WorkspaceInfo,
  WorkspaceSummary,
  Task,
  Spec,
  Review,
  Adr,
  AgentProfile,
  McpRecord,
  McpProposal,
  Handoff,
  PulseData,
  WikiTree,
  WikiPage,
  GitInfo,
  FileEvent,
  DetailArtifact,
} from "../types";
import * as commands from "../lib/commands";

// ─── State ───────────────────────────────────────────────────────

export interface WorkspaceState {
  screen: "picker" | "app";
  view: AppView;
  currentWorkspace: WorkspaceInfo | null;
  recentWorkspaces: WorkspaceSummary[];
  gitInfo: GitInfo | null;
  pulseData: PulseData | null;
  tasks: Task[];
  specs: Spec[];
  reviews: Review[];
  adrs: Adr[];
  agents: AgentProfile[];
  mcpRecords: McpRecord[];
  mcpProposals: McpProposal[];
  handoffs: Handoff[];
  wikiTree: WikiTree | null;
  wikiPage: WikiPage | null;
  selectedSpec: Spec | null;
  drawerTask: Task | null;
  cmdkOpen: boolean;
  watcherActive: boolean;
  loading: boolean;
  error: string | null;
  detailArtifact: DetailArtifact | null;
}

export type Action =
  | { type: "SET_SCREEN"; screen: "picker" | "app" }
  | { type: "SET_VIEW"; view: AppView }
  | { type: "SET_WORKSPACE"; info: WorkspaceInfo }
  | { type: "SET_RECENT"; workspaces: WorkspaceSummary[] }
  | { type: "SET_GIT_INFO"; info: GitInfo }
  | { type: "SET_PULSE"; data: PulseData }
  | { type: "SET_TASKS"; tasks: Task[] }
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
  | { type: "SET_DRAWER_TASK"; task: Task | null }
  | { type: "SET_CMDK"; open: boolean }
  | { type: "SET_WATCHER"; active: boolean }
  | { type: "SET_LOADING"; loading: boolean }
  | { type: "SET_ERROR"; error: string | null }
  | { type: "SET_DETAIL_ARTIFACT"; artifact: DetailArtifact | null };

const initialState: WorkspaceState = {
  screen: "picker",
  view: "pulse",
  currentWorkspace: null,
  recentWorkspaces: [],
  gitInfo: null,
  pulseData: null,
  tasks: [],
  specs: [],
  reviews: [],
  adrs: [],
  agents: [],
  mcpRecords: [],
  mcpProposals: [],
  handoffs: [],
  wikiTree: null,
  wikiPage: null,
  selectedSpec: null,
  drawerTask: null,
  cmdkOpen: false,
  watcherActive: false,
  loading: false,
  error: null,
  detailArtifact: null,
};

function reducer(state: WorkspaceState, action: Action): WorkspaceState {
  switch (action.type) {
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
    case "SET_TASKS":
      return { ...state, tasks: action.tasks };
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
    case "SET_DRAWER_TASK":
      return { ...state, drawerTask: action.task };
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
    default:
      return state;
  }
}

// ─── Context ─────────────────────────────────────────────────────

export interface WorkspaceContextValue {
  state: WorkspaceState;
  dispatch: React.Dispatch<Action>;
  openWorkspace: (path: string) => Promise<void>;
  initializeWorkspaceKit: (path: string) => Promise<void>;
  loadAllData: () => Promise<void>;
  navigateTo: (view: AppView) => void;
  openSpec: (spec: Spec) => void;
  openTaskDrawer: (task: Task) => void;
  closeTaskDrawer: () => void;
  toggleCmdk: () => void;
  closeCmdk: () => void;
  goToPicker: () => void;
  openDetailArtifact: (artifact: DetailArtifact | null) => void;
}

// eslint-disable-next-line react-refresh/only-export-components
export const WorkspaceContext = createContext<WorkspaceContextValue | null>(null);

export function WorkspaceProvider({ children }: { children: ReactNode }) {
  const [state, dispatch] = useReducer(reducer, initialState);

  const loadAllDataInternal = useCallback(async () => {
    try {
      const [pulseData, tasks, specs, reviews, adrs, agents, mcpRecords, mcpProposals, handoffs] =
        await Promise.all([
          commands.getPulseData(),
          commands.getTasks(),
          commands.getSpecs(),
          commands.getReviews(),
          commands.getAdrs(),
          commands.getAgents(),
          commands.getMcpRecords(),
          commands.getMcpProposals(),
          commands.getHandoffs(),
        ]);

      dispatch({ type: "SET_PULSE", data: pulseData });
      dispatch({ type: "SET_TASKS", tasks });
      dispatch({ type: "SET_SPECS", specs });
      dispatch({ type: "SET_REVIEWS", reviews });
      dispatch({ type: "SET_ADRS", adrs });
      dispatch({ type: "SET_AGENTS", agents });
      dispatch({ type: "SET_MCP_RECORDS", records: mcpRecords });
      dispatch({ type: "SET_MCP_PROPOSALS", proposals: mcpProposals });
      dispatch({ type: "SET_HANDOFFS", handoffs });
    } catch (err) {
      console.error("Failed to load data:", err);
    }
  }, []);

  const loadAllData = useCallback(async () => {
    await loadAllDataInternal();
  }, [loadAllDataInternal]);

  // Load recent workspaces on mount
  useEffect(() => {
    commands.listRecentWorkspaces().then((workspaces) => {
      dispatch({ type: "SET_RECENT", workspaces });
    });
  }, []);

  // Listen for file change events
  useEffect(() => {
    const unlisten = listen<FileEvent>("file-changed", () => {
      loadAllData();
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [loadAllData, state.currentWorkspace]);

  const openWorkspace = useCallback(async (path: string) => {
    dispatch({ type: "SET_LOADING", loading: true });
    dispatch({ type: "SET_ERROR", error: null });
    try {
      const info = await commands.openWorkspace(path);
      dispatch({ type: "SET_WORKSPACE", info });

      // A folder without a kit stays in the picker so the operator can explicitly
      // choose whether to initialize it. No project data is loaded beforehand.
      if (info.health === "none") {
        return;
      }

      // Get git info
      try {
        const gitInfo = await commands.getGitInfo();
        dispatch({ type: "SET_GIT_INFO", info: gitInfo });
      } catch {
        // Git info is optional
      }

      // Refresh recent list
      const workspaces = await commands.listRecentWorkspaces();
      dispatch({ type: "SET_RECENT", workspaces });

      // Load all data
      await loadAllDataInternal();

      // Start watcher
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
  }, [loadAllDataInternal]);

  const initializeWorkspaceKit = useCallback(async (path: string) => {
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
  }, [openWorkspace]);

  const navigateTo = useCallback(
    (view: AppView) => {
      dispatch({ type: "SET_VIEW", view });
      dispatch({ type: "SET_CMDK", open: false });
      dispatch({ type: "SET_DRAWER_TASK", task: null });
    },
    []
  );

  const openSpec = useCallback((spec: Spec) => {
    dispatch({ type: "SET_SELECTED_SPEC", spec });
    dispatch({ type: "SET_VIEW", view: "spec" });
    dispatch({ type: "SET_CMDK", open: false });
  }, []);

  const openTaskDrawer = useCallback((task: Task) => {
    dispatch({ type: "SET_DRAWER_TASK", task });
  }, []);

  const closeTaskDrawer = useCallback(() => {
    dispatch({ type: "SET_DRAWER_TASK", task: null });
  }, []);

  const toggleCmdk = useCallback(() => {
    dispatch({ type: "SET_CMDK", open: !state.cmdkOpen });
  }, [state.cmdkOpen]);

  const closeCmdk = useCallback(() => {
    dispatch({ type: "SET_CMDK", open: false });
  }, []);

  const goToPicker = useCallback(() => {
    commands.stopWatcher().catch(() => {});
    dispatch({ type: "SET_WATCHER", active: false });
    dispatch({ type: "SET_SCREEN", screen: "picker" });
  }, []);

  const openDetailArtifact = useCallback((artifact: DetailArtifact | null) => {
    dispatch({ type: "SET_DETAIL_ARTIFACT", artifact });
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
        openTaskDrawer,
        closeTaskDrawer,
        toggleCmdk,
        closeCmdk,
        goToPicker,
        openDetailArtifact,
      }}
    >
      {children}
    </WorkspaceContext.Provider>
  );
}
