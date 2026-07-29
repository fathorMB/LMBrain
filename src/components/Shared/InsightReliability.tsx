import { useEffect, useState } from "react";
import { useWorkspace } from "../../hooks/useWorkspace";
import { getProjectStatistics } from "../../lib/commands";
import { buildDiagnosticFixPrompt } from "../../lib/diagnosticPrompt";
import type { KitDiagnostic, ProjectStatistics } from "../../types";

export interface InsightReliabilityProps {
  reviewsWithoutSpec?: number;
  reviewsWithoutCreated?: number;
  errors?: number;
  warnings?: number;
  diagnostics?: KitDiagnostic[];
}

export function InsightReliability({
  reviewsWithoutSpec: propReviewsWithoutSpec,
  reviewsWithoutCreated: propReviewsWithoutCreated,
  errors: propErrors,
  warnings: propWarnings,
  diagnostics: propDiagnostics,
}: InsightReliabilityProps) {
  const { state: workspaceState } = useWorkspace();
  const needsFetch = propReviewsWithoutSpec === undefined || propReviewsWithoutCreated === undefined;
  const [stats, setStats] = useState<ProjectStatistics | null>(null);
  const [loadingStats, setLoadingStats] = useState(needsFetch);
  const [statsError, setStatsError] = useState<string | null>(null);

  useEffect(() => {
    if (!needsFetch) return;
    let active = true;
    getProjectStatistics()
      .then((data) => {
        if (active) {
          setStats(data);
          setStatsError(null);
        }
      })
      .catch((err) => {
        if (active) {
          console.error(err);
          setStatsError(typeof err === "string" ? err : "Failed to load review quality metrics.");
        }
      })
      .finally(() => {
        if (active) setLoadingStats(false);
      });
    return () => {
      active = false;
    };
  }, [needsFetch]);

  const diagnostics = propDiagnostics ?? workspaceState.diagnostics ?? [];
  const errors = propErrors ?? diagnostics.filter((d: KitDiagnostic) => d.severity === "error").length;
  const warnings = propWarnings ?? diagnostics.filter((d: KitDiagnostic) => d.severity === "warning").length;
  const reviewsWithoutSpec = propReviewsWithoutSpec ?? stats?.review_quality.reviews_without_spec ?? 0;
  const reviewsWithoutCreated = propReviewsWithoutCreated ?? stats?.review_quality.reviews_without_created ?? 0;

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
  const orderedDiagnostics = [...diagnostics].sort((a: KitDiagnostic, b: KitDiagnostic) => (severityOrder[a.severity] ?? 9) - (severityOrder[b.severity] ?? 9));

  if (loadingStats && !stats) {
    return <div style={{ fontSize: 12.5, color: "var(--text-tertiary)" }}>Loading reliability details…</div>;
  }

  if (statsError && needsFetch && !stats) {
    return <div style={{ fontSize: 12.5, color: "#e0584a" }}>{statsError}</div>;
  }

  return (
    <div>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "start", gap: 24, marginBottom: 15 }}>
        <div>
          <div role="status" style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 7, color: summaryColor, fontSize: 14, fontWeight: 750 }}>
            <span aria-hidden="true" style={{ width: 8, height: 8, borderRadius: "50%", background: summaryColor }} />
            {summary}
          </div>
          <p style={{ margin: "0 0 11px", fontSize: 11.5, lineHeight: 1.5, color: "var(--text-tertiary)", maxWidth: 680 }}>
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
        Review and resolve diagnostic issues to maintain reliable project metrics.
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
