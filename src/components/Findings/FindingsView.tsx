import { useMemo, useState } from "react";
import { getFindingContext, getFindings } from "../../lib/commands";
import type { Finding, FindingContext, FindingRelation } from "../../types";
import { useWorkspace } from "../../hooks/useWorkspace";
import { RefreshButton } from "../RefreshButton";
import { MarkdownRenderer } from "../../lib/markdown";

const ACTIVE = new Set(["open", "planned", "deferred"]);
const severityRank: Record<string, number> = {
  critical: 5, high: 4, medium: 3, low: 2, info: 1,
};

export function FindingsView() {
  const { state, dispatch, openDetailArtifact } = useWorkspace();
  const [scope, setScope] = useState<"active" | "history" | "all">("active");
  const [status, setStatus] = useState("all");
  const [severity, setSeverity] = useState("all");
  const [category, setCategory] = useState("all");
  const [query, setQuery] = useState("");
  const [sort, setSort] = useState<"severity" | "age" | "updated" | "milestone">("severity");
  const [selected, setSelected] = useState<FindingContext | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const findings = useMemo(() => {
    const filtered = state.findings.filter((finding) => {
      const inScope = scope === "all"
        || (scope === "active" ? ACTIVE.has(finding.status) : !ACTIVE.has(finding.status));
      const text = [
        finding.id, finding.title, finding.area, finding.milestone, finding.owner,
        finding.origin_artifact, ...finding.target_specs,
      ].filter(Boolean).join(" ").toLowerCase();
      return inScope
        && (status === "all" || finding.status === status)
        && (severity === "all" || finding.severity === severity)
        && (category === "all" || finding.category === category)
        && text.includes(query.trim().toLowerCase());
    });
    return filtered.sort((left, right) => {
      if (sort === "severity") {
        return (severityRank[right.severity] ?? 0) - (severityRank[left.severity] ?? 0)
          || left.id.localeCompare(right.id);
      }
      if (sort === "age") return left.created.localeCompare(right.created);
      if (sort === "updated") return right.updated.localeCompare(left.updated);
      return (left.milestone ?? "ZZZ").localeCompare(right.milestone ?? "ZZZ")
        || left.id.localeCompare(right.id);
    });
  }, [state.findings, scope, status, severity, category, query, sort]);

  const categories = [...new Set(state.findings.map((finding) => finding.category).filter(Boolean))].sort();
  const counts = Object.fromEntries(
    ["open", "planned", "deferred", "accepted-risk", "resolved", "superseded"]
      .map((value) => [value, state.findings.filter((finding) => finding.status === value).length]),
  );
  const refresh = async () => {
    setLoading(true); setError(null);
    try {
      dispatch({ type: "SET_FINDINGS", findings: await getFindings() });
    } catch (reason) {
      setError(message(reason));
    } finally {
      setLoading(false);
    }
  };
  const open = async (finding: Finding) => {
    setLoading(true); setError(null);
    try {
      setSelected(await getFindingContext(finding.id));
    } catch (reason) {
      setError(message(reason));
    } finally {
      setLoading(false);
    }
  };

  return <div style={{ height: "100%", overflow: "auto", padding: "22px 28px 70px" }}>
    <header style={{ display: "flex", alignItems: "flex-start", justifyContent: "space-between", gap: 12 }}>
      <div>
        <h1 style={{ margin: 0, fontSize: 24 }}>Findings</h1>
        <p style={muted}>Durable cross-spec observations and obligations. This view is read-only.</p>
      </div>
      <RefreshButton loading={loading} onClick={refresh} />
    </header>
    {error && <div role="alert" style={errorStyle}>{error}</div>}
    {loading && <p role="status" style={muted}>Loading findings…</p>}

    <section aria-label="Finding summary" style={summaryGrid}>
      {Object.entries(counts).map(([label, count]) => <div key={label} style={card}>
        <div style={summaryValue}>{count}</div>
        <div style={muted}>{label}</div>
      </div>)}
    </section>

    <section aria-label="Finding filters" style={filters}>
      <label style={filterLabel}>Scope
        <select style={filterControl} aria-label="Finding scope" value={scope} onChange={(event) => setScope(event.target.value as typeof scope)}>
          <option value="active">Active</option>
          <option value="history">History</option>
          <option value="all">All</option>
        </select>
      </label>
      <label style={filterLabel}>Status
        <select style={filterControl} aria-label="Finding status" value={status} onChange={(event) => setStatus(event.target.value)}>
          <option value="all">All</option>
          {["open", "planned", "deferred", "resolved", "accepted-risk", "superseded"].map(option)}
        </select>
      </label>
      <label style={filterLabel}>Severity
        <select style={filterControl} aria-label="Finding severity" value={severity} onChange={(event) => setSeverity(event.target.value)}>
          <option value="all">All</option>
          {["critical", "high", "medium", "low", "info"].map(option)}
        </select>
      </label>
      <label style={filterLabel}>Category
        <select style={filterControl} aria-label="Finding category" value={category} onChange={(event) => setCategory(event.target.value)}>
          <option value="all">All</option>
          {categories.map(option)}
        </select>
      </label>
      <label style={filterLabel}>Sort
        <select style={filterControl} aria-label="Finding sort" value={sort} onChange={(event) => setSort(event.target.value as typeof sort)}>
          {["severity", "age", "updated", "milestone"].map(option)}
        </select>
      </label>
      <label style={{ ...filterLabel, flex: "1 1 240px" }}>Search
        <input
          style={{ ...filterControl, width: "100%" }}
          aria-label="Search findings"
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          placeholder="Owner, area, milestone, target…"
        />
      </label>
    </section>

    {state.findings.length === 0 && !loading && <div style={empty}>No first-class findings exist. Legacy review entries are not promoted automatically.</div>}
    {state.findings.length > 0 && findings.length === 0 && <div style={empty}>No findings match these filters.</div>}
    <div style={{ display: "grid", gap: 10 }}>
      {findings.map((finding) => <button
        key={finding.id}
        type="button"
        onClick={() => void open(finding)}
        aria-label={`Open ${finding.id}: ${finding.title}`}
        style={findingCard}
      >
        <div style={{ display: "flex", gap: 9, alignItems: "center", flexWrap: "wrap" }}>
          <strong style={{ fontFamily: "var(--font-mono)" }}>{finding.id}</strong>
          <Indicator text={finding.severity} />
          <Indicator text={finding.status} />
          {finding.malformed && <Indicator text="malformed" />}
        </div>
        <div style={{ fontWeight: 650, marginTop: 7 }}>{finding.title}</div>
        <div style={meta}>
          <span>Origin: {finding.origin_artifact ?? "direct observation"}</span>
          <span>Owner: {finding.owner ?? "needs triage"}</span>
          <span>Targets: {finding.target_specs.join(", ") || "none"}</span>
          <span>Blockers: {finding.blocked_by.join(", ") || "none"}</span>
          <span>Updated: {finding.updated || "unknown"}</span>
        </div>
        <div style={stateLine(finding)}>
          {finding.malformed ? "Malformed — repair before lifecycle use"
            : finding.blocked_by.length ? "Blocked by canonical finding relationship"
            : finding.status === "planned" && finding.target_specs.length === 0 ? "Planned but missing a target"
            : finding.status === "open" && !finding.owner ? "Needs triage"
            : finding.status === "accepted-risk" ? "Accepted risk"
            : nextAction(finding)}
        </div>
      </button>)}
    </div>

    {selected && <FindingDetail
      context={selected}
      onClose={() => setSelected(null)}
      onOpenRelation={(relation) => openDetailArtifact({ title: `${relation.id}: ${relation.title}`, path: relation.path })}
      onOpenMarkdown={() => openDetailArtifact({ title: `${selected.finding.id}: ${selected.finding.title}`, path: selected.finding.path })}
    />}
  </div>;
}

