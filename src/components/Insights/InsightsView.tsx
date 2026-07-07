import { useEffect, useMemo, useState } from "react";
import { getProjectStatistics } from "../../lib/commands";
import type {
  ArtifactFamilyStats,
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

        <div style={{ display: "grid", gridTemplateColumns: "minmax(0, 1fr) minmax(300px, .45fr)", gap: 16 }}>
          <section style={panelStyle}>
            <SectionTitle icon="timeline" title="Review Trend" />
            {review.trend.length === 0 ? (
              <EmptyText label="No dated reviews available." />
            ) : (
              <div style={{ display: "flex", alignItems: "end", gap: 8, height: 170, paddingTop: 10 }}>
                {review.trend.map((point) => {
                  const max = Math.max(...review.trend.map((item) => item.total_reviews), 1);
                  const height = Math.max(8, Math.round((point.total_reviews / max) * 120));
                  return (
                    <div key={point.period} style={{ flex: 1, minWidth: 42, display: "flex", flexDirection: "column", alignItems: "center", gap: 6 }}>
                      <div title={`${point.total_reviews} reviews`} style={{ width: "100%", maxWidth: 42, height, background: "rgba(124,108,246,.38)", border: "1px solid rgba(124,108,246,.55)", borderRadius: 5, position: "relative", overflow: "hidden" }}>
                        {point.changes_requested_reviews > 0 && (
                          <div style={{ position: "absolute", bottom: 0, left: 0, right: 0, height: `${(point.changes_requested_reviews / point.total_reviews) * 100}%`, background: "rgba(224,88,74,.75)" }} />
                        )}
                      </div>
                      <div style={{ fontFamily: "var(--font-mono)", fontSize: 10.5, color: "var(--text-tertiary)" }}>
                        {point.period}
                      </div>
                    </div>
                  );
                })}
              </div>
            )}
          </section>

          <section style={panelStyle}>
            <SectionTitle icon="troubleshoot" title="Diagnostics By Area" />
            {stats.diagnostics.by_family.length === 0 ? (
              <EmptyText label="No diagnostics." />
            ) : (
              <StatusList statuses={stats.diagnostics.by_family} />
            )}
          </section>
        </div>
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

function EmptyText({ label }: { label: string }) {
  return <div style={{ fontSize: 12.5, color: "var(--text-tertiary)" }}>{label}</div>;
}

function formatPercent(value: number) {
  return `${Math.round(value * 100)}%`;
}
