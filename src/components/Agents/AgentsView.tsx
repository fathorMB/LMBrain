import { useEffect, useState } from "react";
import { useWorkspace } from "../../hooks/useWorkspace";
import { getAgentImprovementInsights, getAgentProposals, getAgents } from "../../lib/commands";
import { CardGrid, EmptyState, PageHeader, PageShell } from "../Shared/PageLayout";
import type { AgentImprovementInsights, AgentProfile, AgentProposal } from "../../types";

export function AgentsView() {
  const { state, dispatch } = useWorkspace();
  const [insights, setInsights] = useState<AgentImprovementInsights>({ signals: [], metrics: [] });
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
      getAgentImprovementInsights(),
    ])
      .then(([agents, agentProposals, improvementInsights]) => {
        dispatch({ type: "SET_AGENTS", agents });
        dispatch({ type: "SET_AGENT_PROPOSALS", proposals: agentProposals });
        setInsights(improvementInsights);
      })
      .catch(console.error);
  }, [dispatch]);

  return (
    <PageShell archetype="dense">
      <PageHeader
        title="Agents"
        description="Agent profiles and behavior guidelines. All agents are started manually — LMBrain never auto-launches."
      />

        {/* Agent Profiles */}
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
          Agent Profiles
        </div>
        <div style={{ marginBottom: "var(--space-6)" }}>
          {state.agents.length === 0 ? (
            <EmptyState>No agent profiles found.</EmptyState>
          ) : (
            <CardGrid>
              {state.agents.map((agent) => (
                <AgentCard key={agent.id} agent={agent} />
              ))}
            </CardGrid>
          )}
        </div>

        {(insights.metrics.length > 0 || insights.signals.length > 0) && (
          <>
            <div style={{ fontSize: "var(--text-xs)", letterSpacing: ".09em", textTransform: "uppercase", color: "var(--text-tertiary)", fontWeight: 600, marginBottom: 11 }}>
              Governed improvement signals
            </div>
            <div style={{ display: "flex", flexDirection: "column", gap: 8, marginBottom: 32 }}>
              {insights.metrics.map((metric) => (
                <div key={metric.profile} style={{ background: "var(--bg-tertiary)", border: "1px solid var(--border-secondary)", borderRadius: 11, padding: "12px 15px" }}>
                  <div style={{ fontFamily: "var(--font-mono)", color: "#bcaef6", fontSize: "var(--text-sm)", marginBottom: 5 }}>{metric.profile}</div>
                  <div style={{ color: "var(--text-tertiary)", fontSize: "var(--text-sm)" }}>
                    {metric.reviewed_specs} specs · {metric.review_cycles} cycles · {metric.specs_with_changes_requested} changed · {metric.transcript_fast_fail_reviews} transcript fast-fails
                  </div>
                  <div style={{ color: "var(--text-tertiary)", fontSize: "var(--text-xs)", marginTop: 4 }}>
                    {metric.remediation_cycles} remediations · {metric.escalation_count} escalations · {metric.takeover_count} takeovers
                  </div>
                  <div style={{ color: metric.confidence === "low" ? "#e0a23a" : "var(--text-tertiary)", fontSize: "var(--text-xs)", marginTop: 4 }}>
                    Lifecycle {Math.round(metric.lifecycle_coverage * 100)}% · taxonomy {Math.round(metric.category_coverage * 100)}% · {metric.confidence} confidence
                  </div>
                  <div style={{ color: "var(--text-tertiary)", fontSize: "var(--text-xs)", marginTop: 4 }}>
                    Attribution: {metric.attribution_basis.replaceAll("-", " ")}
                  </div>
                  {metric.unknown_categories.length > 0 && (
                    <div role="status" style={{ color: "#e0a23a", fontSize: "var(--text-xs)", marginTop: 4 }}>
                      Unknown categories: {metric.unknown_categories.join(", ")}
                    </div>
                  )}
                </div>
              ))}
              {insights.signals.filter((signal) => signal.threshold_met).map((signal) => (
                <div key={`${signal.target_profile}:${signal.category}`} style={{ background: "rgba(224,162,58,.07)", border: "1px solid rgba(224,162,58,.25)", borderRadius: 11, padding: "12px 15px" }}>
                  <div style={{ color: "#e0a23a", fontSize: "var(--text-sm)", fontWeight: 650 }}>{signal.category} → {signal.target_profile}</div>
                  <div style={{ color: "var(--text-tertiary)", fontSize: "var(--text-sm)", marginTop: 4 }}>{signal.distinct_specs.length} distinct specs · proposal remains operator-governed</div>
                </div>
              ))}
            </div>
          </>
        )}

        {/* Agent Proposals */}
        {visibleAgentProposals.length > 0 && (
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
              Agent Proposals
            </div>
            <div style={{ marginBottom: "var(--space-6)" }}>
              <CardGrid>
                {visibleAgentProposals.map((proposal) => (
                  <AgentProposalCard key={proposal.id} proposal={proposal} />
                ))}
              </CardGrid>
            </div>
          </>
        )}
    </PageShell>
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
    retired: { color: "var(--text-tertiary)", bg: "rgba(108,102,113,.12)" },
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
              fontSize: "var(--text-sm)",
              color: "#bcaef6",
            }}
          >
            {agent.id}
          </span>
          <span
            style={{
              fontSize: "var(--text-md)",
              fontWeight: 600,
              color: "var(--text-primary)",
            }}
          >
            {displayName}
          </span>
          {agent.mnemonic_name && (
            <span
              style={{
                fontSize: "var(--text-xs)",
                color: "var(--text-tertiary)",
              }}
            >
              {agent.title}
            </span>
          )}
        </div>
        <div
          style={{
            fontSize: "var(--text-sm)",
            color: "var(--text-tertiary)",
            marginBottom: hasDomains || hasReviewFocus ? 6 : 0,
          }}
        >
          {agent.role || agent.status}
          {agent.activation ? ` · ${agent.activation}` : ""}
        </div>
        {hasDomains && (
          <div style={{ display: "flex", flexWrap: "wrap", gap: 4, marginBottom: 4 }}>
            {agent.domains!.map((d) => (
              <span
                key={d}
                style={{
                  fontSize: "var(--text-2xs)",
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
                  fontSize: "var(--text-2xs)",
                  color: "var(--text-secondary)",
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
          fontSize: "var(--text-xs)",
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
          <span style={{ fontFamily: "var(--font-mono)", fontSize: "var(--text-sm)", color: "#bcaef6" }}>
            {proposal.id}
          </span>
          <span style={{ fontSize: "var(--text-md)", fontWeight: 600, color: "var(--text-primary)" }}>
            {proposal.title}
          </span>
          {proposal.proposed_mnemonic_name && (
            <span style={{ fontSize: "var(--text-xs)", color: "var(--text-tertiary)" }}>
              proposes {proposal.proposed_mnemonic_name}
            </span>
          )}
        </div>
        <div style={{ fontSize: "var(--text-sm)", color: "var(--text-tertiary)" }}>
          {isImprovement ? "Improvement proposal" : "New-profile proposal"}
          {proposal.target_profile ? ` → ${proposal.target_profile}` : ""}
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
          flexShrink: 0,
        }}
      >
        {proposal.status.toUpperCase()}
      </span>
    </div>
  );
}
