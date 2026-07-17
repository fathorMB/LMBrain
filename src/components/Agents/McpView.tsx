import { useEffect } from "react";
import { useWorkspace } from "../../hooks/useWorkspace";
import { getMcpRecords, getMcpProposals } from "../../lib/commands";
import type { McpRecord, McpProposal } from "../../types";

const LMBRAIN_MCP_TOOLS: { name: string; category: string; description: string }[] = [
  { name: "spec_ready", category: "Spec", description: "Approve a backlog spec to ready (on operator request)." },
  { name: "spec_start", category: "Spec", description: "Implementation specialist only: move an assigned ready spec to working." },
  { name: "spec_submit", category: "Spec", description: "Implementation specialist only: submit a completed working spec for review." },
  { name: "spec_done", category: "Spec", description: "Project Lead closeout after accepted review, checked criteria, and evidence." },
  { name: "spec_discard", category: "Spec", description: "Discard a spec (requires operator approval)." },
  { name: "review_accept", category: "Review", description: "Accept a review on explicit operator request." },
  { name: "adr_accept", category: "ADR", description: "Accept a proposed ADR (on operator request)." },
  { name: "adr_reject", category: "ADR", description: "Reject a proposed ADR (on operator request)." },
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
  { name: "lmbrain_list_ready_handoffs", category: "Read", description: "List ready handoffs." },
  { name: "lmbrain_project_digest", category: "Context", description: "Compact project overview: title/status, milestone, ready/review specs, blockers, handoffs, active decisions, diagnostics. Read-only." },
  { name: "lmbrain_spec_context", category: "Context", description: "Spec handoff context: metadata, acceptance criteria, linked decisions, agent profile, files, diagnostics. Read-only." },
  { name: "lmbrain_review_context", category: "Context", description: "Review context: acceptance criteria, implementation evidence, linked reviews, decisions, verification commands. Read-only." },
  { name: "verification_manifest_get", category: "Verification", description: "Inspect and validate the repository verification manifest. Read-only." },
  { name: "verification_manifest_approve", category: "Verification", description: "Approve the exact manifest digest for this workspace." },
  { name: "spec_verify", category: "Verification", description: "Execute approved named gates and write an attributable bounded transcript." },
];

export function McpView() {
  const { state, dispatch } = useWorkspace();

  useEffect(() => {
    Promise.all([
      getMcpRecords(),
      getMcpProposals(),
    ])
      .then(([records, proposals]) => {
        dispatch({ type: "SET_MCP_RECORDS", records });
        dispatch({ type: "SET_MCP_PROPOSALS", proposals });
      })
      .catch(console.error);
  }, [dispatch]);

  return (
    <div style={{ overflowY: "auto", height: "100%" }}>
      <div style={{ maxWidth: 920, margin: "0 auto", padding: "24px 36px 70px" }}>
        <h1
          style={{
            fontSize: 24,
            fontWeight: 800,
            letterSpacing: "-.025em",
            margin: "0 0 5px",
          }}
        >
          Model Context Protocol (MCP)
        </h1>
        <p
          style={{
            fontSize: 13.5,
            color: "var(--text-tertiary)",
            margin: "0 0 22px",
          }}
        >
          MCP server capability records and available client integration tools.
        </p>

        {/* MCP Records */}
        <div
          style={{
            fontSize: 11,
            letterSpacing: ".09em",
            textTransform: "uppercase",
            color: "#6c6671",
            fontWeight: 600,
            marginBottom: 11,
          }}
        >
          MCP Specifications
        </div>
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            gap: 9,
            marginBottom: 32,
          }}
        >
          {state.mcpRecords.length === 0 && (
            <div
              style={{
                textAlign: "center",
                padding: 30,
                color: "var(--text-tertiary)",
              }}
            >
              No MCP specifications found.
            </div>
          )}
          {state.mcpRecords.map((mcp) => (
            <MCPCard key={mcp.id} mcp={mcp} />
          ))}
        </div>

        {/* Built-in lmbrain-mcp tools */}
        <div
          style={{
            fontSize: 11,
            letterSpacing: ".09em",
            textTransform: "uppercase",
            color: "#6c6671",
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
              fontSize: 12.5,
              color: "var(--text-tertiary)",
              lineHeight: 1.5,
              marginBottom: 12,
            }}
          >
            Repository-scoped controlled-mutation server, registered automatically for Claude via{" "}
            <span style={{ fontFamily: "var(--font-mono)", color: "#9a949f" }}>.mcp.json</span>{" "}
            and for Codex via{" "}
            <span style={{ fontFamily: "var(--font-mono)", color: "#9a949f" }}>.codex/config.toml</span>,
            Pi via its pinned MCP extension, and OpenCode via{" "}
            <span style={{ fontFamily: "var(--font-mono)", color: "#9a949f" }}>opencode.json</span>.
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
                    fontSize: 11.5,
                    color: "#bcaef6",
                    minWidth: 220,
                  }}
                >
                  {tool.name}
                </span>
                <span
                  style={{
                    fontSize: 10,
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
                <span style={{ fontSize: 12, color: "var(--text-tertiary)" }}>
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
                fontSize: 11,
                letterSpacing: ".09em",
                textTransform: "uppercase",
                color: "#6c6671",
                fontWeight: 600,
                marginBottom: 11,
              }}
            >
              MCP Proposals
            </div>
            <div
              style={{
                display: "flex",
                flexDirection: "column",
                gap: 9,
              }}
            >
              {state.mcpProposals.map((prop) => (
                <MCPCard key={prop.id} mcp={prop} proposal />
              ))}
            </div>
          </>
        )}
      </div>
    </div>
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
    deprecated: { color: "#6c6671", bg: "rgba(108,102,113,.12)" },
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
              fontSize: 12,
              color: "#bcaef6",
            }}
          >
            {mcp.id}
          </span>
          <span
            style={{
              fontSize: 14,
              fontWeight: 600,
              color: "var(--text-primary)",
            }}
          >
            {mcp.title}
          </span>
        </div>
        <div
          style={{
            fontSize: 12,
            color: "var(--text-tertiary)",
          }}
        >
          {proposal ? "Proposal" : "Specification"}
        </div>
      </div>
      <span
        style={{
          fontSize: 10.5,
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
