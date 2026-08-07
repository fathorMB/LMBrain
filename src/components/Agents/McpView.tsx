import { useEffect, useState } from "react";
import { useWorkspace } from "../../hooks/useWorkspace";
import { getMcpRecords, getMcpProposals } from "../../lib/commands";
import { CardGrid, PageHeader, PageShell } from "../Shared/PageLayout";
import type { McpRecord, McpProposal } from "../../types";

const LMBRAIN_MCP_TOOLS: { name: string; category: string; description: string }[] = [
  { name: "spec_ready", category: "Spec", description: "Approve a backlog spec to ready (on operator request)." },
  { name: "spec_start", category: "Spec", description: "Implementation specialist only: move an assigned ready spec to working." },
  { name: "spec_submit", category: "Spec", description: "Implementation specialist only: submit a completed working spec for review." },
  { name: "spec_done", category: "Spec", description: "Project Lead closeout after accepted review, checked criteria, and evidence." },
  { name: "spec_discard", category: "Spec", description: "Discard a spec (requires operator approval)." },
  { name: "spec_park", category: "Spec", description: "Project Lead: park a ready spec in backlog with preserved reason and history." },
  { name: "spec_dependencies_set", category: "Spec", description: "Replace backlog hard prerequisites with graph validation, audit, and stale-write protection." },
  { name: "spec_dependency_context", category: "Context", description: "Inspect direct, dependent, transitive, and blocking hard-spec relationships. Read-only." },
  { name: "spec_dependency_candidates", category: "Context", description: "Inventory explicit legacy hard-dependency prose without promoting it. Read-only." },
  { name: "review_accept", category: "Review", description: "Accept a review and record the verdict event on explicit operator request." },
  { name: "review_changes_requested", category: "Review", description: "Request changes with a required rationale and evidence references." },
  { name: "review_block", category: "Review", description: "Block a review with a required rationale and evidence references." },
  { name: "review_supersede", category: "Review", description: "Supersede a review while preserving its verdict history." },
  { name: "review_remediation", category: "Review", description: "Record an attributable remediation attempt without changing review status." },
  { name: "review_escalate", category: "Review", description: "Record an operator-authorized escalation without changing review status." },
  { name: "review_takeover", category: "Review", description: "Record an operator-authorized bounded corrective takeover." },
  { name: "review_remediation_verified", category: "Review", description: "Project Lead: record verification of a remediation cycle without changing review status." },
  { name: "review_migration_preview", category: "Review", description: "Preview lifecycle and taxonomy migration coverage without rewriting reviews." },
  { name: "finding_create", category: "Finding", description: "Create a governed finding with taxonomy category, severity, and evidence." },
  { name: "finding_context", category: "Finding", description: "Finding context pack: linked artifacts, lifecycle events, diagnostics. Read-only." },
  { name: "finding_candidates", category: "Finding", description: "Inventory review findings eligible for promotion. Read-only." },
  { name: "finding_plan", category: "Finding", description: "Link a finding to the spec that will resolve it." },
  { name: "finding_resolve", category: "Finding", description: "Resolve a finding with evidence references." },
  { name: "finding_defer", category: "Finding", description: "Defer a finding with a reason and revisit condition." },
  { name: "finding_accept_risk", category: "Finding", description: "Accept a finding's risk with an audited rationale (on operator request)." },
  { name: "finding_reopen", category: "Finding", description: "Reopen a resolved, deferred, or risk-accepted finding." },
  { name: "finding_supersede", category: "Finding", description: "Supersede a finding with a newer one, writing both sides." },
  { name: "spec_set_effort", category: "Spec", description: "Project Lead: set the implementation estimate (capability tier and thinking level)." },
  { name: "spec_set_tags", category: "Spec", description: "Project Lead: replace a spec's tags after taxonomy validation." },
  { name: "spec_record_effort_observation", category: "Spec", description: "Specialist: record the effort the work actually required as evidence; never rewrites the estimate." },
  { name: "spec_attest_lead", category: "Spec", description: "Project Lead: record a typed attestation for an owner=lead verification gate." },
  { name: "adr_accept", category: "ADR", description: "Accept a proposed ADR (on operator request)." },
  { name: "adr_reject", category: "ADR", description: "Reject a proposed ADR (on operator request)." },
  { name: "adr_supersede", category: "ADR", description: "Project Lead: retire a decision in favour of an accepted one, writing both sides. Idempotent." },
  { name: "agent_activate", category: "Agent", description: "Activate a proposed agent profile (on operator request)." },
  { name: "agent_deactivate", category: "Agent", description: "Deactivate an agent profile (on operator request)." },
  { name: "agent_proposal_approve", category: "Agent", description: "Approve a governed agent proposal on explicit operator request." },
  { name: "agent_proposal_reject", category: "Agent", description: "Reject a governed agent proposal on explicit operator request." },
  { name: "agent_improvement_signals", category: "Learning", description: "Aggregate repeated review categories and per-profile effectiveness metrics. Read-only." },
  { name: "agent_improvement_propose", category: "Learning", description: "Create an evidence-linked additive profile improvement proposal." },
  { name: "agent_improvement_apply", category: "Learning", description: "Apply an approved, non-stale improvement proposal atomically." },
  { name: "skill_activate", category: "Skill", description: "Activate a proposed project-scoped skill (on operator request)." },
  { name: "skill_retire", category: "Skill", description: "Retire a project-scoped skill that should no longer be recommended." },
  // Handoff lifecycle tools
  { name: "handoff_consume", category: "Handoff", description: "Consume a ready session handoff (Project Lead only, after validation)." },
  { name: "handoff_supersede", category: "Handoff", description: "Supersede a ready session handoff with a newer one." },
  { name: "handoff_archive", category: "Handoff", description: "Archive/retire a session handoff." },
  
  { name: "lmbrain_create", category: "Create", description: "Create an artifact with an allocated ID." },
  { name: "lmbrain_set_recommended_agent", category: "Setter", description: "Set a spec's recommended agent." },
  { name: "lmbrain_set_agent_mnemonic_name", category: "Setter", description: "Set an agent profile's mnemonic human name." },
  { name: "lmbrain_get_artifact", category: "Read", description: "Read a repository artifact." },
  { name: "lmbrain_validate", category: "Read", description: "Validate controlled-mutation invariants." },
  { name: "lmbrain_repair_frontmatter", category: "Repair", description: "Operator-authorized: merge duplicate top-level frontmatter keys left by failed mutations; audited, refuses ambiguity." },
  { name: "branching_strategy_get", category: "Branching", description: "Read the declared project branching strategy and summary digest. Read-only." },
  { name: "branching_strategy_set", category: "Branching", description: "Project Lead: declare the project branching strategy with actor and reason (on operator request)." },
  { name: "harness_config_get", category: "Environment", description: "Read and validate project harness intent. Read-only." },
  { name: "harness_config_validate", category: "Environment", description: "Validate a complete candidate harness manifest without writing it." },
  { name: "harness_config_set", category: "Environment", description: "Atomically replace the project harness manifest after strict validation; approval and apply remain separate Lead actions." },
  { name: "harness_approval_status", category: "Environment", description: "Read the machine-local harness approval state. Read-only." },
  { name: "harness_plan_preview", category: "Environment", description: "Deterministic preview of native host files, readiness, and conflicts. Read-only." },
  { name: "harness_manifest_approve", category: "Environment", description: "Project Lead: approve the exact previewed manifest digest for this workspace; audited." },
  { name: "harness_approval_revoke", category: "Environment", description: "Project Lead: revoke this workspace's harness approval; audited." },
  { name: "harness_config_apply", category: "Environment", description: "Project Lead: materialize the approved manifest into native host files atomically with rollback and drift hashes; audited." },
  { name: "harness_drift_status", category: "Environment", description: "Compare applied native-file hashes against disk and report drift. Read-only." },
  { name: "lmbrain_feedback_record", category: "Feedback", description: "Project Lead: append an evidence-backed observation about LMBrain itself without changing project state." },
  { name: "lmbrain_feedback_report", category: "Feedback", description: "Read the portable typed LMBrain kit-feedback report. Read-only." },
  { name: "lmbrain_list_ready_handoffs", category: "Read", description: "List ready handoffs." },
  { name: "lmbrain_project_digest", category: "Context", description: "Bounded project orientation with declared/derived state, all spec lifecycle counts, roadmap reconciliation, actionable diagnostics, and exact omitted counts. Read-only." },
  { name: "lmbrain_spec_context", category: "Context", description: "Spec handoff context: metadata, acceptance criteria, linked decisions, agent profile, files, diagnostics. Read-only." },
  { name: "lmbrain_review_context", category: "Context", description: "Review context: acceptance criteria, implementation evidence, linked reviews, decisions, verification commands. Read-only." },
  { name: "verification_manifest_get", category: "Verification", description: "Inspect and validate the repository verification manifest. Read-only." },
  { name: "verification_manifest_status", category: "Verification", description: "Report manifest state (absent, unapproved, approved, stale, …) with a next action. Read-only." },
  { name: "verification_manifest_init", category: "Verification", description: "Bounded deterministic gate discovery preview from repository metadata; never executes commands." },
  { name: "verification_manifest_validate", category: "Verification", description: "Validate complete manifest TOML without writing it." },
  { name: "verification_manifest_set", category: "Verification", description: "Atomically replace the manifest with stale-write protection; keeps one recoverable previous version." },
  { name: "verification_manifest_rollback", category: "Verification", description: "Restore the recoverable previous manifest with the same stale-write protection." },
  { name: "verification_manifest_approve", category: "Verification", description: "Project Lead: approve the exact manifest digest for this workspace." },
  { name: "verification_migration_preview", category: "Verification", description: "Conservative preview for legacy operator-owned before-done gates; never rewrites specs. Read-only." },
  { name: "spec_verify", category: "Verification", description: "Execute approved named gates and write an attributable bounded transcript." },
];

