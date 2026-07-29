import { useMemo, useState, useEffect } from "react";
import { getKitFeedback } from "../../lib/commands";
import type { KitFeedbackReport, KitFeedbackNote } from "../../types";
import { RefreshButton } from "../RefreshButton";

const severityRank: Record<string, number> = {
  blocking: 5, critical: 4, high: 3, medium: 2, low: 1, info: 0,
};

export function FeedbackView() {
  const [report, setReport] = useState<KitFeedbackReport | null>(null);
  const [severity, setSeverity] = useState("all");
  const [category, setCategory] = useState("all");
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
        && text.includes(query.trim().toLowerCase());
    }).sort((left, right) => {
      return (severityRank[right.severity] ?? 0) - (severityRank[left.severity] ?? 0)
        || right.timestamp.localeCompare(left.timestamp);
    });
  }, [report, severity, category, query]);

  const categories = Object.keys(report?.counts_by_category ?? {}).sort();
  const severities = Object.keys(report?.counts_by_severity ?? {}).sort((a, b) => 
    (severityRank[b] ?? 0) - (severityRank[a] ?? 0)
  );

  return (
    <div style={{ height: "100%", overflow: "auto", padding: "22px 28px 70px" }}>
      <header style={{ display: "flex", alignItems: "flex-start", justifyContent: "space-between", gap: 12 }}>
        <div>
          <h1 style={{ margin: 0, fontSize: 24 }}>Kit Feedback</h1>
          <p style={muted}>Evidence-backed observations about LMBrain itself. This view is read-only.</p>
        </div>
        <RefreshButton loading={loading} onClick={fetchFeedback} />
      </header>
      
      {error && <div role="alert" style={errorStyle}>{error}</div>}
      {loading && !report && <p role="status" style={muted}>Loading kit feedback…</p>}

      {!loading && report && report.total === 0 && (
        <div style={empty}>No kit feedback notes found.</div>
      )}

      {report && report.total > 0 && (
        <>
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
          
          <div style={{ display: "grid", gap: 10 }}>
            {filteredNotes.map((note) => (
              <FeedbackNoteCard key={note.id} note={note} />
            ))}
          </div>
        </>
      )}
    </div>
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
        <div style={{ marginTop: 12, borderTop: "1px solid var(--border-secondary)", paddingTop: 12, fontSize: 13 }}>
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
            <div style={{ whiteSpace: "pre-wrap", fontFamily: "var(--font-mono)", fontSize: 12 }}>{note.evidence}</div>
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
    fontSize: 10.5
  };
}

const tagBadgeStyle: React.CSSProperties = {
  border: "1px solid var(--border-secondary)",
  color: "var(--text-tertiary)",
  borderRadius: 999,
  padding: "2px 7px",
  fontSize: 10.5
};

const muted: React.CSSProperties = { color: "var(--text-tertiary)", fontSize: 12.5, lineHeight: 1.55 };
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
  fontSize: 11.5,
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
  fontSize: 12,
};
const findingCard: React.CSSProperties = { ...card, width: "100%", color: "var(--text-primary)", textAlign: "left" };
const meta: React.CSSProperties = { ...muted, display: "flex", flexWrap: "wrap", gap: "4px 16px", marginTop: 6 };
const errorStyle: React.CSSProperties = { padding: 10, margin: "10px 0", borderRadius: 7, background: "rgba(224,88,74,.10)", color: "#e9857b", fontSize: 12 };
const empty: React.CSSProperties = { ...muted, padding: 24, textAlign: "center", border: "1px dashed var(--border-secondary)", borderRadius: 9 };
