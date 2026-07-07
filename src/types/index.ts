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

export type KitMigrationStatus =
  | "up-to-date"
  | "migration-available"
  | "project-newer-than-app"
  | "unknown-project-version"
  | "unknown-bundled-version"
  | "migration-guidance-missing";

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
  project_kit_version: string;
  bundled_kit_version: string;
  bundled_kit_path: string;
  kit_migration_status: KitMigrationStatus;
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
  skills: string[];
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
  mnemonic_name: string | null;
  status: AgentStatus;
  role: string | null;
  activation: string | null;
  can_implement: boolean | null;
  can_review: boolean | null;
  // V3 specialization metadata (optional, backward-compatible)
  domains: string[] | null;
  primary_files: string[] | null;
  review_focus: string[] | null;
  context_pack: string | null;
  constraints: string[] | null;
  skills: string[] | null;
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
  proposed_mnemonic_name: string | null;
  // V3: proposal type — "new-profile" (default) or "improvement"
  proposal_type: string | null;
  // V3: target profile ID for improvement proposals
  target_profile: string | null;
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

export type SkillStatus = "proposed" | "active" | "retired";

export interface Skill {
  id: string;
  title: string;
  status: SkillStatus;
  scope: string | null;
  kind: string | null;
  risk: string | null;
  applies_to: string[];
  domains: string[];
  commands: string[];
  requires_operator_approval: boolean | null;
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

// ─── V3 milestone intelligence ────────────────────────────────────

export interface MilestoneSpecSummary {
  id: string;
  title: string;
  status: string;
  priority: string | null;
  area: string | null;
  recommended_agent: string | null;
  path: string | null;
}

export interface MilestoneReviewSummary {
  id: string;
  title: string;
  status: string;
  spec_id: string | null;
  path: string | null;
}

export interface MilestoneAdrSummary {
  id: string;
  title: string;
  status: string;
  path: string | null;
}

export interface MilestoneDetail {
  id: string;
  title: string;
  status: string;
  outcome: string;
  depends_on: string | null;
  risks: string[];
  spec_count: number;
  spec_counts_by_status: Record<string, number>;
  specs: MilestoneSpecSummary[];
  reviews: MilestoneReviewSummary[];
  decisions: MilestoneAdrSummary[];
  unresolved_refs: string[];
  next_action: string | null;
  progress_pct: number;
}

export interface MilestoneOverview {
  title: string;
  milestones: MilestoneDetail[];
  unmapped_specs: MilestoneSpecSummary[];
  warnings: string[];
}

export interface StatusCount {
  status: string;
  count: number;
}

export interface ArtifactFamilyStats {
  family: string;
  label: string;
  total: number;
  statuses: StatusCount[];
}

export interface SpecFlowStats {
  total_specs: number;
  done_specs: number;
  open_specs: number;
  done_ratio: number;
  by_status: StatusCount[];
  by_priority: StatusCount[];
  by_area: StatusCount[];
}

export interface ReviewDimensionStat {
  value: string;
  reviewed_specs: number;
  specs_with_changes_requested: number;
  change_request_rate: number;
}

export interface ReviewTrendPoint {
  period: string;
  total_reviews: number;
  accepted_reviews: number;
  changes_requested_reviews: number;
  reviewed_specs: number;
  specs_with_changes_requested: number;
}

export interface ReviewQualityStats {
  total_reviews: number;
  reviewed_specs: number;
  accepted_reviews: number;
  changes_requested_reviews: number;
  blocked_reviews: number;
  superseded_reviews: number;
  reviews_without_spec: number;
  reviews_without_created: number;
  specs_with_changes_requested: number;
  specs_with_multiple_changes_requested: number;
  change_request_rate: number;
  first_pass_eligible_specs: number;
  first_pass_accepted_specs: number;
  first_pass_acceptance_rate: number;
  average_reviews_per_reviewed_spec: number;
  by_area: ReviewDimensionStat[];
  by_agent: ReviewDimensionStat[];
  trend: ReviewTrendPoint[];
}

export interface DiagnosticStats {
  total: number;
  warnings: number;
  errors: number;
  by_family: StatusCount[];
}

export interface ProjectStatistics {
  artifact_families: ArtifactFamilyStats[];
  spec_flow: SpecFlowStats;
  review_quality: ReviewQualityStats;
  diagnostics: DiagnosticStats;
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

// SessionWindowGeometry and SessionWindowState were removed in v3.
// Sessions are now tab-based; SessionInfo is the only session type needed.

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
  | "skills"
  | "insights"
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