export function McpView() {
  const { state, dispatch } = useWorkspace();
  const [isLoading, setIsLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [reloadRevision, setReloadRevision] = useState(0);

  useEffect(() => {
    let cancelled = false;

    Promise.all([
      getMcpRecords(),
      getMcpProposals(),
    ])
      .then(([records, proposals]) => {
        if (cancelled) return;
        dispatch({ type: "SET_MCP_RECORDS", records });
        dispatch({ type: "SET_MCP_PROPOSALS", proposals });
      })
      .catch((error) => {
        if (cancelled) return;
        console.error("Failed to load MCP project data:", error);
        setLoadError("Unable to load project MCP specifications.");
      })
      .finally(() => {
        if (!cancelled) setIsLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [dispatch, reloadRevision]);

  return (
    <PageShell archetype="dense">
      <PageHeader
        title="Model Context Protocol (MCP)"
        description="MCP server capability records and available client integration tools."
      />

        {/* MCP Records */}
        <div
          style={{
            fontSize: "var(--text-xs)",
            letterSpacing: ".09em",
            textTransform: "uppercase",
            color: "var(--text-tertiary)",
            fontWeight: 600,
            marginBottom: 11,
          }}
        >
          Project MCP specifications
        </div>
        <p style={{ fontSize: "var(--text-sm)", color: "var(--text-tertiary)", margin: "-2px 0 11px" }}>
          Project-scoped MCP records from <span style={{ fontFamily: "var(--font-mono)" }}>.lmbrain/mcp/specs</span>.
          These describe integrations; the built-in section below lists the tools exposed by LMBrain itself.
        </p>
        <div style={{ marginBottom: "var(--space-6)" }}>
          {isLoading && (
            <div role="status" style={{ textAlign: "center", padding: 30, color: "var(--text-tertiary)" }}>
              Loading project MCP specifications…
            </div>
          )}
          {!isLoading && loadError && (
            <div style={{ textAlign: "center", padding: 30, color: "var(--text-tertiary)" }}>
              <div role="alert">{loadError}</div>
              <button
                type="button"
                onClick={() => {
                  setLoadError(null);
                  setIsLoading(true);
                  setReloadRevision((revision) => revision + 1);
                }}
                style={{
                  marginTop: 12,
                  border: "1px solid var(--border-secondary)",
                  borderRadius: 7,
                  background: "var(--bg-tertiary)",
                  color: "var(--text-secondary)",
                  padding: "6px 10px",
                  cursor: "pointer",
                }}
              >
                Retry
              </button>
            </div>
          )}
          {!isLoading && !loadError && state.mcpRecords.length === 0 && (
            <div
              style={{
                textAlign: "center",
                padding: 30,
                color: "var(--text-tertiary)",
              }}
            >
              <div>No project MCP specifications found.</div>
              <div style={{ fontSize: "var(--text-sm)", marginTop: 6 }}>
                This is valid when the workspace does not declare project-specific MCP integrations.
              </div>
            </div>
          )}
          {!isLoading && !loadError && state.mcpRecords.length > 0 && (
            <CardGrid>
              {state.mcpRecords.map((mcp) => (
                <MCPCard key={mcp.id} mcp={mcp} />
              ))}
            </CardGrid>
          )}
        </div>

        {/* Built-in lmbrain-mcp tools */}
        <div
          style={{
            fontSize: "var(--text-xs)",
            letterSpacing: ".09em",
            textTransform: "uppercase",
            color: "var(--text-tertiary)",
            fontWeight: 600,
            marginBottom: 11,
          }}
        >
          Built-in · lmbrain-mcp tools
        </div>
        <div
          style={{
            background: "var(--bg-tertiary)",
            border: "1px solid var(--border-secondary)",
            borderRadius: 11,
            padding: "14px 16px",
            marginBottom: 32,
          }}
        >
          <div
            style={{
              fontSize: "var(--text-sm)",
              color: "var(--text-tertiary)",
              lineHeight: 1.5,
              marginBottom: 12,
            }}
          >
            Repository-scoped controlled-mutation server, registered automatically for Claude via{" "}
            <span style={{ fontFamily: "var(--font-mono)", color: "var(--text-secondary)" }}>.mcp.json</span>{" "}
            and for Codex via{" "}
            <span style={{ fontFamily: "var(--font-mono)", color: "var(--text-secondary)" }}>.codex/config.toml</span>,
            Pi via its pinned MCP extension, and OpenCode via{" "}
            <span style={{ fontFamily: "var(--font-mono)", color: "var(--text-secondary)" }}>opencode.json</span>.
            Agents call these per-verb tools instead of editing Markdown by hand.
          </div>
          <div style={{ display: "flex", flexDirection: "column", gap: 7 }}>
            {LMBRAIN_MCP_TOOLS.map((tool) => (
              <div
                key={tool.name}
                style={{ display: "flex", alignItems: "center", gap: 10 }}
              >
                <span
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "var(--text-xs)",
                    color: "#bcaef6",
                    minWidth: 220,
                  }}
                >
                  {tool.name}
                </span>
                <span
                  style={{
                    fontSize: "var(--text-2xs)",
                    fontWeight: 700,
                    color: "#7fa8f5",
                    background: "rgba(91,141,239,.12)",
                    borderRadius: 5,
                    padding: "2px 7px",
                    flex: "none",
                  }}
                >
                  {tool.category}
                </span>
                <span style={{ fontSize: "var(--text-sm)", color: "var(--text-tertiary)" }}>
                  {tool.description}
                </span>
              </div>
            ))}
          </div>
        </div>

        {/* MCP Proposals */}
        {state.mcpProposals.length > 0 && (
          <>
            <div
              style={{
                fontSize: "var(--text-xs)",
                letterSpacing: ".09em",
                textTransform: "uppercase",
                color: "var(--text-tertiary)",
                fontWeight: 600,
                marginBottom: 11,
              }}
            >
              MCP Proposals
            </div>
            <CardGrid>
              {state.mcpProposals.map((prop) => (
                <MCPCard key={prop.id} mcp={prop} proposal />
              ))}
            </CardGrid>
          </>
        )}
    </PageShell>
  );
}

function MCPCard({
  mcp,
  proposal,
}: {
  mcp: McpRecord | McpProposal;
  proposal?: boolean;
}) {
  const statusColors: Record<string, { color: string; bg: string }> = {
    active: { color: "#46b07d", bg: "rgba(70,176,125,.12)" },
    specified: { color: "#5b8def", bg: "rgba(91,141,239,.12)" },
    inactive: { color: "#8a8d99", bg: "rgba(139,141,152,.12)" },
    proposed: { color: "#e0a23a", bg: "rgba(224,162,58,.12)" },
    approved: { color: "#46b07d", bg: "rgba(70,176,125,.12)" },
    rejected: { color: "#e0584a", bg: "rgba(224,88,74,.12)" },
    implemented: { color: "#7c6cf6", bg: "rgba(124,108,246,.12)" },
    blocked: { color: "#e0584a", bg: "rgba(224,88,74,.12)" },
    deprecated: { color: "var(--text-tertiary)", bg: "rgba(108,102,113,.12)" },
  };
  const sc = statusColors[mcp.status] || statusColors.proposed;

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: 14,
        background: "var(--bg-tertiary)",
        border: "1px solid var(--border-secondary)",
        borderRadius: 11,
        padding: "14px 16px",
      }}
    >
      <div
        style={{
          width: 36,
          height: 36,
          borderRadius: 10,
          background: "rgba(91,141,239,.12)",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          flex: "none",
        }}
      >
        <i
          className="material-symbols-outlined"
          style={{ fontSize: 18, color: "#7fa8f5" }}
        >
          dns
        </i>
      </div>
      <div style={{ flex: 1 }}>
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: 9,
            marginBottom: 2,
          }}
        >
          <span
            style={{
              fontFamily: "var(--font-mono)",
              fontSize: "var(--text-sm)",
              color: "#bcaef6",
            }}
          >
            {mcp.id}
          </span>
          <span
            style={{
              fontSize: "var(--text-md)",
              fontWeight: 600,
              color: "var(--text-primary)",
            }}
          >
            {mcp.title}
          </span>
        </div>
        <div
          style={{
            fontSize: "var(--text-sm)",
            color: "var(--text-tertiary)",
          }}
        >
          {proposal ? "Proposal" : "Specification"}
        </div>
      </div>
      <span
        style={{
          fontSize: "var(--text-xs)",
          fontWeight: 700,
          color: sc.color,
          background: sc.bg,
          borderRadius: 5,
          padding: "3px 8px",
        }}
      >
        {mcp.status.toUpperCase()}
      </span>
    </div>
  );
}
