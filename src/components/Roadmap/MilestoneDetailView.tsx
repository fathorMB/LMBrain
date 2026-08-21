import { useWorkspace } from "../../hooks/useWorkspace";
import { InlineRichText } from "../../lib/inlineRichText";
import { useWikiNavigation } from "../../hooks/useWikiNavigation";
import type { MilestoneDetail } from "../../types";

export interface MilestoneDetailViewProps {
  milestone: MilestoneDetail;
}

const statusColors: Record<string, { color: string; bg: string }> = {
  active: { color: "#5b8def", bg: "rgba(91,141,239,0.13)" },
  planned: { color: "#8a8d99", bg: "rgba(138,141,153,0.13)" },
  completed: { color: "#46b07d", bg: "rgba(70,176,125,0.13)" },
};

const specStatusColors: Record<string, { color: string; bg: string }> = {
  backlog: { color: "#8a8d99", bg: "rgba(138,141,153,0.1)" },
  ready: { color: "#7c6cf6", bg: "rgba(124,108,246,0.12)" },
  working: { color: "#5b8def", bg: "rgba(91,141,239,0.12)" },
  review: { color: "#e0a23a", bg: "rgba(224,162,58,0.12)" },
  done: { color: "#46b07d", bg: "rgba(70,176,125,0.12)" },
  discarded: { color: "var(--text-tertiary)", bg: "rgba(108,102,113,0.12)" },
};