function FindingDetail({ context, onClose, onOpenRelation, onOpenMarkdown }: {
  context: FindingContext;
  onClose: () => void;
  onOpenRelation: (relation: FindingRelation) => void;
  onOpenMarkdown: () => void;
}) {
  const groups: Array<[string, FindingRelation[]]> = [
    ["Origin", context.origin ? [context.origin] : []],
    ["Related work", [...context.related_specs, ...context.related_reviews]],
    ["Target specs", context.target_specs],
    ["Decisions", context.related_decisions],
    ["Blockers", context.blockers],
    ["Resolution evidence", context.resolution_refs],
    ["Superseded by", context.superseded_by ? [context.superseded_by] : []],
  ];
  const prompt = `Review ${context.finding.id} with finding_context, then use the appropriate governed finding_* MCP tool. Do not infer resolution from target spec status.`;
  const f = context.finding;
  const statusExplanation = f.status === "planned"
    ? "This finding is planned and routed to target spec(s), but is awaiting explicit resolution evidence."
    : f.status === "deferred"
    ? "This finding is deferred until declared revisit criteria are met."
    : f.status === "resolved"
    ? "This finding has been resolved with canonical evidence."
    : f.status === "accepted-risk"
    ? "Risk accepted by operator."
    : f.status === "superseded"
    ? "Superseded by newer finding or decision."
    : "Active open finding awaiting triage or assignment.";

  return <div role="dialog" aria-modal="true" aria-labelledby="finding-detail-title" style={dialogBackdrop}>
    <div style={dialog}>
      <div style={{ display: "flex", justifyContent: "space-between", gap: 12 }}>
        <div><div style={mono}>{f.id}</div><h2 id="finding-detail-title">{f.title}</h2></div>
        <button aria-label="Close finding detail" style={secondary} onClick={onClose}>Close</button>
      </div>

      <div style={{ display: "flex", gap: 6, margin: "10px 0 14px", flexWrap: "wrap" }}>
        <Indicator text={`Status: ${f.status}`} />
        <Indicator text={`Severity: ${f.severity}`} />
        <Indicator text={`Category: ${f.category}`} />
        {f.area && <Indicator text={`Area: ${f.area}`} />}
        {f.owner && <Indicator text={`Owner: ${f.owner}`} />}
        {f.milestone && <Indicator text={`Milestone: ${f.milestone}`} />}
      </div>

      <div style={{ padding: "10px 12px", background: "rgba(124,108,246,.08)", border: "1px solid rgba(124,108,246,.2)", borderRadius: 8, fontSize: 12, color: "#c2bdc8", marginBottom: 14 }}>
        <strong>State disposition:</strong> {statusExplanation}
      </div>

      <div style={{ display: "flex", gap: 16, flexWrap: "wrap", fontSize: 12, color: "var(--text-tertiary)", marginBottom: 14 }}>
        <div>Origin: <strong style={{ color: "var(--text-primary)" }}>{f.origin_artifact || "direct observation"}</strong></div>
        {f.origin_ref && <div>Origin Ref: <code style={{ fontFamily: "var(--font-mono)", color: "var(--accent-light)" }}>{f.origin_ref}</code></div>}
        <div>Created: {f.created || "—"}</div>
        <div>Updated: {f.updated || "—"}</div>
      </div>

      {f.body && <section style={{ marginBottom: 16 }}>
        <h3>Details & Statement</h3>
        <div style={{ padding: 14, background: "var(--bg-tertiary)", border: "1px solid var(--border-secondary)", borderRadius: 8, fontSize: 13, lineHeight: 1.6 }}>
          <MarkdownRenderer content={f.body} />
        </div>
      </section>}

      <div style={{ display: "flex", gap: 8, marginBottom: 16 }}>
        <button style={secondary} onClick={onOpenMarkdown}>Open Markdown</button>
        <button style={secondary} onClick={() => void navigator.clipboard.writeText(prompt)}>Copy governed action prompt</button>
      </div>
      <p style={muted}>Lifecycle actions are intentionally not available in the app.</p>
      {groups.map(([label, relations]) => <section key={label}><h3>{label}</h3>{relations.length === 0
        ? <p style={muted}>None declared.</p>
        : <div style={{ display: "flex", gap: 7, flexWrap: "wrap" }}>{relations.map((relation) => (
            <button key={relation.id} style={relationButton} onClick={() => onOpenRelation(relation)}>
              {relation.id} · {relation.title ? `${relation.title} (${relation.status})` : relation.status}
            </button>
          ))}</div>}</section>)}
      <section><h3>Resolution timeline</h3>{context.events.length === 0
        ? <p style={muted}>No typed events.</p>
        : <ol>{context.events.map((event, index) => <li key={String(event.id ?? index)} style={muted}>{String(event.timestamp ?? "")} · {String(event.action ?? "event")} · {String(event.rationale ?? "")}</li>)}</ol>}</section>
      {context.warnings.map((warning) => <div role="alert" key={warning} style={errorStyle}>{warning}</div>)}
    </div>
  </div>;
}

