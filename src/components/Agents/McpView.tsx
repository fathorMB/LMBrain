import { useEffect, useState } from "react";
import { useWorkspace } from "../../hooks/useWorkspace";
import { getMcpRecords, getMcpProposals } from "../../lib/commands";
import { CardGrid, PageHeader, PageShell } from "../Shared/PageLayout";
import type { McpRecord, McpProposal } from "../../types";
import { LMBRAIN_MCP_TOOLS } from "../../lib/mcpCatalog";

export function McpView() {
  const { state } = useWorkspace();
  const [localRecords, setLocalRecords] = useState<McpRecord[] | null>(null);
  const [localProposals, setLocalProposals] = useState<McpProposal[] | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [reloadRevision, setReloadRevision] = useState(0);

  const mcpRecords = localRecords ?? state.mcpRecords;
  const mcpProposals = localProposals ?? state.mcpProposals;

  useEffect(() => {
    let cancelled = false;

    Promise.all([
      getMcpRecords(),
      getMcpProposals(),
    ])
      .then(([records, proposals]) => {
        if (cancelled) return;
        setLocalRecords(records);
        setLocalProposals(proposals);
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
  }, [reloadRevision]);

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
          {!isLoading && !loadError && mcpRecords.length === 0 && (
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
          {!isLoading && !loadError && mcpRecords.length > 0 && (
            <CardGrid>
              {mcpRecords.map((mcp) => (
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
