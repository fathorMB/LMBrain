import { useEffect, useMemo, useState } from "react";
import { useWorkspace } from "../../hooks/useWorkspace";
import { getProjectStatistics } from "../../lib/commands";
import { buildDiagnosticFixPrompt } from "../../lib/diagnosticPrompt";
import type {
  ArtifactFamilyStats,
  KitDiagnostic,
  ProjectStatistics,
  ReviewDimensionStat,
  StatusCount,
} from "../../types";

const STATUS_COLORS: Record<string, string> = {
  done: "#46b07d",
  active: "#46b07d",
  accepted: "#46b07d",
  ready: "#7c6cf6",
  working: "#5b8def",
  review: "#e0a23a",
  pending: "#e0a23a",
  proposed: "#e0a23a",
  "changes-requested": "#e0584a",
  blocked: "#e0584a",
  error: "#e0584a",
  warning: "#e0a23a",
  retired: "#6c6671",
  discarded: "#6c6671",
  superseded: "#6c6671",
  deprecated: "#6c6671",
};

export function InsightsView() {
  const { state: workspaceState } = useWorkspace();
  const [stats, setStats] = useState<ProjectStatistics | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    getProjectStatistics()
      .then((data) => {
        setStats(data);
        setError(null);
      })
      .catch((err) => {
        console.error(err);
        setError(typeof err === "string" ? err : "Failed to load project insights.");
      })
      .finally(() => setLoading(false));
  }, []);

  const totalArtifacts = useMemo(
    () => stats?.artifact_families.reduce((sum, family) => sum + family.total, 0) ?? 0,
    [stats]
  );

  if (loading) {
    return (
      <div style={{ padding: 40, textAlign: "center", color: "var(--text-tertiary)" }}>
        Loading insights...
      </div>
    );
  }

  if (error || !stats) {
    return (
      <div style={{ padding: 40, textAlign: "center", color: "#e0584a" }}>
        {error || "No project insights available."}
      </div>
    );
  }

  const review = stats.review_quality;

  return (
    <div style={{ overflowY: "auto", height: "100%" }}>
      <div style={{ maxWidth: 1180, margin: "0 auto", padding: "24px 36px 70px" }}>
        <h1 style={{ fontSize: 24, fontWeight: 800, letterSpacing: "-.025em", margin: "0 0 5px" }}>
          Insights
        </h1>
        <p style={{ fontSize: 13.5, color: "var(--text-tertiary)", margin: "0 0 22px" }}>
          Derived project statistics from LMBrain artifacts.
        </p>

        <div style={{ display: "grid", gridTemplateColumns: "repeat(5, minmax(0, 1fr))", gap: 10, marginBottom: 22 }}>
          <Kpi label="Artifacts" value={totalArtifacts.toString()} detail={`${stats.spec_flow.total_specs} specs`} accent="#7c6cf6" />
          <Kpi label="Done ratio" value={formatPercent(stats.spec_flow.done_ratio)} detail={`${stats.spec_flow.done_specs}/${stats.spec_flow.total_specs} specs`} accent="#46b07d" />
          <Kpi label="Change-request rate" value={formatPercent(review.change_request_rate)} detail={`${review.specs_with_changes_requested}/${review.reviewed_specs} reviewed specs`} accent="#e0584a" />
          <Kpi label="First-pass accepted" value={formatPercent(review.first_pass_acceptance_rate)} detail={`${review.first_pass_accepted_specs}/${review.first_pass_eligible_specs} date-ordered specs`} accent="#5b8def" />
          <Kpi label="Diagnostics" value={stats.diagnostics.total.toString()} detail={`${stats.diagnostics.errors} errors, ${stats.diagnostics.warnings} warnings`} accent={stats.diagnostics.errors > 0 ? "#e0584a" : "#e0a23a"} />
        </div>

        <div style={{ display: "grid", gridTemplateColumns: "minmax(0, 1.35fr) minmax(320px, .65fr)", gap: 16, marginBottom: 18 }}>
          <section style={panelStyle}>
            <SectionTitle icon="rate_review" title="Review Quality" />
            <div style={{ display: "grid", gridTemplateColumns: "repeat(4, minmax(0, 1fr))", gap: 9, marginBottom: 16 }}>
              <MiniStat label="Reviews" value={review.total_reviews} />
              <MiniStat label="Reviewed specs" value={review.reviewed_specs} />
              <MiniStat label="Changes requested" value={review.changes_requested_reviews} />
              <MiniStat label="Multiple CR specs" value={review.specs_with_multiple_changes_requested} />
            </div>
            <MetricBar
              label="Specs receiving changes requested"
              value={review.specs_with_changes_requested}
              total={review.reviewed_specs}
              color="#e0584a"
            />
            <MetricBar
              label="First-pass accepted specs"
              value={review.first_pass_accepted_specs}
              total={review.first_pass_eligible_specs}
              color="#46b07d"
            />
            <div style={{ marginTop: 14, fontSize: 12, color: "var(--text-tertiary)", lineHeight: 1.5 }}>
              Average review iterations:{" "}
              <span style={{ color: "var(--text-primary)", fontFamily: "var(--font-mono)" }}>
                {review.average_reviews_per_reviewed_spec.toFixed(2)}
              </span>
              {review.reviews_without_spec > 0 && ` · ${review.reviews_without_spec} review(s) without spec reference`}
              {review.reviews_without_created > 0 && ` · ${review.reviews_without_created} review(s) without valid created date`}
            </div>
          </section>

          <section style={panelStyle}>
            <SectionTitle icon="schema" title="Artifact Inventory" />
            <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
              {stats.artifact_families.map((family) => (
                <FamilyRow key={family.family} family={family} />
              ))}
            </div>
          </section>
        </div>

        <div style={{ display: "grid", gridTemplateColumns: "repeat(2, minmax(0, 1fr))", gap: 16, marginBottom: 18 }}>
          <section style={panelStyle}>
            <SectionTitle icon="hub" title="Changes Requested By Area" />
            <DimensionTable rows={review.by_area} emptyLabel="No reviewed specs with area metadata." />
          </section>
          <section style={panelStyle}>
            <SectionTitle icon="smart_toy" title="Changes Requested By Agent" />
            <DimensionTable rows={review.by_agent} emptyLabel="No reviewed specs with recommended agents." />
          </section>
        </div>

        <section style={panelStyle}>
          <SectionTitle icon="verified" title="Insight Reliability" />
          <InsightReliability
            reviewsWithoutSpec={review.reviews_without_spec}
            reviewsWithoutCreated={review.reviews_without_created}
            errors={stats.diagnostics.errors}
            warnings={stats.diagnostics.warnings}
            diagnostics={workspaceState.diagnostics}
          />
        </section>
      </div>
    </div>
  );
}