function Indicator({ text }: { text: string }) {
  return <span style={{ border: "1px solid var(--border-secondary)", borderRadius: 999, padding: "2px 7px", fontSize: 10.5 }}>{text}</span>;
}
function option(value: string) { return <option value={value} key={value}>{value}</option>; }
function nextAction(finding: Finding) {
  if (finding.status === "planned") return "Await explicit resolution evidence";
  if (finding.status === "deferred") return "Retained for its declared revisit condition";
  if (finding.status === "resolved") return "Resolved with canonical evidence";
  return "Review current disposition";
}
function stateLine(finding: Finding): React.CSSProperties {
  const attention = finding.malformed || finding.blocked_by.length > 0 || (finding.status === "open" && !finding.owner);
  return { marginTop: 9, fontSize: 11.5, color: attention ? "#d9b86d" : "var(--text-tertiary)" };
}
function message(value: unknown) { return value instanceof Error ? value.message : String(value); }

const muted: React.CSSProperties = { color: "var(--text-tertiary)", fontSize: 12.5, lineHeight: 1.55 };
const mono: React.CSSProperties = { fontFamily: "var(--font-mono)", color: "var(--text-tertiary)", fontSize: 11 };
const card: React.CSSProperties = { padding: 13, border: "1px solid var(--border-secondary)", borderRadius: 9, background: "var(--bg-tertiary)" };
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
const findingCard: React.CSSProperties = { ...card, width: "100%", color: "var(--text-primary)", textAlign: "left", cursor: "pointer" };
const meta: React.CSSProperties = { ...muted, display: "flex", flexWrap: "wrap", gap: "4px 16px", marginTop: 6 };
const secondary: React.CSSProperties = { border: "1px solid var(--border-secondary)", borderRadius: 7, background: "var(--bg-secondary)", color: "var(--text-secondary)", padding: "7px 11px", cursor: "pointer" };
const relationButton: React.CSSProperties = { ...secondary, fontFamily: "var(--font-mono)", fontSize: 11 };
const errorStyle: React.CSSProperties = { padding: 10, margin: "10px 0", borderRadius: 7, background: "rgba(224,88,74,.10)", color: "#e9857b", fontSize: 12 };
const empty: React.CSSProperties = { ...muted, padding: 24, textAlign: "center", border: "1px dashed var(--border-secondary)", borderRadius: 9 };
const dialogBackdrop: React.CSSProperties = { position: "fixed", inset: 0, zIndex: 60, background: "rgba(6,5,8,.72)", display: "grid", placeItems: "center", padding: 20 };
const dialog: React.CSSProperties = { width: "min(800px, 94vw)", maxHeight: "88vh", overflow: "auto", padding: 22, background: "var(--bg-secondary)", border: "1px solid var(--border-primary)", borderRadius: 13 };