export function MilestoneDetailView({ milestone }: MilestoneDetailViewProps) {
  const { openDetailArtifact } = useWorkspace();
  const navigateToWiki = useWikiNavigation();
  const sc = statusColors[milestone.status] ?? { color: "#8a8d99", bg: "rgba(138,141,153,0.13)" };

  return (
    <div
      style={{
        flex: 1,
        minWidth: 0,
        background: "var(--bg-tertiary)",
        border: "1px solid var(--border-secondary)",
        borderRadius: 13,
        padding: 24,
        display: "flex",
        flexDirection: "column",
        gap: 20,
      }}
    >
      {/* Header */}
      <div>
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 6 }}>
          <span style={{ fontSize: "var(--text-xs)", fontFamily: "var(--font-mono)", color: "var(--text-tertiary)" }}>
            {milestone.id}
          </span>
          <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
            <span style={{ fontSize: "var(--text-xs)", fontWeight: 700, color: sc.color, background: sc.bg, borderRadius: 5, padding: "3px 8px", letterSpacing: "0.03em" }}>
              {milestone.status.toUpperCase()}
            </span>
          </div>
        </div>
        <h2 style={{ fontSize: "var(--text-xl)", fontWeight: 700, margin: "0 0 8px", color: "var(--text-primary)" }}>
          <InlineRichText text={milestone.title} onWikilinkClick={navigateToWiki} />
        </h2>
        {milestone.outcome && (
          <div style={{ fontSize: "var(--text-sm)", color: "var(--text-secondary)", fontStyle: "italic" }}>
            <InlineRichText text={milestone.outcome} onWikilinkClick={navigateToWiki} />
          </div>
        )}
      </div>

      {/* Progress bar */}
      {milestone.progress_pct > 0 && (
        <div>
          <div style={{ display: "flex", justifyContent: "space-between", fontSize: "var(--text-xs)", color: "var(--text-tertiary)", marginBottom: 6 }}>
            <span>Progress</span>
            <span style={{ fontFamily: "var(--font-mono)", fontWeight: 600, color: "var(--text-primary)" }}>
              {Math.round(milestone.progress_pct)}%
            </span>
          </div>
          <div style={{ height: 6, background: "var(--bg-secondary)", borderRadius: 3, overflow: "hidden" }}>
            <div style={{ width: `${milestone.progress_pct}%`, height: "100%", background: "linear-gradient(90deg,#7c6cf6,#9384f8)", borderRadius: 3 }} />
          </div>
        </div>
      )}

      {/* Spec counts by status */}
      {milestone.spec_count > 0 && (
        <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
          {Object.entries(milestone.spec_counts_by_status).map(([status, count]) => {
            const sc2 = specStatusColors[status] || { color: "var(--text-tertiary)", bg: "var(--bg-secondary)" };
            return (
              <span key={status} style={{ fontSize: "var(--text-xs)", fontWeight: 700, color: sc2.color, background: sc2.bg, borderRadius: 5, padding: "3px 8px" }}>
                {status}: {count}
              </span>
            );
          })}
        </div>
      )}

      {/* Next action */}
      {milestone.next_action && (
        <div style={{ display: "flex", alignItems: "center", gap: 8, fontSize: "var(--text-sm)", color: "var(--accent-light)", background: "rgba(124,108,246,0.08)", borderRadius: 8, padding: "8px 12px" }}>
          <i className="material-symbols-outlined" style={{ fontSize: 16 }}>arrow_forward</i>
          <span>{milestone.next_action}</span>
        </div>
      )}

      {/* Specs list */}
      {milestone.specs.length > 0 && (
        <div>
          <div style={{ fontSize: "var(--text-xs)", fontWeight: 600, color: "var(--text-tertiary)", marginBottom: 8 }}>
            Specifications ({milestone.specs.length})
          </div>
          <div style={{ display: "flex", flexDirection: "column", gap: 5 }}>
            {milestone.specs.map((spec) => {
              const sc2 = specStatusColors[spec.status] || { color: "var(--text-tertiary)", bg: "var(--bg-secondary)" };
              return (
                <button
                  type="button"
                  key={spec.id}
                  onClick={() => {
                    const specPath = spec.path || `.lmbrain/specs/${spec.status}/${spec.id}.md`;
                    openDetailArtifact({ title: spec.title, path: specPath });
                  }}
                  style={{
                    width: "100%",
                    display: "flex",
                    alignItems: "center",
                    gap: 8,
                    padding: "7px 10px",
                    borderRadius: 8,
                    background: "rgba(255,255,255,0.02)",
                    border: "1px solid rgba(255,255,255,0.05)",
                    cursor: "pointer",
                    textAlign: "left",
                    fontFamily: "inherit",
                  }}
                >
                  <span style={{ fontFamily: "var(--font-mono)", fontSize: "var(--text-xs)", color: "#bcaef6", flex: "none" }}>
                    {spec.id}
                  </span>
                  <span style={{ fontSize: "var(--text-sm)", color: "var(--text-primary)", flex: 1, minWidth: 0, whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" }}>
                    {spec.title}
                  </span>
                  <span style={{ fontSize: "var(--text-2xs)", fontWeight: 700, color: sc2.color, background: sc2.bg, borderRadius: 4, padding: "1px 6px", flex: "none" }}>
                    {spec.status}
                  </span>
                  {spec.priority && (
                    <span style={{ fontSize: "var(--text-2xs)", color: "var(--text-muted)", flex: "none" }}>
                      {spec.priority}
                    </span>
                  )}
                  {spec.recommended_agent && (
                    <span style={{ fontSize: "var(--text-2xs)", color: "var(--text-muted)", flex: "none", fontFamily: "var(--font-mono)" }}>
                      {spec.recommended_agent}
                    </span>
                  )}
                </button>
              );
            })}
          </div>
        </div>
      )}

      {/* Reviews */}
      {milestone.reviews.length > 0 && (
        <div>
          <div style={{ fontSize: "var(--text-xs)", fontWeight: 600, color: "var(--text-tertiary)", marginBottom: 8 }}>
            Reviews ({milestone.reviews.length})
          </div>
          <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
            {milestone.reviews.map((r) => (
              <button
                type="button"
                key={r.id}
                disabled={!r.path}
                onClick={() => r.path && openDetailArtifact({ title: r.title, path: r.path })}
                style={{
                  width: "100%",
                  display: "flex",
                  alignItems: "center",
                  gap: 8,
                  fontSize: "var(--text-sm)",
                  color: "var(--text-secondary)",
                  cursor: r.path ? "pointer" : "default",
                  padding: "3px 6px",
                  borderRadius: 6,
                  border: "none",
                  background: "transparent",
                  textAlign: "left",
                  fontFamily: "inherit",
                }}
                onMouseEnter={(e) => { if (r.path) { e.currentTarget.style.background = "rgba(255,255,255,0.03)"; } }}
                onMouseLeave={(e) => { e.currentTarget.style.background = "transparent"; }}
              >
                <span style={{ fontFamily: "var(--font-mono)", fontSize: "var(--text-xs)", color: "#bcaef6" }}>{r.id}</span>
                <span>{r.title}</span>
                <span style={{ fontSize: "var(--text-2xs)", fontWeight: 700, color: r.status === "accepted" ? "#46b07d" : "#e0a23a", background: r.status === "accepted" ? "rgba(70,176,125,0.1)" : "rgba(224,162,58,0.1)", borderRadius: 4, padding: "1px 6px" }}>
                  {r.status}
                </span>
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Decisions */}
      {milestone.decisions.length > 0 && (
        <div>
          <div style={{ fontSize: "var(--text-xs)", fontWeight: 600, color: "var(--text-tertiary)", marginBottom: 8 }}>
            Decisions ({milestone.decisions.length})
          </div>
          <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
            {milestone.decisions.map((d) => (
              <button
                type="button"
                key={d.id}
                disabled={!d.path}
                onClick={() => d.path && openDetailArtifact({ title: d.title, path: d.path })}
                style={{
                  width: "100%",
                  display: "flex",
                  alignItems: "center",
                  gap: 8,
                  fontSize: "var(--text-sm)",
                  color: "var(--text-secondary)",
                  cursor: d.path ? "pointer" : "default",
                  padding: "3px 6px",
                  borderRadius: 6,
                  border: "none",
                  background: "transparent",
                  textAlign: "left",
                  fontFamily: "inherit",
                }}
                onMouseEnter={(e) => { if (d.path) { e.currentTarget.style.background = "rgba(255,255,255,0.03)"; } }}
                onMouseLeave={(e) => { e.currentTarget.style.background = "transparent"; }}
              >
                <span style={{ fontFamily: "var(--font-mono)", fontSize: "var(--text-xs)", color: "#bcaef6" }}>{d.id}</span>
                <span>{d.title}</span>
                <span style={{ fontSize: "var(--text-2xs)", fontWeight: 700, color: d.status === "accepted" ? "#46b07d" : "#8a8d99", background: d.status === "accepted" ? "rgba(70,176,125,0.1)" : "rgba(138,141,153,0.1)", borderRadius: 4, padding: "1px 6px" }}>
                  {d.status}
                </span>
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Risks */}
      {milestone.risks.length > 0 && (
        <div>
          <div style={{ fontSize: "var(--text-xs)", fontWeight: 600, color: "var(--text-tertiary)", marginBottom: 6 }}>
            Risks
          </div>
          <ul style={{ margin: 0, paddingLeft: 18, fontSize: "var(--text-sm)", color: "#f0a89f", display: "flex", flexDirection: "column", gap: 3 }}>
            {milestone.risks.map((risk, i) => (
              <li key={i}>{risk}</li>
            ))}
          </ul>
        </div>
      )}

      {/* Dependencies */}
      {milestone.depends_on && (
        <div style={{ fontSize: "var(--text-sm)", color: "var(--text-tertiary)" }}>
          <span style={{ fontWeight: 600 }}>Depends on:</span> {milestone.depends_on}
        </div>
      )}

      {/* Unresolved references */}
      {milestone.unresolved_refs.length > 0 && (
        <div>
          <div style={{ fontSize: "var(--text-xs)", fontWeight: 600, color: "#e0a23a", marginBottom: 6 }}>
            Unresolved References
          </div>
          <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
            {milestone.unresolved_refs.map((ref, i) => (
              <div key={i} style={{ fontSize: "var(--text-sm)", color: "#f0a89f", display: "flex", alignItems: "center", gap: 6 }}>
                <i className="material-symbols-outlined" style={{ fontSize: 14 }}>warning</i>
                {ref}
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