const panelStyle = {
  background: "var(--bg-tertiary)",
  border: "1px solid var(--border-secondary)",
  borderRadius: 8,
  padding: 15,
} as const;

function Kpi({ label, value, detail, accent }: { label: string; value: string; detail: string; accent: string }) {
  return (
    <div style={{ ...panelStyle, borderTop: `2px solid ${accent}`, minHeight: 96 }}>
      <div style={{ fontSize: 11, color: "var(--text-tertiary)", textTransform: "uppercase", letterSpacing: ".08em", fontWeight: 700 }}>
        {label}
      </div>
      <div style={{ fontSize: 27, fontWeight: 800, fontFamily: "var(--font-mono)", marginTop: 8 }}>
        {value}
      </div>
      <div style={{ fontSize: 11.5, color: "var(--text-tertiary)", marginTop: 3 }}>
        {detail}
      </div>
    </div>
  );
}

function SectionTitle({ icon, title }: { icon: string; title: string }) {
  return (
    <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 13 }}>
      <i className="material-symbols-outlined" style={{ fontSize: 17, color: "var(--accent-light)" }}>
        {icon}
      </i>
      <div style={{ fontSize: 12, fontWeight: 800, color: "var(--text-secondary)", textTransform: "uppercase", letterSpacing: ".08em" }}>
        {title}
      </div>
    </div>
  );
}

