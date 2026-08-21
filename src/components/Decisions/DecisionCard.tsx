import type { CSSProperties } from "react";
import type { Adr } from "../../types";
import { supersessionChain } from "../../lib/decisionIndex";
import { STATUS_COLORS } from "./decisionColors";

export interface DecisionCardProps {
  adr: Adr;
  byId: Map<string, Adr>;
  inbound: number;
  onOpen: (adr: { title: string; path: string }) => void;
}

export function DecisionCard({
  adr,
  byId,
  inbound,
  onOpen,
}: DecisionCardProps) {
  const status = STATUS_COLORS[adr.status] ?? STATUS_COLORS.proposed;
  const isMalformed = Boolean(adr.malformed);
  const retires = supersessionChain(adr, byId, "supersedes")[0];
  const retiredBy = supersessionChain(adr, byId, "superseded_by")[0];
  const provenance = [adr.decision_date, adr.decider].filter(Boolean).join(" · ");

  const cardStyle: CSSProperties = {
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
    cursor: "pointer",
  };

  return (
    <button
      type="button"
      aria-label={`${adr.id}, ${adr.title}, ${adr.status}`}
      onClick={() => onOpen(adr)}
      style={cardStyle}
      onMouseEnter={(event) => {
        event.currentTarget.style.borderColor = isMalformed ? "#f06f60" : "var(--border-hover)";
      }}
      onMouseLeave={(event) => {
        event.currentTarget.style.borderColor = isMalformed ? "#e0584a" : "var(--border-secondary)";
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: "var(--space-2)" }}>
        <span
          style={{ fontFamily: "var(--font-mono)", fontSize: "var(--text-sm)", color: "#bcaef6" }}
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

      {provenance && (
        <div style={{ fontSize: "var(--text-xs)", color: "var(--text-tertiary)" }}>{provenance}</div>
      )}

      {(retires || retiredBy || inbound > 0) && (
        <div
          style={{
            display: "flex",
            flexWrap: "wrap",
            gap: "var(--space-3)",
            fontSize: "var(--text-xs)",
            color: "var(--text-tertiary)",
          }}
        >
          {retires && <span>replaces {retires.id}</span>}
          {retiredBy && <span>replaced by {retiredBy.id}</span>}
          {inbound > 0 && (
            <span>
              {inbound} {inbound === 1 ? "reference" : "references"}
            </span>
          )}
        </div>
      )}
    </button>
  );
}
