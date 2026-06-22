import { useEffect } from "react";
import { useWorkspace } from "../../hooks/useWorkspace";
import { getAdrs } from "../../lib/commands";

export function DecisionsList() {
  const { state, dispatch } = useWorkspace();

  useEffect(() => {
    getAdrs()
      .then((adrs) => dispatch({ type: "SET_ADRS", adrs }))
      .catch(console.error);
  }, [dispatch]);

  const statusColors: Record<string, { color: string; bg: string }> = {
    accepted: { color: "#46b07d", bg: "rgba(70,176,125,.12)" },
    proposed: { color: "#8a8d99", bg: "rgba(139,141,152,.12)" },
    superseded: { color: "#e0a23a", bg: "rgba(224,162,58,.12)" },
    deprecated: { color: "#e0584a", bg: "rgba(224,88,74,.12)" },
  };

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
          Decisions
        </h1>
        <p
          style={{
            fontSize: 13.5,
            color: "var(--text-tertiary)",
            margin: "0 0 22px",
          }}
        >
          Architecture decision records in{" "}
          <span
            style={{
              fontFamily: "var(--font-mono)",
              fontSize: 12,
              color: "#9a949f",
            }}
          >
            .lmbrain/decisions/
          </span>
          .
        </p>

        <div style={{ display: "flex", flexDirection: "column", gap: 9 }}>
          {state.adrs.length === 0 && (
            <div
              style={{
                textAlign: "center",
                padding: 40,
                color: "var(--text-tertiary)",
              }}
            >
              No decisions recorded yet.
            </div>
          )}
          {state.adrs.map((adr) => {
            const sc = statusColors[adr.status] || statusColors.proposed;
            const isMalformed = !!adr.malformed;
            return (
              <div
                key={adr.id}
                onClick={() => dispatch({ type: "SET_DETAIL_ARTIFACT", artifact: { title: adr.title, path: adr.path } })}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 14,
                  background: "var(--bg-tertiary)",
                  border: isMalformed ? "1px solid #e0584a" : "1px solid var(--border-secondary)",
                  borderRadius: 11,
                  padding: "14px 16px",
                  cursor: "pointer",
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.borderColor = isMalformed ? "#f06f60" : "#36303f";
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.borderColor = isMalformed ? "#e0584a" : "var(--border-secondary)";
                }}
              >
                <span
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: 12,
                    color: "#bcaef6",
                    width: 62,
                    flex: "none",
                  }}
                >
                  {adr.id}
                </span>
                <div style={{ flex: 1 }}>
                  <div
                    style={{
                      fontSize: 14,
                      fontWeight: 600,
                      color: "var(--text-primary)",
                      display: "flex",
                      alignItems: "center",
                      gap: 8,
                    }}
                  >
                    {adr.title}
                    {isMalformed && (
                      <span
                        style={{
                          display: "inline-flex",
                          alignItems: "center",
                          gap: 4,
                          fontSize: 10,
                          fontWeight: 700,
                          color: "#e0584a",
                          background: "rgba(224,88,74,0.13)",
                          borderRadius: 5,
                          padding: "2px 6px",
                        }}
                      >
                        <i className="material-symbols-outlined" style={{ fontSize: 11 }}>warning</i>
                        MALFORMED
                      </span>
                    )}
                  </div>
                  <div
                    style={{
                      fontSize: 11.5,
                      color: "#6c6671",
                    }}
                  >
                    {adr.status} {adr.decision_date ? `· ${adr.decision_date}` : ""}
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
                  {adr.status.toUpperCase()}
                </span>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