function MiniStat({ label, value }: { label: string; value: number }) {
  return (
    <div style={{ background: "rgba(255,255,255,.035)", border: "1px solid rgba(255,255,255,.06)", borderRadius: 7, padding: "10px 11px" }}>
      <div style={{ fontFamily: "var(--font-mono)", fontSize: 20, fontWeight: 800 }}>{value}</div>
      <div style={{ fontSize: 11, color: "var(--text-tertiary)", marginTop: 2 }}>{label}</div>
    </div>
  );
}

function MetricBar({ label, value, total, color }: { label: string; value: number; total: number; color: string }) {
  const pct = total === 0 ? 0 : value / total;
  return (
    <div style={{ marginBottom: 12 }}>
      <div style={{ display: "flex", justifyContent: "space-between", fontSize: 12, color: "var(--text-secondary)", marginBottom: 6 }}>
        <span>{label}</span>
        <span style={{ fontFamily: "var(--font-mono)", color: "var(--text-primary)" }}>
          {value}/{total} · {formatPercent(pct)}
        </span>
      </div>
      <div style={{ height: 8, background: "#211d27", borderRadius: 5, overflow: "hidden" }}>
        <div style={{ width: `${Math.round(pct * 100)}%`, height: "100%", background: color, borderRadius: 5 }} />
      </div>
    </div>
  );
}

function FamilyRow({ family }: { family: ArtifactFamilyStats }) {
  return (
    <div style={{ borderBottom: "1px solid rgba(255,255,255,.06)", paddingBottom: 8 }}>
      <div style={{ display: "flex", justifyContent: "space-between", gap: 12, marginBottom: 5 }}>
        <span style={{ fontSize: 12.5, fontWeight: 700, color: "var(--text-primary)" }}>{family.label}</span>
        <span style={{ fontFamily: "var(--font-mono)", fontSize: 12, color: "#bcaef6" }}>{family.total}</span>
      </div>
      <StatusList statuses={family.statuses} compact />
    </div>
  );
}

function StatusList({ statuses, compact }: { statuses: StatusCount[]; compact?: boolean }) {
  return (
    <div style={{ display: "flex", flexWrap: "wrap", gap: 5 }}>
      {statuses.map((item) => (
        <span key={item.status} style={{ fontSize: compact ? 10.5 : 11.5, color: STATUS_COLORS[item.status] ?? "#9a949f", background: "rgba(255,255,255,.045)", borderRadius: 5, padding: "2px 6px" }}>
          {item.status}: {item.count}
        </span>
      ))}
    </div>
  );
}

