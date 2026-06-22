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

export type TaskStatus =
  | "backlog"
  | "planned"
  | "in-progress"
  | "review"
  | "done"
  | "blocked"
  | "cancelled";

export interface TaskCriteria {
  text: string;
  completed: boolean;
}

export interface TaskActivity {
  action: string;
  timestamp: string;
}

export interface Task {
  id: string;
  title: string;
  status: TaskStatus;
  priority: string | null;
  area: string | null;
  milestone: string | null;
  spec: string | null;
  dependencies: string[];
  criteria: TaskCriteria[];
  activity: TaskActivity[];
  block_reason: string | null;
  body: string;
  path: string;
  created: string;
  updated: string;
  tags: string[];
  links: string[];
}

export type SpecStatus =
  | "proposed"
  | "ready"
  | "in-progress"
  | "review"
  | "accepted"
  | "changes-requested"
  | "archived";

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
}

export type AdrStatus = "proposed" | "accepted" | "superseded" | "deprecated";

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

export interface ParsedDocument {
  path: string;
  frontmatter: Record<string, unknown>;
  body: string;
  wikilinks: string[];
  diagnostics: string[];
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
  | "wiki"
  | "taskboard"
  | "spec"
  | "reviews"
  | "decisions"
  | "agents"
  | "settings"
  | "roadmap"
  | "search";

// ─── Event Types ─────────────────────────────────────────────────

export interface FileEvent {
  kind: "created" | "modified" | "removed";
  path: string;
}
