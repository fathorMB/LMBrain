import { invoke } from "@tauri-apps/api/core";
import type {
  Adr,
  AgentProfile,
  DirEntry,
  FileContent,
  GitInfo,
  Handoff,
  McpProposal,
  McpRecord,
  ParsedDocument,
  PulseData,
  Review,
  Roadmap,
  SessionInfo,
  SessionMode,
  Spec,
  WikiPage,
  WikiTree,
  OllamaModel,
  WorkspaceInfo,
  WorkspaceSummary,
} from "../types";

// ─── Workspace Commands ──────────────────────────────────────────

export async function openWorkspace(path: string): Promise<WorkspaceInfo> {
  return invoke("open_workspace", { path });
}

export async function initializeWorkspaceKit(path: string): Promise<WorkspaceInfo> {
  return invoke("initialize_workspace_kit", { path });
}

export async function listRecentWorkspaces(): Promise<WorkspaceSummary[]> {
  return invoke("list_recent_workspaces");
}

export async function removeRecentWorkspace(path: string): Promise<void> {
  return invoke("remove_recent_workspace", { path });
}

// ─── Filesystem Commands ─────────────────────────────────────────

export async function readFile(path: string): Promise<FileContent> {
  return invoke("read_file", { path });
}

export async function listDirectory(path: string): Promise<DirEntry[]> {
  return invoke("list_directory", { path });
}

// ─── Parse Commands ──────────────────────────────────────────────

export async function parseMarkdown(path: string): Promise<ParsedDocument> {
  return invoke("parse_markdown", { path });
}

// ─── Data Commands ───────────────────────────────────────────────

export async function getPulseData(): Promise<PulseData> {
  return invoke("get_pulse_data");
}

export async function getSpecs(): Promise<Spec[]> {
  return invoke("get_specs");
}

export async function getReviews(): Promise<Review[]> {
  return invoke("get_reviews");
}

export async function getAdrs(): Promise<Adr[]> {
  return invoke("get_adrs");
}

export async function getAgents(): Promise<AgentProfile[]> {
  return invoke("get_agents");
}

export async function getMcpRecords(): Promise<McpRecord[]> {
  return invoke("get_mcp_records");
}

export async function getMcpProposals(): Promise<McpProposal[]> {
  return invoke("get_mcp_proposals");
}

export async function getHandoffs(): Promise<Handoff[]> {
  return invoke("get_handoffs");
}

export async function getRoadmap(): Promise<Roadmap> {
  return invoke("get_roadmap");
}

export async function getWikilinkIndex(): Promise<Record<string, string[]>> {
  return invoke("get_wikilink_index");
}

export async function getDiagnostics(): Promise<import("../types").KitDiagnostic[]> {
  return invoke("get_diagnostics");
}

export interface SearchResult {
  path: string;
  snippet: string;
}

export async function searchContent(query: string): Promise<SearchResult[]> {
  return invoke("search_content", { query });
}

export async function getWikiTree(): Promise<WikiTree> {
  return invoke("get_wiki_tree");
}

export async function getWikiPage(path: string): Promise<WikiPage> {
  return invoke("get_wiki_page", { path });
}

export async function getGitInfo(): Promise<GitInfo> {
  return invoke("get_git_info");
}

// ─── Watcher Commands ────────────────────────────────────────────

export async function startWatcher(): Promise<void> {
  return invoke("start_watcher");
}

export async function stopWatcher(): Promise<void> {
  return invoke("stop_watcher");
}

export async function watcherStatus(): Promise<boolean> {
  return invoke("watcher_status");
}

export interface SessionStartRequest {
  mode: SessionMode;
  model?: string;
  label?: string;
  codex_bin?: string;
}

export async function sessionStart(request: SessionStartRequest): Promise<string> {
  return invoke("session_start", { request });
}

export async function sessionWrite(id: string, data: string): Promise<void> {
  return invoke("session_write", { id, data });
}

export async function sessionResize(id: string, cols: number, rows: number): Promise<void> {
  return invoke("session_resize", { id, cols, rows });
}

export async function sessionKill(id: string): Promise<void> {
  return invoke("session_kill", { id });
}

export async function sessionList(): Promise<SessionInfo[]> {
  return invoke("session_list");
}

export async function listOllamaModels(): Promise<OllamaModel[]> {
  return invoke("list_ollama_models");
}

export async function setArtifactStatus(path: string, targetStatus: string): Promise<string> {
  return invoke("set_artifact_status", { path, targetStatus });
}
