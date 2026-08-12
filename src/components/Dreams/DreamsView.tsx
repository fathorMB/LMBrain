import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { Dream } from "../../types";
import { getDreams } from "../../lib/commands";
import { useWorkspace } from "../../hooks/useWorkspace";
import { MarkdownRenderer } from "../../lib/markdown";
import { RefreshButton } from "../RefreshButton";
import { CardGrid, EmptyState, PageHeader, PageShell } from "../Shared/PageLayout";
import { ModalCloseButton } from "../Layout/ModalCloseButton";

export function DreamsView() {
  const { state, dispatch } = useWorkspace();
  const [status, setStatus] = useState("all");
  const [classification, setClassification] = useState("all");
  const [area, setArea] = useState("all");
  const [confidence, setConfidence] = useState("all");
  const [selected, setSelected] = useState<Dream | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const closeSelected = useCallback(() => setSelected(null), []);
  const dreams = useMemo(
    () => state.dreams.filter((dream) =>
      (status === "all" || dream.status === status)
      && (classification === "all" || dream.classification === classification)
      && (area === "all" || dream.area === area)
      && (confidence === "all" || dream.confidence === confidence)),
    [state.dreams, status, classification, area, confidence],
  );
  const options = (values: string[]) => [...new Set(values.filter(Boolean))].sort();
  const refresh = async () => {
    setLoading(true);
    setError(null);
    try {
      dispatch({ type: "MERGE_DATA", data: { dreams: await getDreams() } });
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : "Dream Journal could not be refreshed.");
    } finally {
      setLoading(false);
    }
  };

  return <PageShell archetype="dense">
    <PageHeader
      title="Dream Journal"
      description="Tentative, grounded technical and design debt captured during explicitly invited Project Lead dreaming sessions."
      actions={<RefreshButton onClick={refresh} loading={loading} />}
    />
    <div style={readOnlyNotice}>
      <i className="material-symbols-outlined" aria-hidden="true">visibility</i>
      <span>Read-only. Dreams never enter delivery automatically; promotion is always an explicit governed action.</span>
    </div>
    {error && <div role="alert" style={errorStyle}>{error}</div>}
    <section aria-label="Dream filters" style={filters}>
      <Filter label="State" value={status} onChange={setStatus} options={options(state.dreams.map((dream) => dream.status))} />
      <Filter label="Kind" value={classification} onChange={setClassification} options={options(state.dreams.map((dream) => dream.classification))} />
      <Filter label="Area" value={area} onChange={setArea} options={options(state.dreams.map((dream) => dream.area ?? ""))} />
      <Filter label="Confidence" value={confidence} onChange={setConfidence} options={options(state.dreams.map((dream) => dream.confidence))} />
    </section>
    {loading && <p role="status" style={muted}>Loading Dream Journal…</p>}
    {!loading && state.dreams.length === 0 && <EmptyState>No dreams captured yet. An explicitly invited dreaming session may produce zero or more grounded records.</EmptyState>}
    {!loading && state.dreams.length > 0 && dreams.length === 0 && <EmptyState>No dreams match these filters.</EmptyState>}
    <CardGrid minColumnWidth={320}>
      {dreams.map((dream) => <button
        key={dream.path}
        type="button"
        aria-label={`Open ${dream.id}: ${dream.title}`}
        onClick={() => setSelected(dream)}
        style={dreamCard}
      >
        <div style={cardHeading}>
          <div style={{ minWidth: 0 }}>
            <span style={idStyle}>{dream.id}</span>
            <h2 style={cardTitle}>{dream.title}</h2>
          </div>
          <i className="material-symbols-outlined" aria-hidden="true" style={{ color: "var(--text-tertiary)" }}>chevron_right</i>
        </div>
        <div style={chips}>
          <Chip text={dream.malformed ? "malformed" : dream.status} />
          <Chip text={dream.classification} />
          {dream.area && <Chip text={dream.area} />}
          <Chip text={`${dream.confidence} confidence`} />
        </div>
        <p style={preview}>{plainPreview(dream.body)}</p>
        <span style={openHint}>Open full record</span>
      </button>)}
    </CardGrid>
    {selected && <DreamDetail dream={selected} onClose={closeSelected} />}
  </PageShell>;
}