function DimensionTable({ rows, emptyLabel }: { rows: ReviewDimensionStat[]; emptyLabel: string }) {
  if (rows.length === 0) return <EmptyText label={emptyLabel} />;
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
      {rows.slice(0, 8).map((row) => (
        <div key={row.value}>
          <div style={{ display: "flex", justifyContent: "space-between", gap: 10, fontSize: 12.5, marginBottom: 5 }}>
            <span style={{ color: "var(--text-primary)", fontWeight: 650, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{row.value}</span>
            <span style={{ fontFamily: "var(--font-mono)", color: "var(--text-tertiary)" }}>
              {row.specs_with_changes_requested}/{row.reviewed_specs} · {formatPercent(row.change_request_rate)}
            </span>
          </div>
          <div style={{ height: 6, background: "#211d27", borderRadius: 4, overflow: "hidden" }}>
            <div style={{ width: `${Math.round(row.change_request_rate * 100)}%`, height: "100%", background: "#e0584a" }} />
          </div>
        </div>
      ))}
    </div>
  );
}

function InsightReliability({
  reviewsWithoutSpec,
  reviewsWithoutCreated,
  errors,
  warnings,
  diagnostics,
}: {
  reviewsWithoutSpec: number;
  reviewsWithoutCreated: number;
  errors: number;
  warnings: number;
  diagnostics: KitDiagnostic[];
}) {
  const issueCount = reviewsWithoutSpec + reviewsWithoutCreated + errors;
  const summary = issueCount > 0 ? "Needs attention" : warnings > 0 ? "Review recommended" : "Reliable inputs";
  const summaryColor = issueCount > 0 ? "#e0584a" : warnings > 0 ? "#e0a23a" : "#46b07d";
  const checks = [
    { label: "Reviews without spec link", value: reviewsWithoutSpec, detail: "Affects spec-based review rates", tone: reviewsWithoutSpec > 0 ? "error" : "ok" },
    { label: "Reviews without valid date", value: reviewsWithoutCreated, detail: "Excluded from review history", tone: reviewsWithoutCreated > 0 ? "error" : "ok" },
    { label: "Diagnostic errors", value: errors, detail: "Contract violations need attention", tone: errors > 0 ? "error" : "ok" },
    { label: "Diagnostic warnings", value: warnings, detail: "Potential data-quality issues", tone: warnings > 0 ? "warning" : "ok" },
  ];

  const severityOrder: Record<KitDiagnostic["severity"], number> = { error: 0, warning: 1, info: 2 };
  const orderedDiagnostics = [...diagnostics].sort((a, b) => severityOrder[a.severity] - severityOrder[b.severity]);

  return (
    <div>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "start", gap: 24, marginBottom: 15 }}>
        <div>
          <div role="status" style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 7, color: summaryColor, fontSize: 14, fontWeight: 750 }}>
            <span aria-hidden="true" style={{ width: 8, height: 8, borderRadius: "50%", background: summaryColor }} />
            {summary}
          </div>
          <p style={{ ...panelDescriptionStyle, marginBottom: 0, maxWidth: 680 }}>
            These checks show whether missing metadata or contract diagnostics may make the metrics incomplete.
          </p>
        </div>
        <div style={{ fontFamily: "var(--font-mono)", fontSize: 11, color: "var(--text-tertiary)", whiteSpace: "nowrap" }}>
          {diagnostics.length} workspace diagnostic{diagnostics.length === 1 ? "" : "s"}
        </div>
      </div>
      <div style={{ display: "grid", gridTemplateColumns: "repeat(4, minmax(0, 1fr))", gap: 8 }}>
        {checks.map((check) => {
          const color = check.tone === "error" ? "#e0584a" : check.tone === "warning" ? "#e0a23a" : "#46b07d";
          return (
            <div key={check.label} style={{ display: "grid", gridTemplateColumns: "minmax(0, 1fr) auto", gap: "3px 12px", padding: "11px 12px", border: "1px solid rgba(255,255,255,.07)", borderRadius: 7, background: "rgba(255,255,255,.025)" }}>
              <span style={{ fontSize: 11.5, color: "var(--text-secondary)" }}>{check.label}</span>
              <span style={{ gridRow: "1 / span 2", gridColumn: 2, alignSelf: "center", fontFamily: "var(--font-mono)", fontSize: 18, fontWeight: 800, color }}>{check.value}</span>
              <span style={{ fontSize: 10.5, color: "var(--text-tertiary)" }}>{check.detail}</span>
            </div>
          );
        })}
      </div>
      {orderedDiagnostics.length > 0 ? (
        <details style={{ marginTop: 12, border: "1px solid rgba(255,255,255,.08)", borderRadius: 7, background: "rgba(255,255,255,.02)", overflow: "hidden" }}>
          <summary style={{ cursor: "pointer", padding: "11px 13px", color: "var(--text-secondary)", fontSize: 12, fontWeight: 700, userSelect: "none" }}>
            Diagnostic details ({orderedDiagnostics.length})
          </summary>
          <div style={{ display: "flex", flexDirection: "column", gap: 7, padding: "0 12px 12px" }}>
            {orderedDiagnostics.map((diagnostic, index) => (
              <DiagnosticDetail key={`${diagnostic.path ?? "workspace"}-${index}`} diagnostic={diagnostic} />
            ))}
          </div>
        </details>
      ) : (
        <div style={{ marginTop: 12, padding: "10px 12px", borderRadius: 7, background: "rgba(70,176,125,.06)", color: "#70c99a", fontSize: 11.5 }}>
          No workspace diagnostics to inspect.
        </div>
      )}
      <div style={{ marginTop: 11, fontSize: 11, lineHeight: 1.45, color: "var(--text-tertiary)" }}>
        Review and resolve diagnostic issues in <span style={{ color: "var(--text-secondary)", fontWeight: 700 }}>Project Pulse</span>.
      </div>
    </div>
  );
}

