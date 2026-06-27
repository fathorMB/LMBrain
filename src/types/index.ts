// ─── Domain Types (mirrors Rust models) ───────────────────────────

export type KitHealth = "ok" | "warn" | "none";

export interface KitDiagnostic {
  message: string;
  severity: "info" | "warning" | "error";
  path: string | null;
}

export interface WorkspaceSummary {
  path: string;
  name: string;
  health: KitHealth;
  last_opened: string;
  branch: string | null;
  is_clean: boolean | null;
}

export interface WorkspaceInfo {
  path: string;
  name: string;
  kit_version: string;
  health: KitHealth;
  diagnostics: KitDiagnostic[];
  branch: string | null;
  is_clean: boolean | null;
  spec_count: number;
  task_count: number;
  decision_count: number;
  agent_count: number;
}

export type SpecStatus =
  | "backlog"
  | "ready"
  | "working"
  | "review"
  | "done"
  | "discarded";

export interface Spec {
  id: string;
  title: string;
  status: SpecStatus;
  priority: string | null;
  area: string | null;
  milestone: string | null;
  recommended_agent: string | null;
  body: string;
  path: string;
  created: string;
  updated: string;
  tags: string[];
  links: string[];
  related_tasks: string[];
  related_decisions: string[];
  malformed?: boolean;
}

export type ReviewStatus =
  | "pending"
  | "accepted"
  | "changes-requested"
  | "blocked"
  | "superseded";

export interface ReviewFinding {
  id: string;
  text: string;
  severity: string;
}

export interface Review {
  id: string;
  title: string;
  status: ReviewStatus;
  spec_id: string | null;
  reviewer: string | null;
  findings: ReviewFinding[];
  body: string;
  path: string;
  created: string;
  updated: string;
  tags: string[];
  links: string[];
  malformed?: boolean;
}

export type AdrStatus = "proposed" | "accepted" | "rejected" | "superseded" | "deprecated";

export interface Adr {
  id: string;
  title: string;
  status: AdrStatus;
  decision_date: string | null;
  decider: string | null;
  body: string;
  path: string;
  created: string;
  updated: string;
  tags: string[];
  links: string[];
  malformed?: boolean;
}

export type AgentStatus = "proposed" | "active" | "inactive" | "retired";

export interface AgentProfile {
  id: string;
  title: string;
  status: AgentStatus;
  role: string | null;
  activation: string | null;
  can_implement: boolean | null;
  can_review: boolean | null;
  body: string;
  path: string;
  created: string;
  updated: string;
  tags: string[];
  links: string[];
  malformed?: boolean;
}

export type AgentProposalStatus = "proposed" | "approved" | "rejected";

export interface AgentProposal {
  id: string;
  title: string;
  status: AgentProposalStatus;
  body: string;
  path: string;
  created: string;
  updated: string;
  tags: string[];
  links: string[];
  malformed?: boolean;
}

export type McpStatus = "specified" | "active" | "inactive" | "deprecated";
export type McpProposalStatus =
  | "proposed"
  | "approved"
  | "rejected"
  | "implemented"
  | "blocked";

export interface McpRecord {
  id: string;
  title: string;
  status: McpStatus;
  body: string;
  path: string;
  created: string;
  updated: string;
  tags: string[];
  links: string[];
  malformed?: boolean;
}

export interface McpProposal {
  id: string;
  title: string;
  status: McpProposalStatus;
  body: string;
  path: string;
  created: string;
  updated: string;
  tags: string[];
  links: string[];
  malformed?: boolean;
}

export type HandoffStatus = "ready" | "consumed" | "superseded" | "archived";

export interface Handoff {
  id: string;
  title: string;
  status: HandoffStatus;
  body: string;
  path: string;
  created: string;
  updated: string;
  tags: string[];
  links: string[];
  malformed?: boolean;
}

export type DesignMockupKind = "package" | "html-file";

export interface DesignMockup {
  id: string;
  name: string;
  path: string;
  entry_path: string;
  kind: DesignMockupKind;
  modified: string | null;
  size: number;
  summary: string | null;
  manifest_title: string | null;
  manifest_description: string | null;
  has_manifest: boolean;
  has_readme: boolean;
}

export interface DesignMockupHtml {
  path: string;
  content: string;
}

export interface Milestone {
  id: string;
  title: string;
  status: string;
  outcome: string;
  specs: string[];
  decisions: string[];
  risks: string[];
  depends_on: string | null;
}

export interface Roadmap {
  title: string;
  milestones: Milestone[];
}

export interface MetricCard {
  label: string;
  count: number;
  accent: string;
}

export interface ActionItem {
  title: string;
  description: string;
  action_type: string;
  spec_id: string | null;
  agent: string | null;
}

export interface RecentActivity {
  action: string;
  path: string;
  description: string;
  timestamp: string;
}

export interface PulseData {
  focus: string | null;
  milestone: string | null;
  milestone_progress: number | null;
  milestone_due: string | null;
  metrics: MetricCard[];
  actions: ActionItem[];
  blockers: ActionItem[];
  recent_activity: RecentActivity[];
  ready_handoffs: Handoff[];
  active_handoff: Handoff | null;
}

export interface FileContent {
  path: string;
  content: string;
  size: number;
  modified: string;
}

export interface DirEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size: number | null;
  modified: string | null;
}

export interface GitInfo {
  branch: string | null;
  is_clean: boolean | null;
  current_commit: string | null;
}

export type SessionMode = "claude" | "ollama" | "codex";
export type SessionStatus = "running" | "exited";

export interface SessionInfo {
  id: string;
  label: string;
  mode: SessionMode;
  model: string | null;
  status: SessionStatus;
  exit_code: number | null;
}

export interface OllamaModel {
  name: string;
  cloud: boolean;
  capabilities: string[];
}

export interface SessionWindowGeometry {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface SessionWindowState extends SessionInfo {
  geometry: SessionWindowGeometry;
  zIndex: number;
}

export interface ParsedDocument {
  path: string;
  frontmatter: Record<string, unknown>;
  body: string;
  wikilinks: string[];
  diagnostics: string[];
  malformed?: boolean;
}

export type WikiNodeKind =
  | "file"
  | "folder"
  | "knowledge"
  | "decisions"
  | "specs"
  | "tasks"
  | "reviews"
  | "handoffs"
  | "agents"
  | "mcp";

export interface WikiNode {
  name: string;
  path: string;
  kind: WikiNodeKind;
  children: WikiNode[];
  count: number | null;
}

export interface WikiTree {
  root: WikiNode;
}

export interface WikiPage {
  path: string;
  name: string;
  content_html: string;
  frontmatter: Record<string, string>;
  wikilinks: string[];
  backlinks: string[];
  updated: string | null;
  word_count: number | null;
}

export type AppView =
  | "pulse"
  | "sessions"
  | "wiki"
  | "taskboard"
  | "spec"
  | "reviews"
  | "decisions"
  | "agents"
  | "design"
  | "settings"
  | "roadmap"
  | "search";

// ─── Event Types ─────────────────────────────────────────────────

export interface FileEvent {
  kind: "created" | "modified" | "removed";
  path: string;
}

export interface DetailArtifact {
  title: string;
  path: string;
}
