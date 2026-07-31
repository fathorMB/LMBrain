import { useMemo, useState, useEffect } from "react";
import { getKitFeedback, saveTextFile } from "../../lib/commands";
import type { KitFeedbackReport, KitFeedbackNote } from "../../types";
import { RefreshButton } from "../RefreshButton";
import { CardGrid, PageHeader, PageShell } from "../Shared/PageLayout";

const severityRank: Record<string, number> = {
  blocking: 5, critical: 4, high: 3, medium: 2, low: 1, info: 0,
};

function exportFilename(version: string | null): string {
  const scope = version ? `v${version.replace(/[^a-zA-Z0-9._-]+/g, "_")}` : "all";
  return `lmbrain-kit-feedback-${scope}.json`;
}

function feedbackExportContent(report: KitFeedbackReport, version: string | null): string {
  const notes = version === null
    ? report.notes
    : report.notes.filter((note) => note.lmbrain_version === version);
  return JSON.stringify({
    schema_version: "1",
    source_report: report.path,
    scope: version === null ? "all" : "version",
    lmbrain_version: version,
    notes,
  }, null, 2);
}

async function saveFeedbackExport(report: KitFeedbackReport, version: string | null): Promise<boolean> {
  const { save } = await import("@tauri-apps/plugin-dialog");
  const path = await save({
    defaultPath: exportFilename(version),
    filters: [{ name: "JSON", extensions: ["json"] }],
  });
  if (!path) return false;
  await saveTextFile(path, feedbackExportContent(report, version));
  return true;
}

export function FeedbackView() {
  const [report, setReport] = useState<KitFeedbackReport | null>(null);
  const [severity, setSeverity] = useState("all");
  const [category, setCategory] = useState("all");
  const [version, setVersion] = useState("all");
  const [exportOpen, setExportOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchFeedback = async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await getKitFeedback();
      setReport(data);
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
      setReport(null);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    let active = true;
    getKitFeedback()
      .then((data) => {
        if (active) setReport(data);
      })
      .catch((reason: unknown) => {
        if (active) {
          setError(reason instanceof Error ? reason.message : String(reason));
          setReport(null);
        }
      })
      .finally(() => {
        if (active) setLoading(false);
      });
    return () => { active = false; };
  }, []);

  const filteredNotes = useMemo(() => {
    if (!report) return [];
    return report.notes.filter((note) => {
      const text = [
        note.id, note.summary, note.observed_behavior, note.expected_behavior,
        note.impact, note.workaround, note.suggested_improvement, note.actor
      ].filter(Boolean).join(" ").toLowerCase();
      
      return (severity === "all" || note.severity === severity)
        && (category === "all" || note.category === category)
        && (version === "all" || note.lmbrain_version === version)
        && text.includes(query.trim().toLowerCase());
    }).sort((left, right) => {
      return (severityRank[right.severity] ?? 0) - (severityRank[left.severity] ?? 0)
        || right.timestamp.localeCompare(left.timestamp);
    });
  }, [report, severity, category, version, query]);

  const categories = Object.keys(report?.counts_by_category ?? {}).sort();
  const severities = Object.keys(report?.counts_by_severity ?? {}).sort((a, b) => 
    (severityRank[b] ?? 0) - (severityRank[a] ?? 0)
  );
  const versions = [...new Set((report?.notes ?? []).map((note) => note.lmbrain_version).filter(Boolean))]
    .sort((left, right) => right.localeCompare(left, undefined, { numeric: true, sensitivity: "base" }));

  const handleExport = async (selectedVersion: string | null) => {
    if (!report) return;
    setError(null);
    try {
      await saveFeedbackExport(report, selectedVersion);
      setExportOpen(false);
    } catch (reason) {
      setError(`Unable to export feedback: ${reason instanceof Error ? reason.message : String(reason)}`);
    }
  };

  return (
    <PageShell archetype="dense">
      <PageHeader
        title="Kit Feedback"
        description="Evidence-backed observations about LMBrain itself. This view is read-only."
        actions={<RefreshButton loading={loading} onClick={fetchFeedback} />}
      />

      {error && <div role="alert" style={errorStyle}>{error}</div>}
      {loading && !report && <p role="status" style={muted}>Loading kit feedback…</p>}

      {!loading && report && report.total === 0 && (
        <div style={empty}>No kit feedback notes found.</div>
      )}

      {report && report.total > 0 && (
        <>
          <div style={{ display: "flex", justifyContent: "flex-end", margin: "18px 0 0" }}>
            <button
              type="button"
              aria-expanded={exportOpen}
              aria-controls="feedback-export-options"
              onClick={() => setExportOpen((open) => !open)}
              style={exportButton}
            >
              <i className="material-symbols-outlined" aria-hidden="true" style={{ fontSize: 17 }}>download</i>
              Export feedback
            </button>
          </div>
          {exportOpen && (
            <section id="feedback-export-options" aria-label="Feedback export options" style={exportOptions}>
              <span style={muted}>Exports use all loaded notes and ignore the current view filters.</span>
              <button type="button" onClick={() => void handleExport(null)} style={exportOptionButton}>
                Download all items
              </button>
              {versions.map((item) => (
                <button key={item} type="button" onClick={() => void handleExport(item)} style={exportOptionButton}>
                  Download v{item}
                </button>
              ))}
            </section>
          )}

          <section aria-label="Feedback summary" style={summaryGrid}>
            <div style={card}>
              <div style={summaryValue}>{report.total}</div>
              <div style={muted}>Total</div>
            </div>
            {Object.entries(report.counts_by_severity).sort((a, b) => (severityRank[b[0]] ?? 0) - (severityRank[a[0]] ?? 0)).map(([label, count]) => (
              <div key={`sev-${label}`} style={card}>
                <div style={summaryValue}>{count}</div>
                <div style={muted}>{label} (severity)</div>
              </div>
            ))}
            {Object.entries(report.counts_by_category).map(([label, count]) => (
              <div key={`cat-${label}`} style={card}>
                <div style={summaryValue}>{count}</div>
                <div style={muted}>{label}</div>
              </div>
            ))}
          </section>

          <section aria-label="Feedback filters" style={filters}>
            <label style={filterLabel}>Version
              <select style={filterControl} aria-label="Feedback version" value={version} onChange={(event) => setVersion(event.target.value)}>
                <option value="all">All versions</option>
                {versions.map((item) => <option value={item} key={item}>v{item}</option>)}
              </select>
            </label>
            <label style={filterLabel}>Severity
              <select style={filterControl} aria-label="Feedback severity" value={severity} onChange={(event) => setSeverity(event.target.value)}>
                <option value="all">All</option>
                {severities.map((s) => <option value={s} key={s}>{s}</option>)}
              </select>
            </label>
            <label style={filterLabel}>Category
              <select style={filterControl} aria-label="Feedback category" value={category} onChange={(event) => setCategory(event.target.value)}>
                <option value="all">All</option>
                {categories.map((c) => <option value={c} key={c}>{c}</option>)}
              </select>
            </label>
            <label style={{ ...filterLabel, flex: "1 1 240px" }}>Search
              <input
                style={{ ...filterControl, width: "100%" }}
                aria-label="Search feedback"
                value={query}
                onChange={(event) => setQuery(event.target.value)}
                placeholder="Search notes…"
              />
            </label>
          </section>

          {filteredNotes.length === 0 && <div style={empty}>No feedback matches these filters.</div>}
          
          {/* Notes expand into multi-paragraph prose, so they get a wider
              column minimum than the 360px default. */}
          <CardGrid minColumnWidth={420}>
            {filteredNotes.map((note) => (
              <FeedbackNoteCard key={note.id} note={note} />
            ))}
          </CardGrid>
        </>
      )}
    </PageShell>
  );
}