function DreamDetail({ dream, onClose }: { dream: Dream; onClose: () => void }) {
  const dialogRef = useRef<HTMLDivElement | null>(null);
  useEffect(() => {
    const previousFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        onClose();
        return;
      }
      if (event.key !== "Tab" || !dialogRef.current) return;
      const focusable = Array.from(dialogRef.current.querySelectorAll<HTMLElement>(
        "button:not([disabled]), [href], [tabindex]:not([tabindex='-1'])",
      ));
      if (focusable.length === 0) return;
      const first = focusable[0];
      const last = focusable[focusable.length - 1];
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault(); last.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault(); first.focus();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    dialogRef.current?.focus();
    return () => { window.removeEventListener("keydown", handleKeyDown); previousFocus?.focus(); };
  }, [onClose]);
  const prompt = `Review ${dream.path} and explicitly choose whether to triage, promote, or discard ${dream.id}.`;
  return <div role="presentation" style={backdrop} onMouseDown={(event) => { if (event.target === event.currentTarget) onClose(); }}>
    <div ref={dialogRef} tabIndex={-1} role="dialog" aria-modal="true" aria-labelledby="dream-detail-title" style={dialog} onMouseDown={(event) => event.stopPropagation()}>
      <header style={dialogHeader}>
        <div style={{ minWidth: 0 }}><span style={idStyle}>{dream.id}</span><h2 id="dream-detail-title" style={dialogTitle}>{dream.title}</h2></div>
        <ModalCloseButton label="Close dream detail" onClick={onClose} />
      </header>
      <div style={chips}>
        <Chip text={dream.malformed ? "malformed" : dream.status} />
        <Chip text={dream.classification} />
        {dream.area && <Chip text={dream.area} />}
        <Chip text={`${dream.confidence} confidence`} />
      </div>
      <section aria-label="Dream content" style={markdownPanel}>
        <MarkdownRenderer content={dream.body || "_No readable rationale._"} />
      </section>
      <details style={metadataPanel}>
        <summary style={summary}>Provenance and suggested disposition</summary>
        <dl style={metadataGrid}>
          <dt>Context digest</dt><dd><code>{dream.context_digest || "unavailable"}</code></dd>
          <dt>Related artifacts</dt><dd>{dream.related_artifacts.join(", ") || "none"}</dd>
          <dt>Captured</dt><dd>{dream.created || "unknown"}</dd>
          <dt>Updated</dt><dd>{dream.updated || "unknown"}</dd>
        </dl>
        <button type="button" style={secondaryButton} onClick={() => void navigator.clipboard?.writeText(prompt)}>Copy governed action prompt</button>
      </details>
    </div>
  </div>;
}

function Filter({ label, value, onChange, options }: { label: string; value: string; onChange: (value: string) => void; options: string[] }) {
  return <label style={filterLabel}>{label}<select className="app-select" style={filterControl} aria-label={`Dream ${label.toLowerCase()}`} value={value} onChange={(event) => onChange(event.target.value)}><option value="all">All</option>{options.map((option) => <option key={option} value={option}>{option}</option>)}</select></label>;
}
function Chip({ text }: { text: string }) { return <span style={chip}>{text}</span>; }
function plainPreview(markdown: string) {
  const text = markdown.replace(/```[\s\S]*?```/g, " ").replace(/[^\w\s:/.()-]/g, " ").replace(/\s+/g, " ").trim();
  if (!text) return "No readable rationale.";
  return text.length > 180 ? `${text.slice(0, 177).trimEnd()}…` : text;
}

