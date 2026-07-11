import { useEffect } from "react";
import { useWorkspace } from "../../hooks/useWorkspace";
import { getAgentProposals, getAgents, getMcpRecords, getMcpProposals } from "../../lib/commands";
import type { AgentProfile, AgentProposal, McpRecord, McpProposal } from "../../types";

// Built-in controlled-mutation tools exposed by the repository-scoped `lmbrain-mcp`
// server. Keep in sync with `lmbrain-mcp/src/main.rs` (tools()).
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
  { name: "skill_activate", category: "Skill", description: "Activate a proposed project-scoped skill (on operator request)." },
  { name: "skill_retire", category: "Skill", description: "Retire a project-scoped skill that should no longer be recommended." },
  { name: "lmbrain_create", category: "Create", description: "Create an artifact with an allocated ID." },
  { name: "lmbrain_set_recommended_agent", category: "Setter", description: "Set a spec's recommended agent." },
  { name: "lmbrain_set_agent_mnemonic_name", category: "Setter", description: "Set an agent profile's mnemonic human name." },
  { name: "lmbrain_get_artifact", category: "Read", description: "Read a repository artifact." },
  { name: "lmbrain_validate", category: "Read", description: "Validate controlled-mutation invariants." },
  { name: "lmbrain_list_ready_handoffs", category: "Read", description: "List ready handoffs." },
  // V3 context-pack tools
  { name: "lmbrain_project_digest", category: "Context", description: "Compact project overview: title/status, milestone, ready/review specs, blockers, handoffs, active decisions, diagnostics. Read-only." },
  { name: "lmbrain_spec_context", category: "Context", description: "Spec handoff context: metadata, acceptance criteria, linked decisions, agent profile, files, diagnostics. Read-only." },
  { name: "lmbrain_review_context", category: "Context", description: "Review context: acceptance criteria, implementation evidence, linked reviews, decisions, verification commands. Read-only." },
];

