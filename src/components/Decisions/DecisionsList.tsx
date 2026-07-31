import { useEffect } from "react";
import { useWorkspace } from "../../hooks/useWorkspace";
import { getAdrs } from "../../lib/commands";
import { CardGrid, EmptyState, PageHeader, PageShell } from "../Shared/PageLayout";

const STATUS_COLORS: Record<string, { color: string; bg: string }> = {
  accepted: { color: "#46b07d", bg: "rgba(70,176,125,.12)" },
  proposed: { color: "#8a8d99", bg: "rgba(139,141,152,.12)" },
  superseded: { color: "#e0a23a", bg: "rgba(224,162,58,.12)" },
  deprecated: { color: "#e0584a", bg: "rgba(224,88,74,.12)" },
};

export function DecisionsList() {
  const { state, dispatch } = useWorkspace();

  useEffect(() => {
    getAdrs()
      .then((adrs) => dispatch({ type: "SET_ADRS", adrs }))
      .catch(console.error);
  }, [dispatch]);

  return (
    <PageShell archetype="dense">
      <PageHeader
        title="Decisions"
        description={
          <>
            Architecture decision records in{" "}
            <span style={{ fontFamily: "var(--font-mono)", fontSize: "var(--text-sm)", color: "var(--text-secondary)" }}>
              .lmbrain/decisions/
            </span>
            .
          </>
        }
      />

      {state.adrs.length === 0 ? (
        <EmptyState>No decisions recorded yet.</EmptyState>
      ) : (
        <CardGrid>
          {state.adrs.map((adr) => {
            const status = STATUS_COLORS[adr.status] ?? STATUS_COLORS.proposed;
            const isMalformed = Boolean(adr.malformed);
            return (
              <button
                key={adr.id}
                type="button"
                onClick={() =>
                  dispatch({ type: "SET_DETAIL_ARTIFACT", artifact: { title: adr.title, path: adr.path } })
                }
                style={{
                  display: "flex",
                  flexDirection: "column",
                  gap: "var(--space-2)",
                  width: "100%",
                  textAlign: "left",
                  font: "inherit",
                  color: "inherit",
                  background: "var(--bg-tertiary)",
                  border: `1px solid ${isMalformed ? "#e0584a" : "var(--border-secondary)"}`,
                  borderRadius: "var(--radius-lg)",
                  padding: "var(--space-3)",
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.borderColor = isMalformed ? "#f06f60" : "var(--border-hover)";
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.borderColor = isMalformed ? "#e0584a" : "var(--border-secondary)";
                }}
              >
                <div style={{ display: "flex", alignItems: "center", gap: "var(--space-2)" }}>
                  <span
                    style={{
                      fontFamily: "var(--font-mono)",
                      fontSize: "var(--text-sm)",
                      color: "#bcaef6",
                    }}
                  >
                    {adr.id}
                  </span>
                  <span style={{ flex: 1 }} />
                  {isMalformed && (
                    <span
                      style={{
                        display: "inline-flex",
                        alignItems: "center",
                        gap: "var(--space-1)",
                        fontSize: "var(--text-2xs)",
                        fontWeight: 700,
                        color: "#e0584a",
                        background: "rgba(224,88,74,0.13)",
                        borderRadius: "var(--radius-sm)",
                        padding: "var(--space-0) var(--space-2)",
                      }}
                    >
                      <i className="material-symbols-outlined" style={{ fontSize: "var(--text-xs)" }}>
                        warning
                      </i>
                      MALFORMED
                    </span>
                  )}
                  <span
                    style={{
                      fontSize: "var(--text-2xs)",
                      fontWeight: 700,
                      color: status.color,
                      background: status.bg,
                      borderRadius: "var(--radius-sm)",
                      padding: "var(--space-0) var(--space-2)",
                    }}
                  >
                    {adr.status.toUpperCase()}
                  </span>
                </div>

                <div style={{ fontSize: "var(--text-lg)", fontWeight: 600, color: "var(--text-primary)" }}>
                  {adr.title}
                </div>

                <div style={{ fontSize: "var(--text-xs)", color: "var(--text-tertiary)" }}>
                  {adr.status}
                  {adr.decision_date ? ` · ${adr.decision_date}` : ""}
                </div>
              </button>
            );
          })}
        </CardGrid>
      )}
    </PageShell>
  );
}
