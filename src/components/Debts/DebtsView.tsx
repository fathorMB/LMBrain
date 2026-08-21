import { useCallback, useMemo, useState } from "react";
import { getDebtContext } from "../../lib/commands";
import type { Debt, DebtContext, DebtRelation } from "../../types";
import { useWorkspace } from "../../hooks/useWorkspace";
import { useDialog } from "../../hooks/useDialog";
import { RefreshButton } from "../RefreshButton";
import { CardGrid, PageHeader, PageShell } from "../Shared/PageLayout";
import { FilterBar, FilterSelect, FilterSearchInput } from "../Shared/FilterBar";
import { MarkdownRenderer } from "../../lib/markdown";
import { ModalCloseButton } from "../Layout/ModalCloseButton";

const ACTIVE = new Set(["open", "planned", "deferred"]);
const severityRank: Record<string, number> = {
  critical: 5, high: 4, medium: 3, low: 2, info: 1,
};

export function DebtsView() {
  const { state, refreshWorkspaceData, openDetailArtifact } = useWorkspace();
  const [scope, setScope] = useState<"active" | "history" | "all">("active");
  const [status, setStatus] = useState("all");
  const [severity, setSeverity] = useState("all");
  const [category, setCategory] = useState("all");
  const [query, setQuery] = useState("");
  const [sort, setSort] = useState<"severity" | "age" | "updated" | "milestone">("severity");
  const [selected, setSelected] = useState<DebtContext | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const closeSelected = useCallback(() => setSelected(null), []);

  const debts = useMemo(() => {
    const filtered = state.debts.filter((debt) => {
      const inScope = scope === "all"
        || (scope === "active" ? ACTIVE.has(debt.status) : !ACTIVE.has(debt.status));
      const text = [
        debt.id, debt.title, debt.area, debt.milestone, debt.owner,
        debt.origin_artifact, ...debt.target_specs,
      ].filter(Boolean).join(" ").toLowerCase();
      return inScope
        && (status === "all" || debt.status === status)
        && (severity === "all" || debt.severity === severity)
        && (category === "all" || debt.category === category)
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
  }, [state.debts, scope, status, severity, category, query, sort]);

  const categories = [...new Set(state.debts.map((debt) => debt.category).filter(Boolean))].sort();
  const counts = Object.fromEntries(
    ["open", "planned", "deferred", "accepted-risk", "resolved", "superseded"]
      .map((value) => [value, state.debts.filter((debt) => debt.status === value).length]),
  );
  const refresh = async () => {
    setLoading(true); setError(null);
    try {
      await refreshWorkspaceData();
    } catch (reason) {
      setError(message(reason));
    } finally {
      setLoading(false);
    }
  };
  const open = async (debt: Debt) => {
    setLoading(true); setError(null);
    try {
      setSelected(await getDebtContext(debt.id));
    } catch (reason) {
      setError(message(reason));
    } finally {
      setLoading(false);
    }
  };

  return <PageShell archetype="dense">
    <PageHeader
      title="Debts"
      description="Durable cross-spec observations and obligations. This view is read-only."
      actions={<RefreshButton loading={loading} onClick={refresh} />}
    />
    {error && <div role="alert" style={errorStyle}>{error}</div>}
    {loading && <p role="status" style={muted}>Loading debts…</p>}

    <section aria-label="Debt summary" style={summaryGrid}>
      {Object.entries(counts).map(([label, count]) => <div key={label} style={card}>
        <div style={summaryValue}>{count}</div>
        <div style={muted}>{label}</div>
      </div>)}
    </section>

    <FilterBar ariaLabel="Debt filters">
      <FilterSelect
        label="Scope"
        ariaLabel="Debt scope"
        value={scope}
        onChange={(val) => setScope(val as typeof scope)}
        options={[
          { value: "active", label: "Active" },
          { value: "history", label: "History" },
          { value: "all", label: "All" },
        ]}
      />
      <FilterSelect
        label="Status"
        ariaLabel="Debt status"
        value={status}
        allLabel="All"
        onChange={setStatus}
        options={["open", "planned", "deferred", "resolved", "accepted-risk", "superseded"]}
      />
      <FilterSelect
        label="Severity"
        ariaLabel="Debt severity"
        value={severity}
        allLabel="All"
        onChange={setSeverity}
        options={["critical", "high", "medium", "low", "info"]}
      />
      <FilterSelect
        label="Category"
        ariaLabel="Debt category"
        value={category}
        allLabel="All"
        onChange={setCategory}
        options={categories}
      />
      <FilterSelect
        label="Sort"
        ariaLabel="Debt sort"
        value={sort}
        onChange={(val) => setSort(val as typeof sort)}
        options={["severity", "age", "updated", "milestone"]}
      />
      <FilterSearchInput
        label="Search"
        ariaLabel="Search debts"
        value={query}
        onChange={setQuery}
        placeholder="Owner, area, milestone, target…"
      />
    </FilterBar>

    {state.debts.length === 0 && !loading && <div style={empty}>No first-class debts exist. Legacy review entries are not promoted automatically.</div>}
    {state.debts.length > 0 && debts.length === 0 && <div style={empty}>No debts match these filters.</div>}
    <CardGrid>
      {debts.map((debt) => <button
        key={debt.id}
        type="button"
        onClick={() => void open(debt)}
        aria-label={`Open ${debt.id}: ${debt.title}`}
        style={debtCard}
      >
        <div style={{ display: "flex", gap: 9, alignItems: "center", flexWrap: "wrap" }}>
          <strong style={{ fontFamily: "var(--font-mono)" }}>{debt.id}</strong>
          <Indicator text={debt.severity} />
          <Indicator text={debt.status} />
          {debt.malformed && <Indicator text="malformed" />}
        </div>
        <div style={{ fontWeight: 650, marginTop: 7 }}>{debt.title}</div>
        <div style={meta}>
          <span>Origin: {debt.origin_artifact ?? "direct observation"}</span>
          <span>Owner: {debt.owner ?? "needs triage"}</span>
          <span>Targets: {debt.target_specs.join(", ") || "none"}</span>
          <span>Blockers: {debt.blocked_by.join(", ") || "none"}</span>
          <span>Updated: {debt.updated || "unknown"}</span>
        </div>
        <div style={stateLine(debt)}>
          {debt.malformed ? "Malformed — repair before lifecycle use"
            : debt.blocked_by.length ? "Blocked by canonical debt relationship"
            : debt.status === "planned" && debt.target_specs.length === 0 ? "Planned but missing a target"
            : debt.status === "open" && !debt.owner ? "Needs triage"
            : debt.status === "accepted-risk" ? "Accepted risk"
            : nextAction(debt)}
        </div>
      </button>)}
    </CardGrid>

    {selected && <DebtDetail
      context={selected}
      onClose={closeSelected}
      onOpenRelation={(relation) => openDetailArtifact({ title: `${relation.id}: ${relation.title}`, path: relation.path })}
      onOpenMarkdown={() => openDetailArtifact({ title: `${selected.debt.id}: ${selected.debt.title}`, path: selected.debt.path })}
    />}
  </PageShell>;
}

function DebtDetail({ context, onClose, onOpenRelation, onOpenMarkdown }: {
  context: DebtContext;
  onClose: () => void;
  onOpenRelation: (relation: DebtRelation) => void;
  onOpenMarkdown: () => void;
}) {
  const { dialogRef, handleKeyDown } = useDialog<HTMLDivElement>({ isOpen: true, onClose });
  const groups: Array<[string, DebtRelation[]]> = [
    ["Origin", context.origin ? [context.origin] : []],
    ["Related work", [...context.related_specs, ...context.related_reviews]],
    ["Target specs", context.target_specs],
    ["Decisions", context.related_decisions],
    ["Blockers", context.blockers],
    ["Resolution evidence", context.resolution_refs],
    ["Superseded by", context.superseded_by ? [context.superseded_by] : []],
  ];
  const prompt = `Review ${context.debt.id} with debt_context, then use the appropriate governed debt_* MCP tool. Do not infer resolution from target spec status.`;
  const f = context.debt;
  const statusExplanation = f.status === "planned"
    ? "This debt is planned and routed to target spec(s), but is awaiting explicit resolution evidence."
    : f.status === "deferred"
    ? "This debt is deferred until declared revisit criteria are met."
    : f.status === "resolved"
    ? "This debt has been resolved with canonical evidence."
    : f.status === "accepted-risk"
    ? "Risk accepted by operator."
    : f.status === "superseded"
    ? "Superseded by newer debt or decision."
    : "Active open debt awaiting triage or assignment.";

  return <div
    role="presentation"
    style={dialogBackdrop}
    onKeyDown={handleKeyDown}
    onMouseDown={(event) => {
      if (event.target === event.currentTarget) onClose();
    }}
  >
    <div
      ref={dialogRef}
      tabIndex={-1}
      role="dialog"
      aria-modal="true"
      aria-labelledby="debt-detail-title"
      style={dialog}
      onMouseDown={(event) => event.stopPropagation()}
    >
      <div style={dialogHeader}>
        <div style={{ minWidth: 0, flex: 1 }}>
          <div style={mono}>{f.id}</div>
          <h2 id="debt-detail-title" style={dialogTitle}>{f.title}</h2>
        </div>
        <ModalCloseButton label="Close debt detail" onClick={onClose} />
      </div>

      <div style={{ display: "flex", gap: 6, margin: "10px 0 14px", flexWrap: "wrap" }}>
        <Indicator text={`Status: ${f.status}`} />
        <Indicator text={`Severity: ${f.severity}`} />
        <Indicator text={`Category: ${f.category}`} />
        {f.area && <Indicator text={`Area: ${f.area}`} />}
        {f.owner && <Indicator text={`Owner: ${f.owner}`} />}
        {f.milestone && <Indicator text={`Milestone: ${f.milestone}`} />}
      </div>

      <div style={{ padding: "10px 12px", background: "rgba(124,108,246,.08)", border: "1px solid rgba(124,108,246,.2)", borderRadius: 8, fontSize: "var(--text-sm)", color: "#c2bdc8", marginBottom: 14 }}>
        <strong>State disposition:</strong> {statusExplanation}
      </div>

      <div style={{ display: "flex", gap: 16, flexWrap: "wrap", fontSize: "var(--text-sm)", color: "var(--text-tertiary)", marginBottom: 14 }}>
        <div>Origin: <strong style={{ color: "var(--text-primary)" }}>{f.origin_artifact || "direct observation"}</strong></div>
        {f.origin_ref && <div>Origin Ref: <code style={{ fontFamily: "var(--font-mono)", color: "var(--accent-light)" }}>{f.origin_ref}</code></div>}
        <div>Created: {f.created || "—"}</div>
        <div>Updated: {f.updated || "—"}</div>
      </div>

      {f.body && <section style={{ marginBottom: 16 }}>
        <h3>Details & Statement</h3>
        <div style={{ padding: 14, background: "var(--bg-tertiary)", border: "1px solid var(--border-secondary)", borderRadius: 8, fontSize: "var(--text-md)", lineHeight: 1.6 }}>
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
  return <span style={{ border: "1px solid var(--border-secondary)", borderRadius: 999, padding: "2px 7px", fontSize: "var(--text-xs)" }}>{text}</span>;
}
function option(value: string) { return <option value={value} key={value}>{value}</option>; }
function nextAction(debt: Debt) {
  if (debt.status === "planned") return "Await explicit resolution evidence";
  if (debt.status === "deferred") return "Retained for its declared revisit condition";
  if (debt.status === "resolved") return "Resolved with canonical evidence";
  return "Review current disposition";
}
function stateLine(debt: Debt): React.CSSProperties {
  const attention = debt.malformed || debt.blocked_by.length > 0 || (debt.status === "open" && !debt.owner);
  return { marginTop: 9, fontSize: "var(--text-xs)", color: attention ? "#d9b86d" : "var(--text-tertiary)" };
}
function message(value: unknown) { return value instanceof Error ? value.message : String(value); }

const muted: React.CSSProperties = { color: "var(--text-tertiary)", fontSize: "var(--text-sm)", lineHeight: 1.55 };
const mono: React.CSSProperties = { fontFamily: "var(--font-mono)", color: "var(--text-tertiary)", fontSize: "var(--text-xs)" };
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
const debtCard: React.CSSProperties = { ...card, width: "100%", color: "var(--text-primary)", textAlign: "left", cursor: "pointer" };
const meta: React.CSSProperties = { ...muted, display: "flex", flexWrap: "wrap", gap: "4px 16px", marginTop: 6 };
const secondary: React.CSSProperties = { border: "1px solid var(--border-secondary)", borderRadius: 7, background: "var(--bg-secondary)", color: "var(--text-secondary)", padding: "7px 11px", cursor: "pointer" };
const relationButton: React.CSSProperties = { ...secondary, fontFamily: "var(--font-mono)", fontSize: "var(--text-xs)" };
const errorStyle: React.CSSProperties = { padding: 10, margin: "10px 0", borderRadius: 7, background: "rgba(224,88,74,.10)", color: "#e9857b", fontSize: "var(--text-sm)" };
const empty: React.CSSProperties = { ...muted, padding: 24, textAlign: "center", border: "1px dashed var(--border-secondary)", borderRadius: 9 };
const dialogBackdrop: React.CSSProperties = { position: "fixed", inset: 0, zIndex: 60, background: "rgba(6,5,8,.72)", display: "grid", placeItems: "center", padding: 20 };
const dialog: React.CSSProperties = { width: "min(800px, 94vw)", maxHeight: "88vh", overflow: "auto", padding: 22, background: "var(--bg-secondary)", border: "1px solid var(--border-primary)", borderRadius: 13, outline: "none" };
const dialogHeader: React.CSSProperties = { display: "flex", alignItems: "flex-start", justifyContent: "space-between", gap: 12, minWidth: 0, marginBottom: 4 };
const dialogTitle: React.CSSProperties = { margin: 0, maxWidth: "100%", overflowWrap: "anywhere", fontSize: 20, lineHeight: 1.25 };
