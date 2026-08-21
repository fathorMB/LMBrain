export * from "./generated";

// Backwards-compatible aliases
import type { Diagnostic, RelationSummary, VerificationGate } from "./generated";
export type KitDiagnostic = Diagnostic;
export type DebtRelation = RelationSummary;

// ─── UI and View-Only Types ────────────────────────────────────────

export interface GitWorktree {
  name: string;
  path: string;
  branch: string | null;
  head: string | null;
  prunable: boolean;
  locked: boolean;
  details: GitDetails | null;
}

export type DebtSeverity = "critical" | "high" | "medium" | "low" | "info";

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
  delegated_by?: string;
  delegation_channel?: string;
  delegation_authorization?: string;
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
  proposed_manifest: { schema_version: number; gates: VerificationGate[] };
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

export type PiPreparationStatus = "ready" | "installed" | "unavailable";

export interface PiPreparationResult {
  status: PiPreparationStatus;
  message: string;
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
  | "operations"
  | "debts"
  | "dreams"
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

export interface DetailArtifact {
  title: string;
  path: string;
}

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

export interface GitHubDashboard {
  has_token: boolean;
  pull_requests: GitHubPullRequest[];
  workflow_runs: GitHubWorkflowRun[];
}
