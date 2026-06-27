import { useEffect } from "react";
import { useWorkspace } from "../../hooks/useWorkspace";
import { getSpecs } from "../../lib/commands";
import type { Spec, SpecStatus } from "../../types";

const COLUMNS: { status: SpecStatus; label: string; color: string }[] = [
  { status: "backlog", label: "Backlog", color: "#6c6671" },
  { status: "ready", label: "Ready", color: "#8a8d99" },
  { status: "working", label: "Working", color: "#5b8def" },
  { status: "review", label: "Review", color: "#e0a23a" },
  { status: "done", label: "Done", color: "#46b07d" },
  { status: "discarded", label: "Discarded", color: "#e0584a" },
];

function criteriaProgress(body: string): { done: number; total: number } {
  let done = 0;
  let total = 0;
  for (const line of body.split("\n")) {
    const t = line.trimStart();
    if (t.startsWith("- [x]") || t.startsWith("- [X]")) {
      done += 1;
      total += 1;
    } else if (t.startsWith("- [ ]")) {
      total += 1;
    }
  }
  return { done, total };
}

export function TaskboardView() {
  const { state, dispatch, openSpec } = useWorkspace();

  useEffect(() => {
    getSpecs()
      .then((specs) => dispatch({ type: "SET_SPECS", specs }))
      .catch(console.error);
  }, [dispatch]);

  const specsByStatus = (status: SpecStatus) =>
    state.specs.filter((s) => s.status === status);

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%", minHeight: 0 }}>
      {/* Header */}
      <div
        style={{
          flex: "none",
          padding: "20px 24px 14px",
          borderBottom: "1px solid var(--border-primary)",
        }}
      >
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            marginBottom: 14,
          }}
        >
          <h1 style={{ fontSize: 24, fontWeight: 800, letterSpacing: "-.025em", margin: 0 }}>
            Board
          </h1>
          <div
            style={{
              fontSize: 12,
              color: "var(--text-tertiary)",
              display: "flex",
              alignItems: "center",
              gap: 7,
            }}
          >
            <i className="material-symbols-outlined" style={{ fontSize: 15, color: "var(--green)" }}>
              cloud_done
            </i>
            backed by{" "}
            <span style={{ fontFamily: "var(--font-mono)", fontSize: 11.5, color: "#9a949f" }}>
              .lmbrain/specs/&lt;status&gt;/*.md
            </span>
          </div>
        </div>
        <div
          style={{
            fontSize: 11.5,
            color: "#56525b",
            display: "flex",
            alignItems: "center",
            gap: 6,
          }}
        >
          <i className="material-symbols-outlined" style={{ fontSize: 15 }}>
            visibility
          </i>
          Read-only view · specs move through these states via the `lmbrain-mcp` tools
        </div>
      </div>

      {/* Columns */}
      <div style={{ flex: 1, minHeight: 0, overflowX: "auto", overflowY: "hidden", padding: "16px 24px" }}>
        <div style={{ display: "flex", gap: 14, height: "100%", minWidth: "max-content" }}>
          {COLUMNS.map((col) => {
            const specs = specsByStatus(col.status);
            return (
              <div
                key={col.status}
                style={{
                  width: 262,
                  flex: "none",
                  display: "flex",
                  flexDirection: "column",
                  minHeight: 0,
                }}
              >
                <div
                  style={{ display: "flex", alignItems: "center", gap: 8, padding: "0 4px 11px" }}
                >
                  <span
                    style={{ width: 9, height: 9, borderRadius: "50%", background: col.color }}
                  />
                  <span style={{ fontSize: 12.5, fontWeight: 700, color: "var(--text-primary)" }}>
                    {col.label}
                  </span>
                  <span
                    style={{ fontFamily: "var(--font-mono)", fontSize: 11, color: "#56525b" }}
                  >
                    {specs.length}
                  </span>
                </div>
                <div
                  style={{
                    display: "flex",
                    flexDirection: "column",
                    gap: 9,
                    flex: 1,
                    minHeight: 0,
                    overflowY: "auto",
                    paddingRight: 2,
                  }}
                >
                  {specs.map((spec) => (
                    <SpecCard key={spec.id} spec={spec} onClick={() => openSpec(spec)} />
                  ))}
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}

function SpecCard({ spec, onClick }: { spec: Spec; onClick: () => void }) {
  const { done, total } = criteriaProgress(spec.body);
  const isMalformed = !!spec.malformed;

  return (
    <div
      onClick={onClick}
      style={{
        background: "var(--bg-tertiary)",
        border: isMalformed ? "1px solid #e0584a" : "1px solid #262330",
        borderRadius: 11,
        padding: "12px 13px",
        cursor: "pointer",
        display: "flex",
        flexDirection: "column",
        gap: 8,
      }}
      onMouseEnter={(e) => {
        e.currentTarget.style.borderColor = isMalformed ? "#f06f60" : "#3a3446";
        e.currentTarget.style.background = "#181520";
      }}
      onMouseLeave={(e) => {
        e.currentTarget.style.borderColor = isMalformed ? "#e0584a" : "#262330";
        e.currentTarget.style.background = "var(--bg-tertiary)";
      }}
    >
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
        <span
          style={{ fontFamily: "var(--font-mono)", fontSize: 11, color: "var(--text-tertiary)" }}
        >
          {spec.id}
        </span>
        {isMalformed && (
          <span
            style={{
              fontSize: 10,
              fontWeight: 700,
              color: "#e0584a",
              background: "rgba(224,88,74,0.13)",
              borderRadius: 5,
              padding: "2px 7px",
              letterSpacing: "0.03em",
            }}
          >
            MALFORMED
          </span>
        )}
      </div>
      <div style={{ fontSize: 13, fontWeight: 600, lineHeight: 1.35, color: "var(--text-primary)" }}>
        {spec.title}
      </div>
      <div style={{ display: "flex", alignItems: "center", gap: 8, marginTop: 1 }}>
        {spec.recommended_agent && (
          <span
            style={{ fontFamily: "var(--font-mono)", fontSize: 10.5, color: "#bcaef6" }}
          >
            {spec.recommended_agent}
          </span>
        )}
        {total > 0 && (
          <span
            style={{
              fontFamily: "var(--font-mono)",
              fontSize: 10.5,
              color: done === total ? "var(--green)" : "#9a949f",
              background: "#1a1722",
              borderRadius: 5,
              padding: "2px 6px",
            }}
          >
            {done}/{total}
          </span>
        )}
        <span style={{ flex: 1 }} />
        <span style={{ fontSize: 10.5, color: "#56525b", whiteSpace: "nowrap" }}>
          {spec.updated}
        </span>
      </div>
    </div>
  );
}
