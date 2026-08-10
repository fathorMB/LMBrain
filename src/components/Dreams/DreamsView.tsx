import { useMemo, useState } from "react";
import { getDreams } from "../../lib/commands";
import { useWorkspace } from "../../hooks/useWorkspace";
import { RefreshButton } from "../RefreshButton";
import { PageHeader, PageShell } from "../Shared/PageLayout";

export function DreamsView() {
  const { state, dispatch } = useWorkspace();
  const [status, setStatus] = useState("all"); const [classification, setClassification] = useState("all");
  const [area, setArea] = useState("all"); const [confidence, setConfidence] = useState("all");
  const [loading, setLoading] = useState(false); const [error, setError] = useState<string | null>(null);
  const dreams = useMemo(() => state.dreams.filter((dream) => (status === "all" || dream.status === status) && (classification === "all" || dream.classification === classification) && (area === "all" || dream.area === area) && (confidence === "all" || dream.confidence === confidence)), [state.dreams, status, classification, area, confidence]);
  const refresh = async () => { setLoading(true); setError(null); try { dispatch({ type: "MERGE_DATA", data: { dreams: await getDreams() } }); } catch (reason) { setError(reason instanceof Error ? reason.message : "Dream Journal could not be refreshed."); } finally { setLoading(false); } };
  const options = (values: string[]) => [...new Set(values.filter(Boolean))].sort();
  return <PageShell archetype="dense">
    <PageHeader title="Dream Journal" description="Tentative, grounded technical and design debt captured during explicitly invited Project Lead dreaming sessions." actions={<RefreshButton onClick={refresh} loading={loading} />} />
    <p style={{ color: "var(--text-tertiary)", fontSize: "var(--text-sm)", margin: "0 0 16px" }}>Read-only. Dreams never enter delivery automatically; promotion must be an explicit governed action.</p>
    {error && <p role="alert" style={{ color: "#f87171" }}>{error}</p>}
    <div aria-label="Dream filters" style={{ display: "flex", flexWrap: "wrap", gap: 8, marginBottom: 16 }}>
      <Filter label="State" value={status} onChange={setStatus} options={options(state.dreams.map((d) => d.status))} />
      <Filter label="Kind" value={classification} onChange={setClassification} options={options(state.dreams.map((d) => d.classification))} />
      <Filter label="Area" value={area} onChange={setArea} options={options(state.dreams.map((d) => d.area ?? ""))} />
      <Filter label="Confidence" value={confidence} onChange={setConfidence} options={options(state.dreams.map((d) => d.confidence))} />
    </div>
    {loading && <p role="status">Loading Dream Journal…</p>}
    {!loading && state.dreams.length === 0 && <div style={empty}>No dreams captured yet. An explicitly invited Project Lead dreaming session may produce zero or more grounded records.</div>}
    {!loading && state.dreams.length > 0 && dreams.length === 0 && <div style={empty}>No dreams match these filters.</div>}
    <div style={{ display: "grid", gap: 10 }}>{dreams.map((dream) => <article key={dream.path} aria-label={`${dream.id}: ${dream.title}`} style={{ border: "1px solid var(--border-primary)", background: "var(--bg-secondary)", borderRadius: 10, padding: 14 }}>
      <div style={{ display: "flex", justifyContent: "space-between", gap: 12 }}><strong>{dream.id} · {dream.title}</strong><span>{dream.malformed ? "malformed" : dream.status}</span></div>
      <p style={{ color: "var(--text-secondary)", margin: "8px 0" }}>{dream.classification} · {dream.confidence}{dream.area ? ` · ${dream.area}` : ""}</p>
      <p style={{ margin: "0 0 8px", whiteSpace: "pre-wrap" }}>{dream.body || "No readable rationale."}</p>
      <details><summary>Provenance and next disposition</summary><p>Context digest: <code>{dream.context_digest || "unavailable"}</code></p><p>References: {dream.related_artifacts.join(", ") || "none"}</p><button type="button" onClick={() => void navigator.clipboard?.writeText(`Review ${dream.path} and explicitly choose whether to triage, promote, or discard ${dream.id}.`)}>Copy governed action prompt</button></details>
    </article>)}</div>
  </PageShell>;
}
function Filter({ label, value, onChange, options }: { label: string; value: string; onChange: (value: string) => void; options: string[] }) { return <label>{label} <select value={value} onChange={(event) => onChange(event.target.value)}><option value="all">All</option>{options.map((option) => <option key={option} value={option}>{option}</option>)}</select></label>; }
const empty = { border: "1px dashed var(--border-primary)", borderRadius: 10, padding: 20, color: "var(--text-tertiary)" } as const;