const muted = { color: "var(--text-tertiary)", fontSize: "var(--text-sm)" } as const;
const filters = { display: "flex", alignItems: "flex-end", flexWrap: "wrap", gap: 12, padding: 14, marginBottom: 14, border: "1px solid var(--border-secondary)", borderRadius: 9, background: "var(--bg-secondary)" } as const;
const filterLabel = { display: "grid", gap: 6, flex: "1 1 118px", minWidth: 0, color: "var(--text-tertiary)", fontSize: "var(--text-xs)", fontWeight: 650 } as const;
const filterControl = { minWidth: 0, height: 34, boxSizing: "border-box", border: "1px solid var(--border-primary)", borderRadius: 7, outline: "none", background: "var(--bg-tertiary)", color: "var(--text-primary)", colorScheme: "dark", padding: "0 9px", fontFamily: "inherit", fontSize: "var(--text-sm)" } as const;
const readOnlyNotice = { display: "flex", gap: 8, alignItems: "center", margin: "0 0 16px", padding: "9px 11px", border: "1px solid var(--border-secondary)", borderRadius: 8, color: "var(--text-tertiary)", fontSize: "var(--text-sm)" } as const;
const dreamCard = { width: "100%", minWidth: 0, border: "1px solid var(--border-primary)", borderLeft: "3px solid #7768d8", background: "var(--bg-secondary)", color: "var(--text-primary)", borderRadius: 11, padding: 15, cursor: "pointer", textAlign: "left" } as const;
const cardHeading = { display: "flex", justifyContent: "space-between", alignItems: "flex-start", gap: 12 } as const;
const idStyle = { display: "block", fontFamily: "var(--font-mono)", color: "#bcaef6", fontSize: "var(--text-xs)", marginBottom: 3 } as const;
const cardTitle = { margin: 0, fontSize: "var(--text-md)", lineHeight: 1.35, overflowWrap: "anywhere" } as const;
const chips = { display: "flex", flexWrap: "wrap", gap: 6, margin: "11px 0" } as const;
const chip = { display: "inline-flex", border: "1px solid var(--border-secondary)", borderRadius: 999, padding: "3px 8px", color: "var(--text-secondary)", background: "var(--bg-tertiary)", fontSize: "var(--text-xs)" } as const;
const preview = { margin: "8px 0", color: "var(--text-secondary)", fontSize: "var(--text-sm)", lineHeight: 1.55, display: "-webkit-box", WebkitLineClamp: 3, WebkitBoxOrient: "vertical", overflow: "hidden" } as const;
const openHint = { color: "#bcaef6", fontSize: "var(--text-xs)", fontWeight: 650 } as const;
const backdrop = { position: "fixed", inset: 0, zIndex: 60, display: "grid", placeItems: "center", padding: 20, background: "rgba(6,5,8,.72)" } as const;
const dialog = { width: "min(780px, 94vw)", maxHeight: "88vh", overflow: "auto", border: "1px solid var(--border-primary)", borderRadius: 13, padding: 22, outline: "none", background: "var(--bg-secondary)" } as const;
const dialogHeader = { display: "flex", justifyContent: "space-between", alignItems: "flex-start", gap: 12 } as const;
const dialogTitle = { margin: 0, fontSize: 22, lineHeight: 1.3, overflowWrap: "anywhere" } as const;
const markdownPanel = { maxWidth: "68ch", margin: "18px auto", padding: "16px 18px", border: "1px solid var(--border-secondary)", borderRadius: 9, background: "var(--bg-tertiary)", lineHeight: 1.65 } as const;
const metadataPanel = { marginTop: 16, padding: 13, border: "1px solid var(--border-secondary)", borderRadius: 9, background: "var(--bg-tertiary)" } as const;
const summary = { cursor: "pointer", color: "var(--text-secondary)", fontWeight: 650 } as const;
const metadataGrid = { display: "grid", gridTemplateColumns: "minmax(120px, auto) minmax(0, 1fr)", gap: "8px 14px", margin: "14px 0", color: "var(--text-tertiary)", fontSize: "var(--text-sm)", overflowWrap: "anywhere" } as const;
const secondaryButton = { border: "1px solid var(--border-secondary)", borderRadius: 7, background: "var(--bg-secondary)", color: "var(--text-secondary)", padding: "7px 11px", cursor: "pointer" } as const;
const errorStyle = { padding: 10, margin: "10px 0", borderRadius: 7, background: "rgba(224,88,74,.10)", color: "#e9857b", fontSize: "var(--text-sm)" } as const;
