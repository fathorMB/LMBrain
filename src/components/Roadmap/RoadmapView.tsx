import { useEffect, useState } from "react";
import { useWorkspace } from "../../hooks/useWorkspace";
import { getMilestoneOverview } from "../../lib/commands";
import { PageHeader, PageShell } from "../Shared/PageLayout";
import type { MilestoneOverview } from "../../types";
import { MilestoneSidebar } from "./MilestoneSidebar";
import { MilestoneDetailView } from "./MilestoneDetailView";

export function RoadmapView() {
  const { state, navigateTo } = useWorkspace();
  const [overview, setOverview] = useState<MilestoneOverview | null>(null);
  const [loading, setLoading] = useState(true);
  const [selectedId, setSelectedId] = useState<string | null>(null);

  useEffect(() => {
    getMilestoneOverview()
      .then((ov) => {
        setOverview(ov);
        setLoading(false);
        const first = ov.milestones[0];
        if (first) {
          setSelectedId(first.id);
        }
      })
      .catch((err) => {
        console.error(err);
        setLoading(false);
      });
  }, []);

  if (loading) {
    return (
      <div style={{ padding: 40, textAlign: "center", color: "var(--text-tertiary)" }}>
        Loading roadmap...
      </div>
    );
  }

  const selected = overview?.milestones.find((m) => m.id === selectedId) ?? null;
  const milestoneDebts = (state.debts ?? []).filter((debt) =>
    ["open", "planned", "deferred"].includes(debt.status)
    && (debt.milestone === selectedId
      || debt.target_specs.some((target) => selected?.specs.some((spec) => spec.id === target)))
  );

  return (
    <PageShell archetype="dense">
      <PageHeader
        title={overview?.title || "Roadmap"}
        description={
          <>
            Milestone intelligence — derived from{" "}
            <span style={{ fontFamily: "var(--font-mono)", fontSize: "var(--text-sm)", color: "var(--text-secondary)" }}>
              .lmbrain/ROADMAP.md
            </span>
            , specs, reviews, and decisions.
          </>
        }
      />
      {selected && milestoneDebts.length > 0 && (
        <button
          type="button"
          onClick={() => navigateTo("debts")}
          style={{
            marginBottom: 16,
            padding: "9px 12px",
            border: "1px solid rgba(224,162,58,.35)",
            borderRadius: 8,
            background: "rgba(224,162,58,.08)",
            color: "#d9b86d",
            cursor: "pointer",
          }}
        >
          {milestoneDebts.length} active {milestoneDebts.length === 1 ? "debt" : "debts"} attached to {selected.id}
        </button>
      )}

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
          <MilestoneSidebar
            milestones={overview.milestones}
            selectedId={selectedId}
            onSelectMilestone={setSelectedId}
          />

          {/* Milestone detail */}
          {selected && <MilestoneDetailView milestone={selected} />}
        </div>
      )}
    </PageShell>
  );
}
