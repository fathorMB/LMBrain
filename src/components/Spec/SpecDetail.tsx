import { useWorkspace } from "../../hooks/useWorkspace";
import { MarkdownRenderer } from "../../lib/markdown";
import { OperatorVerificationPanel } from "./OperatorVerificationPanel";
import { PageShell } from "../Shared/PageLayout";
import { SpecMetaPill } from "./SpecMetaPill";
import { SpecLifecycleRail } from "./SpecLifecycleRail";
import { SpecHandoffCTA } from "./SpecHandoffCTA";

export function SpecDetail() {
  const { state, closeSpecDetail, loadAllData, navigateTo } = useWorkspace();
  const specs = state.specs;
  const readySpecs = specs.filter((s) => s.status === "ready");

  const spec = state.selectedSpec || readySpecs[0] || specs[0];
  const relatedDebts = spec
    ? (state.debts ?? []).filter(
        (debt) =>
          ["open", "planned", "deferred"].includes(debt.status) &&
          (debt.origin_artifact === spec.id ||
            debt.related_specs.includes(spec.id) ||
            debt.target_specs.includes(spec.id))
      )
    : [];
  const directDependencies = spec
    ? (spec.depends_on ?? []).map((id) => ({
        id,
        spec: specs.find((candidate) => candidate.id === id),
      }))
    : [];
  const dependencyBlockers = directDependencies.filter(
    (dependency) => dependency.spec?.status !== "done"
  );
  const latestParking = spec?.parking_events?.at(-1);

  if (!spec) {
    return (
      <div
        style={{
          height: "100%",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          color: "var(--text-tertiary)",
        }}
      >
        No specifications found.
      </div>
    );
  }

  return (
    <PageShell archetype="reading">
      {/* Breadcrumb */}
      <button
        type="button"
        aria-label="Back to specification board"
        onClick={closeSpecDetail}
        style={{
          display: "flex",
          alignItems: "center",
          gap: 6,
          fontFamily: "var(--font-mono)",
          fontSize: "var(--text-xs)",
          color: "var(--text-tertiary)",
          marginBottom: 18,
          cursor: "pointer",
          width: "max-content",
          padding: 0,
          border: 0,
          background: "transparent",
          textAlign: "left",
        }}
      >
        <i className="material-symbols-outlined" style={{ fontSize: 15 }}>
          arrow_back
        </i>
        specs / {spec.status} / {spec.id}.md
      </button>

      {/* Header */}
      <div
        style={{
          display: "flex",
          alignItems: "flex-start",
          justifyContent: "space-between",
          gap: 20,
          marginBottom: 18,
        }}
      >
        <div>
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: 10,
              marginBottom: 8,
            }}
          >
            <span
              style={{
                fontFamily: "var(--font-mono)",
                fontSize: "var(--text-md)",
                color: "#bcaef6",
                fontWeight: 500,
              }}
            >
              {spec.id}
            </span>
            {spec.status === "ready" && (
              <span
                style={{
                  display: "inline-flex",
                  alignItems: "center",
                  gap: 5,
                  fontSize: "var(--text-xs)",
                  fontWeight: 700,
                  color: "var(--accent)",
                  background: "rgba(124,108,246,.13)",
                  border: "1px solid rgba(124,108,246,.3)",
                  borderRadius: 6,
                  padding: "3px 9px",
                }}
              >
                <span
                  style={{
                    width: 6,
                    height: 6,
                    borderRadius: "50%",
                    background: "var(--accent)",
                  }}
                />
                READY FOR HANDOFF
              </span>
            )}
          </div>
          <h1
            style={{
              fontSize: 27,
              fontWeight: 800,
              letterSpacing: "-.028em",
              margin: 0,
            }}
          >
            {spec.title}
          </h1>
        </div>
      </div>

      {/* Meta row */}
      <div
        style={{
          display: "flex",
          gap: 9,
          marginBottom: 22,
          flexWrap: "wrap",
        }}
      >
        {spec.recommended_agent && (
          <SpecMetaPill
            icon="smart_toy"
            label="Recommended agent"
            value={spec.recommended_agent}
          />
        )}
        {spec.priority && (
          <SpecMetaPill
            icon="priority_high"
            label="Priority"
            value={spec.priority}
          />
        )}
        {spec.milestone && (
          <SpecMetaPill
            icon="target"
            label="Milestone"
            value={spec.milestone}
          />
        )}
      </div>

      {relatedDebts.length > 0 && (
        <button
          type="button"
          onClick={() => navigateTo("debts")}
          style={{
            width: "100%",
            marginBottom: 18,
            padding: "10px 12px",
            textAlign: "left",
            border: "1px solid rgba(224,162,58,.35)",
            borderRadius: 8,
            background: "rgba(224,162,58,.08)",
            color: "#d9b86d",
            cursor: "pointer",
          }}
        >
          {relatedDebts.length} active {relatedDebts.length === 1 ? "debt" : "debts"} related to this spec · Open Debts
        </button>
      )}

      {directDependencies.length > 0 && (
        <section
          aria-label="Hard spec dependencies"
          style={{
            marginBottom: 18,
            padding: "12px 14px",
            border: `1px solid ${dependencyBlockers.length ? "rgba(224,162,58,.4)" : "var(--border-primary)"}`,
            borderRadius: 8,
            background: dependencyBlockers.length ? "rgba(224,162,58,.06)" : "var(--bg-tertiary)",
          }}
        >
          <div style={{ fontSize: "var(--text-sm)", fontWeight: 700, marginBottom: 8 }}>
            Hard prerequisites
            {dependencyBlockers.length > 0 && (
              <span style={{ marginLeft: 8, color: "#d9b86d" }}>
                {dependencyBlockers.length} incomplete
              </span>
            )}
          </div>
          {directDependencies.map((dependency) => (
            <div
              key={dependency.id}
              style={{ fontFamily: "var(--font-mono)", fontSize: "var(--text-xs)", marginTop: 5 }}
            >
              {dependency.id} · {dependency.spec?.status ?? "missing"}
            </div>
          ))}
          <div style={{ marginTop: 9, fontSize: "var(--text-xs)", color: "var(--text-tertiary)" }}>
            Dependency lifecycle changes are intentionally unavailable in the app.
          </div>
        </section>
      )}

      {spec.status === "backlog" && latestParking && (
        <section
          aria-label="Parking history"
          style={{
            marginBottom: 18,
            padding: "12px 14px",
            border: "1px solid var(--border-primary)",
            borderRadius: 8,
            background: "var(--bg-tertiary)",
          }}
        >
          <div style={{ fontSize: "var(--text-sm)", fontWeight: 700 }}>Previously parked in backlog</div>
          <div style={{ marginTop: 7, fontSize: "var(--text-sm)" }}>{latestParking.reason}</div>
          <div style={{ marginTop: 5, fontSize: "var(--text-xs)", color: "var(--text-tertiary)" }}>
            {latestParking.actor} · {latestParking.timestamp}
            {latestParking.revisit_condition ? ` · Revisit: ${latestParking.revisit_condition}` : ""}
          </div>
          <div style={{ marginTop: 8, fontSize: "var(--text-xs)", color: "var(--text-tertiary)" }}>
            Re-approval and lifecycle actions are intentionally unavailable in the app.
          </div>
        </section>
      )}

      {/* Lifecycle rail */}
      <SpecLifecycleRail status={spec.status} />

      {/* Handoff CTA for ready specs */}
      {spec.status === "ready" && <SpecHandoffCTA spec={spec} />}

      {(spec.status === "review" || spec.status === "done") && (
        <OperatorVerificationPanel spec={spec} onAttested={loadAllData} />
      )}

      {/* Body */}
      <MarkdownRenderer content={spec.body} />
    </PageShell>
  );
}
