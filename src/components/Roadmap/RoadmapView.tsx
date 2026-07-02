import { useEffect, useState } from "react";
import { useWorkspace } from "../../hooks/useWorkspace";
import { getMilestoneOverview } from "../../lib/commands";
import { InlineRichText } from "../../lib/inlineRichText";
import { useWikiNavigation } from "../../hooks/useWikiNavigation";
import type { MilestoneOverview, MilestoneDetail } from "../../types";

export function RoadmapView() {
  const { dispatch, openDetailArtifact } = useWorkspace();
  const [overview, setOverview] = useState<MilestoneOverview | null>(null);
  const [loading, setLoading] = useState(true);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const navigateToWiki = useWikiNavigation();

  useEffect(() => {
    getMilestoneOverview()
      .then((ov) => {
        setOverview(ov);
        setLoading(false);
        if (ov.milestones.length > 0) {
          setSelectedId(ov.milestones[0].id);
        }
      })
      .catch((err) => {
        console.error(err);
        setLoading(false);
      });
  }, [dispatch]);

  if (loading) {
    return (
      <div style={{ padding: 40, textAlign: "center", color: "var(--text-tertiary)" }}>
        Loading roadmap...
      </div>
    );
  }

  const selected = overview?.milestones.find((m) => m.id === selectedId) ?? null;

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
    discarded: { color: "#6c6671", bg: "rgba(108,102,113,0.12)" },
  };

  return (
    <div style={{ overflowY: "auto", height: "100%" }}>
      <div style={{ maxWidth: 1100, margin: "0 auto", padding: "24px 36px 70px" }}>
        <h1
          style={{
            fontSize: 24,
            fontWeight: 800,
            letterSpacing: "-.025em",
            margin: "0 0 5px",
          }}
        >
          {overview?.title || "Roadmap"}
        </h1>
        <p
          style={{
            fontSize: 13.5,
            color: "var(--text-tertiary)",
            margin: "0 0 22px",
          }}
        >
          Milestone intelligence — derived from{" "}
          <span style={{ fontFamily: "var(--font-mono)", fontSize: 12, color: "#9a949f" }}>
            .lmbrain/ROADMAP.md
          </span>
          , specs, reviews, and decisions.
        </p>

        {(!overview || overview.milestones.length === 0) && (
          <div
            style={{
              textAlign: "center",
              padding: 40,
              color: "var(--text-tertiary)",
              background: "var(--bg-tertiary)",
              border: "1px solid var(--border-secondary)",
              borderRadius: 13,
              marginBottom: 20,
            }}
          >
            No milestones defined in ROADMAP.md.
          </div>
        )}

        {overview && overview.milestones.length > 0 && (
          <div style={{ display: "flex", gap: 20, alignItems: "flex-start" }}>
            {/* Milestone list / sidebar */}
            <div style={{ flex: "0 0 280px", display: "flex", flexDirection: "column", gap: 8 }}>
              {overview.milestones.map((m) => {
                const sc = statusColors[m.status] || statusColors.planned;
                const isSelected = m.id === selectedId;
                return (
                  <div
                    key={m.id}
                    onClick={() => setSelectedId(m.id)}
                    style={{
                      display: "flex",
                      alignItems: "center",
                      gap: 10,
                      padding: "10px 12px",
                      borderRadius: 10,
                      background: isSelected ? "rgba(124,108,246,0.08)" : "transparent",
                      border: `1px solid ${isSelected ? "rgba(124,108,246,0.25)" : "transparent"}`,
                      cursor: "pointer",
                    }}
                  >
                    <div
                      style={{
                        width: 8,
                        height: 8,
                        borderRadius: "50%",
                        background: sc.color,
                        flex: "none",
                      }}
                    />
                    <div style={{ flex: 1, minWidth: 0 }}>
                      <div
                        style={{
                          fontFamily: "var(--font-mono)",
                          fontSize: 11.5,
                          color: isSelected ? "var(--accent-light)" : "#bcaef6",
                          fontWeight: isSelected ? 700 : 500,
                        }}
                      >
                        {m.id}
                      </div>
                      <div
                        style={{
                          fontSize: 12.5,
                          fontWeight: 600,
                          color: "var(--text-primary)",
                          whiteSpace: "nowrap",
                          overflow: "hidden",
                          textOverflow: "ellipsis",
                        }}
                      >
                        {m.title}
                      </div>
                    </div>
                    <div
                      style={{
                        fontSize: 10,
                        fontWeight: 700,
                        color: sc.color,
                        background: sc.bg,
                        borderRadius: 4,
                        padding: "2px 6px",
                        flex: "none",
                      }}
                    >
                      {m.status.toUpperCase()}
                    </div>
                  </div>
                );
              })}
            </div>

            {/* Selected milestone detail */}
            <div style={{ flex: 1, minWidth: 0 }}>
              {selected && <MilestoneDetailCard milestone={selected} navigateToWiki={navigateToWiki} openDetailArtifact={openDetailArtifact} specStatusColors={specStatusColors} />}
            </div>
          </div>
        )}

        {/* Unmapped specs */}
        {overview && overview.unmapped_specs.length > 0 && (
          <div
            style={{
              border: "1px dashed var(--border-secondary)",
              borderRadius: 13,
              padding: "20px 22px",
              marginTop: 20,
            }}
          >
            <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 10 }}>
              <i className="material-symbols-outlined" style={{ fontSize: 18, color: "#8a8d99" }}>
                link_off
              </i>
              <span style={{ fontWeight: 700, fontSize: 15, color: "var(--text-secondary)" }}>
                Unmapped Specs ({overview.unmapped_specs.length})
              </span>
            </div>
            <div style={{ display: "flex", gap: 6, flexWrap: "wrap" }}>
              {overview.unmapped_specs.map((spec) => (
                <span
                  key={spec.id}
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: 11,
                    color: "#8a8d99",
                    background: "rgba(138,141,153,0.1)",
                    border: "1px solid rgba(138,141,153,0.2)",
                    borderRadius: 5,
                    padding: "3px 8px",
                  }}
                  title={spec.title}
                >
                  {spec.id}
                </span>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

function MilestoneDetailCard({
  milestone,
  navigateToWiki,
  openDetailArtifact,
  specStatusColors,
}: {
  milestone: MilestoneDetail;
  navigateToWiki: (target: string) => void;
  openDetailArtifact: (artifact: { title: string; path: string } | null) => void;
  specStatusColors: Record<string, { color: string; bg: string }>;
}) {
  const sc = (statusColors as Record<string, { color: string; bg: string }>)[milestone.status] || { color: "var(--text-tertiary)", bg: "var(--bg-secondary)" };

  return (
    <div
      style={{
        background: "var(--bg-tertiary)",
        border: "1px solid var(--border-secondary)",
        borderRadius: 13,
        padding: "22px 24px",
        display: "flex",
        flexDirection: "column",
        gap: 16,
      }}
    >
      {/* Header */}
      <div style={{ display: "flex", alignItems: "flex-start", justifyContent: "space-between" }}>
        <div>
          <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 4 }}>
            <span style={{ fontFamily: "var(--font-mono)", fontWeight: 700, fontSize: 15, color: "var(--accent-light)" }}>
              {milestone.id}
            </span>
            <span style={{ fontWeight: 700, fontSize: 16, color: "var(--text-primary)" }}>
              <InlineRichText text={milestone.title} onWikilinkClick={navigateToWiki} />
            </span>
          </div>
          <span style={{ fontSize: 11, fontWeight: 700, color: sc.color, background: sc.bg, borderRadius: 5, padding: "3px 8px", letterSpacing: "0.03em" }}>
            {milestone.status.toUpperCase()}
          </span>
        </div>
      </div>

      {/* Outcome */}
      {milestone.outcome && (
        <div style={{ fontSize: 13.5, color: "var(--text-secondary)", lineHeight: 1.45, borderLeft: "2px solid #3c3547", paddingLeft: 10 }}>
          <InlineRichText text={milestone.outcome} onWikilinkClick={navigateToWiki} />
        </div>
      )}

      {/* Progress */}
      {milestone.spec_count > 0 && (
        <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
          <div style={{ display: "flex", justifyContent: "space-between", fontSize: 11.5, color: "var(--text-tertiary)" }}>
            <span>Progress</span>
            <span>{milestone.specs.filter((s) => s.status === "done").length}/{milestone.spec_count} specs ({Math.round(milestone.progress_pct)}%)</span>
          </div>
          <div style={{ height: 6, background: "#211d27", borderRadius: 3, overflow: "hidden" }}>
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
              <span key={status} style={{ fontSize: 11, fontWeight: 700, color: sc2.color, background: sc2.bg, borderRadius: 5, padding: "3px 8px" }}>
                {status}: {count}
              </span>
            );
          })}
        </div>
      )}

      {/* Next action */}
      {milestone.next_action && (
        <div style={{ display: "flex", alignItems: "center", gap: 8, fontSize: 12.5, color: "var(--accent-light)", background: "rgba(124,108,246,0.08)", borderRadius: 8, padding: "8px 12px" }}>
          <i className="material-symbols-outlined" style={{ fontSize: 16 }}>arrow_forward</i>
          <span>{milestone.next_action}</span>
        </div>
      )}

      {/* Specs list */}
      {milestone.specs.length > 0 && (
        <div>
          <div style={{ fontSize: 11.5, fontWeight: 600, color: "var(--text-tertiary)", marginBottom: 8 }}>
            Specifications ({milestone.specs.length})
          </div>
          <div style={{ display: "flex", flexDirection: "column", gap: 5 }}>
            {milestone.specs.map((spec) => {
              const sc2 = specStatusColors[spec.status] || { color: "var(--text-tertiary)", bg: "var(--bg-secondary)" };
              return (
                <div
                  key={spec.id}
                  onClick={() => {
                    const specPath = spec.path || `.lmbrain/specs/${spec.status}/${spec.id}.md`;
                    openDetailArtifact({ title: spec.title, path: specPath });
                  }}
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: 8,
                    padding: "7px 10px",
                    borderRadius: 8,
                    background: "rgba(255,255,255,0.02)",
                    border: "1px solid rgba(255,255,255,0.05)",
                    cursor: "pointer",
                  }}
                >
                  <span style={{ fontFamily: "var(--font-mono)", fontSize: 11, color: "#bcaef6", flex: "none" }}>
                    {spec.id}
                  </span>
                  <span style={{ fontSize: 12.5, color: "var(--text-primary)", flex: 1, minWidth: 0, whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" }}>
                    {spec.title}
                  </span>
                  <span style={{ fontSize: 10, fontWeight: 700, color: sc2.color, background: sc2.bg, borderRadius: 4, padding: "1px 6px", flex: "none" }}>
                    {spec.status}
                  </span>
                  {spec.priority && (
                    <span style={{ fontSize: 10, color: "var(--text-muted)", flex: "none" }}>
                      {spec.priority}
                    </span>
                  )}
                  {spec.recommended_agent && (
                    <span style={{ fontSize: 10, color: "var(--text-muted)", flex: "none", fontFamily: "var(--font-mono)" }}>
                      {spec.recommended_agent}
                    </span>
                  )}
                </div>
              );
            })}
          </div>
        </div>
      )}

      {/* Reviews */}
      {milestone.reviews.length > 0 && (
        <div>
          <div style={{ fontSize: 11.5, fontWeight: 600, color: "var(--text-tertiary)", marginBottom: 8 }}>
            Reviews ({milestone.reviews.length})
          </div>
          <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
            {milestone.reviews.map((r) => (
              <div
                key={r.id}
                onClick={() => r.path && openDetailArtifact({ title: r.title, path: r.path })}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 8,
                  fontSize: 12,
                  color: "var(--text-secondary)",
                  cursor: r.path ? "pointer" : "default",
                  padding: "3px 6px",
                  borderRadius: 6,
                }}
                onMouseEnter={(e) => { if (r.path) { e.currentTarget.style.background = "rgba(255,255,255,0.03)"; } }}
                onMouseLeave={(e) => { e.currentTarget.style.background = "transparent"; }}
              >
                <span style={{ fontFamily: "var(--font-mono)", fontSize: 11, color: "#bcaef6" }}>{r.id}</span>
                <span>{r.title}</span>
                <span style={{ fontSize: 10, fontWeight: 700, color: r.status === "accepted" ? "#46b07d" : "#e0a23a", background: r.status === "accepted" ? "rgba(70,176,125,0.1)" : "rgba(224,162,58,0.1)", borderRadius: 4, padding: "1px 6px" }}>
                  {r.status}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Decisions */}
      {milestone.decisions.length > 0 && (
        <div>
          <div style={{ fontSize: 11.5, fontWeight: 600, color: "var(--text-tertiary)", marginBottom: 8 }}>
            Decisions ({milestone.decisions.length})
          </div>
          <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
            {milestone.decisions.map((d) => (
              <div
                key={d.id}
                onClick={() => d.path && openDetailArtifact({ title: d.title, path: d.path })}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 8,
                  fontSize: 12,
                  color: "var(--text-secondary)",
                  cursor: d.path ? "pointer" : "default",
                  padding: "3px 6px",
                  borderRadius: 6,
                }}
                onMouseEnter={(e) => { if (d.path) { e.currentTarget.style.background = "rgba(255,255,255,0.03)"; } }}
                onMouseLeave={(e) => { e.currentTarget.style.background = "transparent"; }}
              >
                <span style={{ fontFamily: "var(--font-mono)", fontSize: 11, color: "#bcaef6" }}>{d.id}</span>
                <span>{d.title}</span>
                <span style={{ fontSize: 10, fontWeight: 700, color: d.status === "accepted" ? "#46b07d" : "#8a8d99", background: d.status === "accepted" ? "rgba(70,176,125,0.1)" : "rgba(138,141,153,0.1)", borderRadius: 4, padding: "1px 6px" }}>
                  {d.status}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Risks */}
      {milestone.risks.length > 0 && (
        <div>
          <div style={{ fontSize: 11.5, fontWeight: 600, color: "var(--text-tertiary)", marginBottom: 6 }}>
            Risks
          </div>
          <ul style={{ margin: 0, paddingLeft: 18, fontSize: 12.5, color: "#f0a89f", display: "flex", flexDirection: "column", gap: 3 }}>
            {milestone.risks.map((risk, i) => (
              <li key={i}>{risk}</li>
            ))}
          </ul>
        </div>
      )}

      {/* Dependencies */}
      {milestone.depends_on && (
        <div style={{ fontSize: 12.5, color: "var(--text-tertiary)" }}>
          <span style={{ fontWeight: 600 }}>Depends on:</span> {milestone.depends_on}
        </div>
      )}

      {/* Unresolved references */}
      {milestone.unresolved_refs.length > 0 && (
        <div>
          <div style={{ fontSize: 11.5, fontWeight: 600, color: "#e0a23a", marginBottom: 6 }}>
            Unresolved References
          </div>
          <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
            {milestone.unresolved_refs.map((ref, i) => (
              <div key={i} style={{ fontSize: 12, color: "#f0a89f", display: "flex", alignItems: "center", gap: 6 }}>
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

const statusColors: Record<string, { color: string; bg: string }> = {
  active: { color: "#5b8def", bg: "rgba(91,141,239,0.13)" },
  planned: { color: "#8a8d99", bg: "rgba(138,141,153,0.13)" },
  completed: { color: "#46b07d", bg: "rgba(70,176,125,0.13)" },
};