function FeedbackNoteCard({ note }: { note: KitFeedbackNote }) {
  const [expanded, setExpanded] = useState(false);
  
  return (
    <div style={findingCard}>
      <div 
        onClick={() => setExpanded(!expanded)} 
        style={{ cursor: "pointer" }}
        role="button"
        aria-expanded={expanded}
      >
        <div style={{ display: "flex", gap: 9, alignItems: "center", flexWrap: "wrap" }}>
          <strong style={{ fontFamily: "var(--font-mono)", color: "#bcaef6" }}>{note.id}</strong>
          <span style={getSeverityBadgeStyle(note.severity)}>{note.severity}</span>
          <span style={tagBadgeStyle}>{note.category}</span>
          <span style={tagBadgeStyle}>v{note.lmbrain_version}</span>
        </div>
        <div style={{ fontWeight: 650, marginTop: 7 }}>{note.summary}</div>
      </div>
      
      {expanded && (
        <div style={{ marginTop: 12, borderTop: "1px solid var(--border-secondary)", paddingTop: 12, fontSize: "var(--text-md)" }}>
          <div style={{ marginBottom: 8 }}>
            <strong style={{ color: "var(--text-secondary)", display: "block", marginBottom: 2 }}>Observed Behavior</strong>
            <div style={{ whiteSpace: "pre-wrap" }}>{note.observed_behavior}</div>
          </div>
          
          <div style={{ marginBottom: 8 }}>
            <strong style={{ color: "var(--text-secondary)", display: "block", marginBottom: 2 }}>Expected Behavior</strong>
            <div style={{ whiteSpace: "pre-wrap" }}>{note.expected_behavior}</div>
          </div>
          
          <div style={{ marginBottom: 8 }}>
            <strong style={{ color: "var(--text-secondary)", display: "block", marginBottom: 2 }}>Impact</strong>
            <div style={{ whiteSpace: "pre-wrap" }}>{note.impact}</div>
          </div>
          
          <div style={{ marginBottom: 8 }}>
            <strong style={{ color: "var(--text-secondary)", display: "block", marginBottom: 2 }}>Evidence</strong>
            <div style={{ whiteSpace: "pre-wrap", fontFamily: "var(--font-mono)", fontSize: "var(--text-sm)" }}>{note.evidence}</div>
          </div>
          
          {note.workaround && (
            <div style={{ marginBottom: 8 }}>
              <strong style={{ color: "var(--text-secondary)", display: "block", marginBottom: 2 }}>Workaround</strong>
              <div style={{ whiteSpace: "pre-wrap" }}>{note.workaround}</div>
            </div>
          )}
          
          {note.suggested_improvement && (
            <div style={{ marginBottom: 8 }}>
              <strong style={{ color: "var(--text-secondary)", display: "block", marginBottom: 2 }}>Suggested Improvement</strong>
              <div style={{ whiteSpace: "pre-wrap" }}>{note.suggested_improvement}</div>
            </div>
          )}
          
          <div style={{ display: "flex", justifyContent: "space-between", marginTop: 12, ...meta }}>
            <span>Actor: {note.actor}</span>
            <span>Date: {note.timestamp}</span>
            {note.related_note && <span>Related: {note.related_note}</span>}
          </div>
        </div>
      )}
    </div>
  );
}

