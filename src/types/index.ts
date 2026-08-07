// ─── Domain Types (mirrors Rust models) ───────────────────────────

export interface KitFeedbackNote {
  id: string;
  timestamp: string;
  lmbrain_version: string;
  category: string;
  severity: string;
  summary: string;
  observed_behavior: string;
  expected_behavior: string;
  impact: string;
  evidence: string;
  workaround: string | null;
  suggested_improvement: string | null;
  related_note: string | null;
  actor: string;
}

export interface KitFeedbackReport {
  schema_version: string;
  path: string;
  updated: string;
  total: number;
  counts_by_category: Record<string, number>;
  counts_by_severity: Record<string, number>;
  notes: KitFeedbackNote[];
}

export type KitHealth = "ok" | "warn" | "none";

export interface KitDiagnostic {
  id?: string;
  code?: string;
  message: string;
  severity: "info" | "warning" | "error";
  artifact_id?: string | null;
  path: string | null;
  next_action?: string;
  fixability?: "manual" | "governed-mutation" | "read-only";
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
  finding_count?: number;
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
  /** Lead-owned implementation estimate (issue #64); absent on legacy specs. */
  capability_tier: string | null;
  thinking_level: string | null;
  depends_on?: string[];
  parking_events?: SpecParkingEvent[];
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

export interface SpecParkingEvent {
  timestamp: string;
  actor: string;
  reason: string;
  revisit_condition: string | null;
}

export type FindingStatus =
  | "open"
  | "planned"
  | "deferred"
  | "resolved"
  | "accepted-risk"
  | "superseded";

export type FindingSeverity = "critical" | "high" | "medium" | "low" | "info";

export interface Finding {
  id: string;
  title: string;
  status: FindingStatus | string;
  category: string;
  severity: FindingSeverity | string;
  origin_severity: string | null;
  area: string | null;
  milestone: string | null;
  owner: string | null;
  origin_artifact: string | null;
  origin_ref: string | null;
  related_specs: string[];
  related_reviews: string[];
  related_decisions: string[];
  target_specs: string[];
  blocked_by: string[];
  resolution_refs: string[];
  superseded_by: string | null;
  created: string;
  updated: string;
  tags: string[];
  body: string;
  path: string;
  malformed: boolean;
}

export interface FindingRelation {
  id: string;
  title: string;
  status: string;
  path: string;
}

export interface FindingContext {
  schema_version: string;
  finding: Finding;
  origin: FindingRelation | null;
  related_specs: FindingRelation[];
  related_reviews: FindingRelation[];
  related_decisions: FindingRelation[];
  target_specs: FindingRelation[];
  blockers: FindingRelation[];
  resolution_refs: FindingRelation[];
  superseded_by: FindingRelation | null;
  events: Array<Record<string, unknown>>;
  warnings: string[];
  omitted_relations: number;
}

export interface VerificationRequirement {
  id: string;
  text: string;
  checked: boolean;
  kind: string;
  owner: string;
  phase: string;
  evidence: string;
  source: string;
}

export interface VerificationAttestation {
  schema_version: string;
  id: string;
  requirement_id: string;
  requirement_digest: string;
  actor_role: string;
  actor: string;
  timestamp: string;
  result: string;
  evidence_ref: string;
  evidence_digest?: string;
}

export interface VerificationBlocker {
  requirement_id: string;
  owner: string;
  cause: string;
}

export interface SpecVerificationState {
  requirements: VerificationRequirement[];
  attestations: VerificationAttestation[];
  blockers: VerificationBlocker[];
}

export interface AttestationResult {
  path: string;
  attestation: VerificationAttestation;
  created: boolean;
}

export type VerificationManifestState =
  | "absent"
  | "invalid"
  | "unsafe"
  | "unapproved"
  | "approved"
  | "stale"
  | "approval-invalid";

export interface VerificationGate {
  id: string;
  title?: string | null;
  program: string;
  args: string[];
  cwd: string;
  timeout_seconds?: number | null;
  output_limit_bytes?: number | null;
  expected_exit_code?: number | null;
  result_matcher?: string | null;
  environment: Record<string, string>;
  fingerprint_exclude?: string[];
}

export interface VerificationManifest {
  schema_version: number;
  gates: VerificationGate[];
}

export interface VerificationManifestStatus {
  schema_version: string;
  state: VerificationManifestState;
  manifest_digest: string | null;
  approved_digest: string | null;
  approved_at: string | null;
  workspace_fingerprint: string;
  gate_count: number;
  issues: string[];
  next_action: string;
  can_rollback: boolean;
}

export interface VerificationGateCandidate {
  gate: VerificationGate;
  provenance: string;
  confidence: string;
  selected: boolean;
  environment_policy: string;
  mutation_policy: string;
  security_notes: string[];
}

export interface VerificationManifestPreview {
  schema_version: string;
  status: VerificationManifestStatus;
  candidates: VerificationGateCandidate[];
  conflicts: string[];
  guidance: string[];
  proposed_manifest: VerificationManifest;
  proposed_toml: string;
  proposed_digest: string;
  current_toml: string | null;
  diff: string;
  discovered_only: boolean;
}

export interface VerificationManifestWriteResult {
  path: string;
  digest: string;
  previous_digest: string | null;
  approval_required: boolean;
  rollback_available: boolean;
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

export interface ReviewLifecycleEvent {
  schema_version: string;
  id: string;
  timestamp: string;
  action: string;
  from_status: string;
  to_status: string;
  actor_role: string;
  reason: string;
  evidence_refs: string[];
  implementation_agent: string | null;
  remediation_agent: string | null;
}

export interface ReviewLifecycleAnalysis {
  source: "structured-events" | "legacy-explicit" | "status-only";
  confidence: "high" | "medium" | "low";
  review_passes: number;
  remediation_cycles: number;
  initial_verdict: string | null;
  final_verdict: string | null;
  escalation_count: number;
  takeover_count: number;
  remediation_agents: string[];
  escalation_owners: string[];
  takeover_owners: string[];
  warnings: string[];
}

export interface Review {
  id: string;
  title: string;
  status: ReviewStatus;
  spec_id: string | null;
  reviewer: string | null;
  implementation_agent: string | null;
  finding_categories: string[];
  findings: ReviewFinding[];
  events: ReviewLifecycleEvent[];
  lifecycle: ReviewLifecycleAnalysis;
  lifecycle_warnings: string[];
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
  supersedes: string[];
  superseded_by: string[];
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

export interface AgentImprovementSignal {
  target_profile: string;
  category: string;
  distinct_specs: string[];
  reviews: string[];
  threshold_met: boolean;
  rationale: string;
}

export interface AgentEffectivenessMetrics {
  profile: string;
  reviewed_specs: number;
  accepted_specs: number;
  specs_with_changes_requested: number;
  transcript_fast_fail_reviews: number;
  review_cycles: number;
  remediation_cycles: number;
  lead_escalation_reviews: number;
  escalation_count: number;
  takeover_count: number;
  categorized_findings: number;
  uncategorized_reviews: number;
  first_pass_accepted_specs: number;
  first_pass_acceptance_rate: number;
  average_review_cycles: number;
  transcript_fast_fail_rate: number;
  lead_escalation_rate: number;
  lifecycle_known_reviews: number;
  lifecycle_unknown_reviews: number;
  lifecycle_coverage: number;
  category_values: number;
  canonical_category_values: number;
  category_alias_values: number;
  unknown_categories: string[];
  category_coverage: number;
  attribution_basis: string;
  confidence: string;
  diagnostics: string[];
  data_quality_caveat: string;
}

export interface AgentImprovementInsights {
  signals: AgentImprovementSignal[];
  metrics: AgentEffectivenessMetrics[];
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
  total_review_passes: number;
  remediation_cycles: number;
  escalation_count: number;
  takeover_count: number;
  lifecycle_known_reviews: number;
  lifecycle_coverage: number;
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

export interface WorkspaceSnapshot {
  pulse_data: PulseData;
  specs: Spec[];
  reviews: Review[];
  findings: Finding[];
  adrs: Adr[];
  agents: AgentProfile[];
  agent_proposals: AgentProposal[];
  mcp_records: McpRecord[];
  mcp_proposals: McpProposal[];
  skills: Skill[];
  handoffs: Handoff[];
  diagnostics: KitDiagnostic[];
  project_statistics: ProjectStatistics;
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

export type AgentHost = "claude" | "codex" | "pi" | "opencode";
export type HarnessProbeState = "installed" | "missing" | "error";

export interface HarnessStatus {
  host: AgentHost;
  label: string;
  state: HarnessProbeState;
  executable: string | null;
  version: string | null;
  detail: string | null;
  probed_at: string;
  install_url: string;
  install_command: string;
}

export interface HarnessUpdateRequest {
  host: AgentHost;
  codex_bin?: string;
}

export interface HarnessUpdateResult {
  host: AgentHost;
  success: boolean;
  already_current: boolean;
  before: HarnessStatus;
  after: HarnessStatus;
  exit_code: number | null;
  timed_out: boolean;
  stdout: string;
  stderr: string;
}

export type HarnessApprovalState = "unconfigured" | "approval-required" | "approved" | "stale";

export interface HarnessApprovalStatus {
  state: HarnessApprovalState;
  manifest_digest: string | null;
  approved_digest: string | null;
  approved_at: string | null;
  approved_by: string | null;
  workspace_fingerprint: string;
}

export type HarnessPreviewAction = "preserved" | "added" | "changed" | "conflicted";

export interface HarnessNativeFilePreview {
  path: string;
  owned_paths: string[];
  action: HarnessPreviewAction;
  detail: string;
}

export interface HarnessToolReadiness {
  tool: string;
  available: boolean;
  resolved_path: string | null;
}

export interface HarnessBrowserMcpCapability {
  provider: "playwright";
  mode: "isolated";
  headed?: boolean;
}

export interface HarnessBrowserMcpReadiness {
  provider: string;
  package_available: boolean;
  package_version: string | null;
  browser_runtime_found: boolean;
  state: "configured" | "prerequisite-ready" | "active" | "inactive-lazy" | "failed" | "unknown";
  detail: string;
}

export interface HarnessHostPlan {
  host: "claude-code" | "codex" | "pi" | "open-code";
  effective: {
    enabled: boolean;
    required_tools: string[];
    environment: Record<string, string>;
    lsp?: { required: boolean };
    browser_mcp?: HarnessBrowserMcpCapability;
  };
  supported_capabilities: string[];
  tools: HarnessToolReadiness[];
  lsp: { configured: boolean; prerequisite_ready: boolean; state: "configured" | "prerequisite-ready" | "active" | "inactive-lazy" | "failed" | "unknown" } | null;
  browser_mcp: HarnessBrowserMcpReadiness | null;
  native_files: HarnessNativeFilePreview[];
  ready: boolean;
}

export interface HarnessConfigurationPlan {
  manifest_digest: string;
  hosts: HarnessHostPlan[];
  has_conflicts: boolean;
}

export interface HarnessApplyResult {
  manifest_digest: string;
  changed: boolean;
  files: Array<{ path: string; content_digest: string }>;
}

export interface HarnessDriftEntry {
  path: string;
  state: "changed" | "missing";
  expected_digest: string;
  actual_digest: string | null;
}

export type ModelRoute = "native" | "ollama";
export type SessionStatus = "running" | "exited";

export interface SessionInfo {
  id: string;
  label: string;
  host: AgentHost;
  route: ModelRoute;
  model: string | null;
  status: SessionStatus;
  exit_code: number | null;
}

export interface OllamaModel {
  name: string;
  cloud: boolean;
  capabilities: string[];
}

export type PiPreparationStatus = "ready" | "installed" | "unavailable";

export interface PiPreparationResult {
  status: PiPreparationStatus;
  message: string;
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
  | "findings"
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
  | "harnesses"
  | "environment"
  | "wiki"
  | "taskboard"
  | "spec"
  | "reviews"
  | "findings"
  | "feedback"
  | "decisions"
  | "agents"
  | "mcp"
  | "repository"
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

// ─── Git & GitHub Types ──────────────────────────────────────────

export interface GitFile {
  path: string;
  status: "staged" | "unstaged" | "untracked" | "conflicted" | "deleted" | "renamed";
  diff_target: GitDiffTarget;
  original_path: string | null;
}

export type GitDiffTarget = "staged" | "unstaged" | "untracked" | "conflicted";

export interface GitFileDiff {
  path: string;
  diff: string;
  binary: boolean;
  truncated: boolean;
}

export interface GitDetails {
  branch: string;
  current_commit: string;
  ahead: number;
  behind: number;
  remote_url: string | null;
  owner: string | null;
  repo: string | null;
  files: GitFile[];
}

export interface GitHubPullRequest {
  number: number;
  title: string;
  html_url: string;
  state: string;
  user: string;
  draft: boolean;
  created_at: string;
  updated_at: string;
}

export interface GitHubWorkflowRun {
  id: number;
  name: string;
  display_title: string;
  head_branch: string;
  head_sha: string;
  status: string;
  conclusion: string | null;
  event: string;
  run_number: number;
  run_attempt: number;
  actor: string | null;
  html_url: string;
  created_at: string;
  updated_at: string;
  run_started_at: string | null;
}

export interface GitHubBranch {
  name: string;
  sha: string;
  protected: boolean;
  commits: GitHubCommit[];
  merge_base_sha: string | null;
}

export interface GitHubCommit {
  sha: string;
  message: string;
  author: string | null;
  date: string | null;
  parents: string[];
}

export interface GitHubDashboard {
  has_token: boolean;
  default_branch: string | null;
  branches: GitHubBranch[];
  branches_error: string | null;
  pull_requests: GitHubPullRequest[];
  workflow_runs: GitHubWorkflowRun[];
}
