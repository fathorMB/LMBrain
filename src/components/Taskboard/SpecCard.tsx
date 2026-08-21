import type { Spec } from "../../types";

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

const TIER_COLORS: Record<string, string> = {
  luna: "#5b8def",
  terra: "#46b07d",
  sol: "#e0a23a",
};

export interface SpecCardProps {
  spec: Spec;
  activeDebtCount: number;
  dependencyBlockers: string[];
  onClick: () => void;
}

export function SpecCard({
  spec,
  activeDebtCount,
  dependencyBlockers,
  onClick,
}: SpecCardProps) {
  const { done, total } = criteriaProgress(spec.body);
  const isMalformed = !!spec.malformed;

  return (
    <button
      type="button"
      onClick={onClick}
      aria-label={`View spec ${spec.id}: ${spec.title}`}
      style={{
        width: "100%",
        textAlign: "left",
        fontFamily: "inherit",
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
          style={{ fontFamily: "var(--font-mono)", fontSize: "var(--text-xs)", color: "var(--text-tertiary)" }}
        >
          {spec.id}
        </span>
        {isMalformed && (
          <span
            style={{
              fontSize: "var(--text-2xs)",
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
      <div style={{ fontSize: "var(--text-md)", fontWeight: 600, lineHeight: 1.35, color: "var(--text-primary)" }}>
        {spec.title}
      </div>
      {(spec.tags ?? []).length > 0 && (
        <div style={{ display: "flex", flexWrap: "wrap", gap: "var(--space-1)" }}>
          {(spec.tags ?? []).slice(0, 3).map((tag) => (
            <span
              key={tag}
              style={{
                fontSize: "var(--text-2xs)",
                color: "var(--text-secondary)",
                background: "#1a1722",
                border: "1px solid var(--border-secondary)",
                borderRadius: "var(--radius-pill)",
                padding: "1px 7px",
              }}
            >
              {tag}
            </span>
          ))}
          {(spec.tags ?? []).length > 3 && (
            <span
              title={(spec.tags ?? []).join(", ")}
              style={{ fontSize: "var(--text-2xs)", color: "var(--text-muted)" }}
            >
              +{(spec.tags ?? []).length - 3}
            </span>
          )}
        </div>
      )}
      {spec.status === "backlog" && (spec.parking_events?.length ?? 0) > 0 && (
        <div
          title={spec.parking_events?.at(-1)?.reason}
          style={{ fontSize: "var(--text-xs)", color: "#bcaef6" }}
        >
          Parked · readiness expired
        </div>
      )}
      <div style={{ display: "flex", alignItems: "center", gap: 8, marginTop: 1 }}>
        {spec.capability_tier && (
          <span
            aria-label={`Capability tier ${spec.capability_tier}${spec.thinking_level ? `, ${spec.thinking_level} reasoning` : ""}`}
            title={`Implementation estimate: ${spec.capability_tier}${spec.thinking_level ? ` · ${spec.thinking_level} reasoning` : ""}`}
            style={{
              fontSize: "var(--text-2xs)",
              fontWeight: 700,
              textTransform: "uppercase",
              letterSpacing: ".05em",
              color: TIER_COLORS[spec.capability_tier] ?? "var(--text-secondary)",
              border: `1px solid ${TIER_COLORS[spec.capability_tier] ?? "var(--border-secondary)"}`,
              borderRadius: "var(--radius-sm)",
              padding: "1px 6px",
            }}
          >
            {spec.capability_tier}
          </span>
        )}
        {spec.recommended_agent && (
          <span
            style={{ fontFamily: "var(--font-mono)", fontSize: "var(--text-xs)", color: "#bcaef6" }}
          >
            {spec.recommended_agent}
          </span>
        )}
        {total > 0 && (
          <span
            style={{
              fontFamily: "var(--font-mono)",
              fontSize: "var(--text-xs)",
              color: done === total ? "var(--green)" : "var(--text-secondary)",
              background: "#1a1722",
              borderRadius: 5,
              padding: "2px 6px",
            }}
          >
            {done}/{total}
          </span>
        )}
        {activeDebtCount > 0 && (
          <span
            aria-label={`${activeDebtCount} active debts`}
            title={`${activeDebtCount} active debt${activeDebtCount > 1 ? "s" : ""} linked to this spec`}
            style={{
              fontSize: "var(--text-xs)",
              color: "#d9b86d",
              display: "inline-flex",
              alignItems: "center",
              gap: 3,
            }}
          >
            <i className="material-symbols-outlined" style={{ fontSize: 13 }}>
              link
            </i>{" "}
            {activeDebtCount}
          </span>
        )}
        {dependencyBlockers.length > 0 && (
          <span
            aria-label={`Blocked by hard dependencies: ${dependencyBlockers.join(", ")}`}
            title={`Ready after ${dependencyBlockers.join(", ")}`}
            style={{ fontSize: "var(--text-xs)", color: "#e0a23a" }}
          >
            ⛓ {dependencyBlockers.length}
          </span>
        )}
        <span style={{ flex: 1 }} />
        <span style={{ fontSize: "var(--text-xs)", color: "var(--text-muted)", whiteSpace: "nowrap" }}>
          {spec.updated}
        </span>
      </div>
    </button>
  );
}