function getSeverityBadgeStyle(severity: string): React.CSSProperties {
  let color = "var(--text-tertiary)";
  if (severity === "blocking" || severity === "critical" || severity === "high") color = "#f0a2a2";
  else if (severity === "medium") color = "#d6b277";
  else if (severity === "low") color = "#91d5ad";
  else if (severity === "info") color = "#9aadcf";
  
  return {
    border: `1px solid ${color}`,
    color: color,
    borderRadius: 999,
    padding: "2px 7px",
    fontSize: "var(--text-xs)"
  };
}

const tagBadgeStyle: React.CSSProperties = {
  border: "1px solid var(--border-secondary)",
  color: "var(--text-tertiary)",
  borderRadius: 999,
  padding: "2px 7px",
  fontSize: "var(--text-xs)"
};

const muted: React.CSSProperties = { color: "var(--text-tertiary)", fontSize: "var(--text-sm)", lineHeight: 1.55 };
const card: React.CSSProperties = { padding: 13, border: "1px solid #2a2631", borderRadius: 9, background: "#121016" };
const summaryGrid: React.CSSProperties = { display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(120px, 1fr))", gap: 9, margin: "18px 0" };
const summaryValue: React.CSSProperties = { fontFamily: "var(--font-mono)", fontSize: 20, fontWeight: 700 };
const filters: React.CSSProperties = {
  display: "flex",
  alignItems: "flex-end",
  flexWrap: "wrap",
  gap: 12,
  padding: 14,
  marginBottom: 14,
  border: "1px solid var(--border-secondary)",
  borderRadius: 9,
  background: "var(--bg-secondary)",
};
const filterLabel: React.CSSProperties = {
  display: "grid",
  gap: 6,
  flex: "1 1 118px",
  minWidth: 0,
  color: "var(--text-tertiary)",
  fontSize: "var(--text-xs)",
  fontWeight: 650,
};
const filterControl: React.CSSProperties = {
  minWidth: 0,
  height: 34,
  boxSizing: "border-box",
  border: "1px solid var(--border-primary)",
  borderRadius: 7,
  outline: "none",
  background: "var(--bg-tertiary)",
  color: "var(--text-primary)",
  colorScheme: "dark",
  padding: "0 9px",
  fontFamily: "inherit",
  fontSize: "var(--text-sm)",
};
const findingCard: React.CSSProperties = { ...card, width: "100%", color: "var(--text-primary)", textAlign: "left" };
const meta: React.CSSProperties = { ...muted, display: "flex", flexWrap: "wrap", gap: "4px 16px", marginTop: 6 };
const errorStyle: React.CSSProperties = { padding: 10, margin: "10px 0", borderRadius: 7, background: "rgba(224,88,74,.10)", color: "#e9857b", fontSize: "var(--text-sm)" };
const empty: React.CSSProperties = { ...muted, padding: 24, textAlign: "center", border: "1px dashed var(--border-secondary)", borderRadius: 9 };
const exportButton: React.CSSProperties = { display: "inline-flex", alignItems: "center", gap: 7, border: "1px solid var(--border-primary)", borderRadius: 7, padding: "8px 11px", background: "var(--bg-secondary)", color: "var(--text-primary)", cursor: "pointer", fontFamily: "inherit", fontSize: "var(--text-sm)", fontWeight: 650 };
const exportOptions: React.CSSProperties = { display: "flex", alignItems: "center", flexWrap: "wrap", gap: 8, padding: 12, margin: "10px 0 0", border: "1px solid var(--border-secondary)", borderRadius: 9, background: "var(--bg-secondary)" };
const exportOptionButton: React.CSSProperties = { border: "1px solid var(--border-primary)", borderRadius: 7, padding: "7px 9px", background: "var(--bg-tertiary)", color: "var(--text-primary)", cursor: "pointer", fontFamily: "inherit", fontSize: "var(--text-xs)" };