function DiagnosticDetail({ diagnostic }: { diagnostic: KitDiagnostic }) {
  const [copyState, setCopyState] = useState<"idle" | "copied" | "error">("idle");
  const presentation = diagnostic.severity === "error"
    ? { color: "#e0584a", background: "rgba(224,88,74,.07)", border: "rgba(224,88,74,.18)", icon: "error" }
    : diagnostic.severity === "warning"
      ? { color: "#e0a23a", background: "rgba(224,162,58,.07)", border: "rgba(224,162,58,.18)", icon: "warning" }
      : { color: "#5b8def", background: "rgba(91,141,239,.07)", border: "rgba(91,141,239,.18)", icon: "info" };

  return (
    <div style={{ display: "grid", gridTemplateColumns: "auto minmax(0, 1fr)", gap: "3px 9px", padding: "10px 11px", border: `1px solid ${presentation.border}`, borderRadius: 7, background: presentation.background }}>
      <i className="material-symbols-outlined" aria-hidden="true" style={{ gridRow: "1 / span 2", fontSize: 16, color: presentation.color, marginTop: 1 }}>
        {presentation.icon}
      </i>
      <div style={{ display: "flex", alignItems: "start", justifyContent: "space-between", gap: 12 }}>
        <div style={{ minWidth: 0 }}>
        <span style={{ marginRight: 8, color: presentation.color, fontSize: 10, fontWeight: 800, letterSpacing: ".06em", textTransform: "uppercase" }}>
          {diagnostic.severity}
        </span>
        <span style={{ color: "var(--text-secondary)", fontSize: 12, lineHeight: 1.5 }}>{diagnostic.message}</span>
        </div>
        <button
          type="button"
          onClick={async () => {
            try {
              await navigator.clipboard.writeText(buildDiagnosticFixPrompt(diagnostic));
              setCopyState("copied");
              setTimeout(() => setCopyState("idle"), 2000);
            } catch {
              setCopyState("error");
            }
          }}
          style={{
            flex: "none",
            display: "inline-flex",
            alignItems: "center",
            gap: 5,
            padding: "4px 8px",
            border: "1px solid rgba(255,255,255,.11)",
            borderRadius: 6,
            background: "rgba(255,255,255,.055)",
            color: copyState === "error" ? "#e0584a" : "var(--text-secondary)",
            fontSize: 10.5,
            fontWeight: 650,
            cursor: "pointer",
          }}
        >
          <i className="material-symbols-outlined" aria-hidden="true" style={{ fontSize: 13 }}>
            {copyState === "copied" ? "check" : copyState === "error" ? "error" : "content_copy"}
          </i>
          {copyState === "copied" ? "Copied!" : copyState === "error" ? "Copy failed" : "Copy fix prompt"}
        </button>
      </div>
      {diagnostic.path && (
        <div style={{ fontFamily: "var(--font-mono)", fontSize: 10.5, color: "var(--text-tertiary)", overflowWrap: "anywhere" }}>
          {diagnostic.path}
        </div>
      )}
    </div>
  );
}

const panelDescriptionStyle = {
  margin: "0 0 11px",
  fontSize: 11.5,
  lineHeight: 1.5,
  color: "var(--text-tertiary)",
} as const;

function EmptyText({ label }: { label: string }) {
  return <div style={{ fontSize: 12.5, color: "var(--text-tertiary)" }}>{label}</div>;
}

function formatPercent(value: number) {
  return `${Math.round(value * 100)}%`;
}