export function AgentsMCPView() {
  const { state, dispatch } = useWorkspace();
  const visibleAgentProposals = state.agentProposals.filter(
    (proposal) =>
      proposal.status === "proposed" ||
      (proposal.status === "approved" &&
        !approvedProposalHasMaterializedProfile(proposal, state.agents))
  );

  useEffect(() => {
    Promise.all([
      getAgents(),
      getAgentProposals(),
      getMcpRecords(),
      getMcpProposals(),
    ])
      .then(([agents, agentProposals, records, proposals]) => {
        dispatch({ type: "SET_AGENTS", agents });
        dispatch({ type: "SET_AGENT_PROPOSALS", proposals: agentProposals });
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
          Agents &amp; MCP
        </h1>
        <p
          style={{
            fontSize: 13.5,
            color: "var(--text-tertiary)",
            margin: "0 0 22px",
          }}
        >
          Agent profiles and MCP capability records. All agents are started
          manually — LMBrain never auto-launches.
        </p>

        {/* Agent Profiles */}
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
          Agent Profiles
        </div>
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            gap: 9,
            marginBottom: 32,
          }}
        >
          {state.agents.length === 0 && (
            <div
              style={{
                textAlign: "center",
                padding: 30,
                color: "var(--text-tertiary)",
              }}
            >
              No agent profiles found.
            </div>
          )}
          {state.agents.map((agent) => (
            <AgentCard key={agent.id} agent={agent} />
          ))}
        </div>

        {/* Agent Proposals */}
        {visibleAgentProposals.length > 0 && (
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
              Agent Proposals
            </div>
            <div
              style={{
                display: "flex",
                flexDirection: "column",
                gap: 9,
                marginBottom: 32,
              }}
            >
              {visibleAgentProposals.map((proposal) => (
                <AgentProposalCard key={proposal.id} proposal={proposal} />
              ))}
            </div>
          </>
        )}

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

function approvedProposalHasMaterializedProfile(
  proposal: AgentProposal,
  agents: AgentProfile[]
) {
  const proposalTitle = normalizeTitle(proposal.title);
  return agents.some(
    (agent) =>
      agent.status !== "proposed" &&
      (agent.links.includes(proposal.id) || normalizeTitle(agent.title) === proposalTitle)
  );
}

function normalizeTitle(title: string) {
  return title.trim().toLowerCase();
}

function AgentCard({ agent }: { agent: AgentProfile }) {
  const { openDetailArtifact } = useWorkspace();
  const statusColors: Record<string, { color: string; bg: string }> = {
    active: { color: "#46b07d", bg: "rgba(70,176,125,.12)" },
    inactive: { color: "#8a8d99", bg: "rgba(139,141,152,.12)" },
    proposed: { color: "#e0a23a", bg: "rgba(224,162,58,.12)" },
    retired: { color: "#6c6671", bg: "rgba(108,102,113,.12)" },
  };
  const sc = statusColors[agent.status] || statusColors.proposed;
  const hasDomains = agent.domains && agent.domains.length > 0;
  const hasReviewFocus = agent.review_focus && agent.review_focus.length > 0;
  const displayName = agent.mnemonic_name || agent.title;

  return (
    <div
      onClick={() => openDetailArtifact({ title: agent.title, path: agent.path })}
      style={{
        display: "flex",
        alignItems: "flex-start",
        gap: 14,
        background: "var(--bg-tertiary)",
        border: "1px solid var(--border-secondary)",
        borderRadius: 11,
        padding: "14px 16px",
        cursor: "pointer",
      }}
    >
      <div
        style={{
          width: 36,
          height: 36,
          borderRadius: 10,
          background: "rgba(124,108,246,.12)",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          flex: "none",
          marginTop: 2,
        }}
      >
        <i
          className="material-symbols-outlined"
          style={{ fontSize: 18, color: "var(--accent-light)" }}
        >
          smart_toy
        </i>
      </div>
      <div style={{ flex: 1, minWidth: 0 }}>
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: 9,
            marginBottom: 2,
            flexWrap: "wrap",
          }}
        >
          <span
            style={{
              fontFamily: "var(--font-mono)",
              fontSize: 12,
              color: "#bcaef6",
            }}
          >
            {agent.id}
          </span>
          <span
            style={{
              fontSize: 14,
              fontWeight: 600,
              color: "var(--text-primary)",
            }}
          >
            {displayName}
          </span>
          {agent.mnemonic_name && (
            <span
              style={{
                fontSize: 11.5,
                color: "var(--text-tertiary)",
              }}
            >
              {agent.title}
            </span>
          )}
        </div>
        <div
          style={{
            fontSize: 12,
            color: "var(--text-tertiary)",
            marginBottom: hasDomains || hasReviewFocus ? 6 : 0,
          }}
        >
          {agent.role || agent.status}
          {agent.activation ? ` · ${agent.activation}` : ""}
        </div>
        {/* V3 specialization metadata */}
        {hasDomains && (
          <div style={{ display: "flex", flexWrap: "wrap", gap: 4, marginBottom: 4 }}>
            {agent.domains!.map((d) => (
              <span
                key={d}
                style={{
                  fontSize: 10,
                  fontWeight: 600,
                  color: "#7fa8f5",
                  background: "rgba(91,141,239,.1)",
                  borderRadius: 4,
                  padding: "1px 6px",
                }}
              >
                {d}
              </span>
            ))}
          </div>
        )}
        {hasReviewFocus && (
          <div style={{ display: "flex", flexWrap: "wrap", gap: 4 }}>
            {agent.review_focus!.map((f) => (
              <span
                key={f}
                style={{
                  fontSize: 10,
                  color: "#9a949f",
                  background: "rgba(255,255,255,.04)",
                  borderRadius: 4,
                  padding: "1px 6px",
                }}
              >
                {f}
              </span>
            ))}
          </div>
        )}
      </div>
      <span
        style={{
          fontSize: 10.5,
          fontWeight: 700,
          color: sc.color,
          background: sc.bg,
          borderRadius: 5,
          padding: "3px 8px",
          flexShrink: 0,
        }}
      >
        {agent.status.toUpperCase()}
      </span>
    </div>
  );
}

function AgentProposalCard({ proposal }: { proposal: AgentProposal }) {
  const { openDetailArtifact } = useWorkspace();
  const statusColors: Record<string, { color: string; bg: string }> = {
    proposed: { color: "#e0a23a", bg: "rgba(224,162,58,.12)" },
    approved: { color: "#46b07d", bg: "rgba(70,176,125,.12)" },
    rejected: { color: "#e0584a", bg: "rgba(224,88,74,.12)" },
  };
  const sc = statusColors[proposal.status] || statusColors.proposed;
  const isImprovement = proposal.proposal_type === "improvement";

  return (
    <div
      onClick={() => openDetailArtifact({ title: proposal.title, path: proposal.path })}
      style={{
        display: "flex",
        alignItems: "flex-start",
        gap: 14,
        background: "var(--bg-tertiary)",
        border: `1px solid ${isImprovement ? "rgba(91,141,239,.3)" : "var(--border-secondary)"}`,
        borderRadius: 11,
        padding: "14px 16px",
        cursor: "pointer",
      }}
    >
      <div
        style={{
          width: 36,
          height: 36,
          borderRadius: 10,
          background: isImprovement ? "rgba(91,141,239,.12)" : "rgba(224,162,58,.12)",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          flex: "none",
          marginTop: 2,
        }}
      >
        <i
          className="material-symbols-outlined"
          style={{ fontSize: 18, color: isImprovement ? "#7fa8f5" : "#e0a23a" }}
        >
          {isImprovement ? "auto_awesome" : "pending_actions"}
        </i>
      </div>
      <div style={{ flex: 1, minWidth: 0 }}>
        <div style={{ display: "flex", alignItems: "center", gap: 9, marginBottom: 2, flexWrap: "wrap" }}>
          <span style={{ fontFamily: "var(--font-mono)", fontSize: 12, color: "#bcaef6" }}>
            {proposal.id}
          </span>
          <span style={{ fontSize: 14, fontWeight: 600, color: "var(--text-primary)" }}>
            {proposal.title}
          </span>
          {proposal.proposed_mnemonic_name && (
            <span style={{ fontSize: 11.5, color: "var(--text-tertiary)" }}>
              proposes {proposal.proposed_mnemonic_name}
            </span>
          )}
        </div>
        <div style={{ fontSize: 12, color: "var(--text-tertiary)" }}>
          {isImprovement ? "Improvement proposal" : "New-profile proposal"}
          {proposal.target_profile ? ` → ${proposal.target_profile}` : ""}
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
          flexShrink: 0,
        }}
      >
        {proposal.status.toUpperCase()}
      </span>
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
