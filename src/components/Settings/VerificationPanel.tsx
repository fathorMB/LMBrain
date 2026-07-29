import { useCallback, useEffect, useState } from "react";
import {
  getVerificationManifestStatus,
  previewVerificationManifest,
  rollbackVerificationManifest,
  setVerificationManifest,
} from "../../lib/commands";
import type {
  VerificationGate,
  VerificationManifestPreview,
  VerificationManifestStatus,
} from "../../types";

export function VerificationPanel() {
  const [status, setStatus] = useState<VerificationManifestStatus | null>(null);
  const [preview, setPreview] = useState<VerificationManifestPreview | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setBusy(true);
    setError(null);
    try {
      setStatus(await getVerificationManifestStatus());
    } catch (reason) {
      setError(message(reason));
    } finally {
      setBusy(false);
    }
  }, []);

  useEffect(() => {
    const timer = window.setTimeout(() => void refresh(), 0);
    return () => window.clearTimeout(timer);
  }, [refresh]);

  const discover = async () => {
    setBusy(true);
    setError(null);
    try {
      const next = await previewVerificationManifest();
      setPreview(next);
      setStatus(next.status);
    } catch (reason) {
      setError(message(reason));
    } finally {
      setBusy(false);
    }
  };

  const write = async () => {
    if (!preview) return;
    setBusy(true);
    setError(null);
    try {
      await setVerificationManifest(
        preview.proposed_manifest,
        preview.status.manifest_digest,
      );
      setPreview(null);
      await refresh();
    } catch (reason) {
      setError(message(reason));
      setBusy(false);
    }
  };

  const rollback = async () => {
    if (!status?.manifest_digest) return;
    setBusy(true);
    setError(null);
    try {
      await rollbackVerificationManifest(status.manifest_digest);
      setPreview(null);
      await refresh();
    } catch (reason) {
      setError(message(reason));
      setBusy(false);
    }
  };

  return <div style={{ maxWidth: 980, margin: "0 auto", padding: "24px 30px 70px" }}>
    <div style={{ display: "flex", justifyContent: "space-between", gap: 12, alignItems: "flex-start" }}>
      <div>
        <h2 style={{ marginTop: 0 }}>Verification</h2>
        <p style={muted}>
          Inspect and configure the project verification manifest. Discovery never executes commands.
        </p>
      </div>
      <button style={secondary} disabled={busy} onClick={() => void refresh()}>Refresh</button>
    </div>

    {error && <div role="alert" style={errorStyle}>{error}</div>}
    {!status && busy && <p role="status" style={muted}>Loading verification status…</p>}
    {status && <section style={card}>
      <div style={{ display: "flex", justifyContent: "space-between", gap: 12, flexWrap: "wrap" }}>
        <div>
          <strong>Manifest: {status.state}</strong>
          <div style={mono}>{status.manifest_digest ?? "No current digest"}</div>
        </div>
        <button style={secondary} disabled={busy} onClick={() => void discover()}>
          Discover / refresh preview
        </button>
      </div>
      <p style={muted}>{status.next_action}</p>
      <dl style={details}>
        <Info label="Gates" value={String(status.gate_count)} />
        <Info label="Approved digest" value={status.approved_digest ?? "None"} />
        <Info label="Approved at" value={status.approved_at ?? "Not approved"} />
      </dl>
      {status.issues.map((issue) => <div key={issue} style={warningStyle}>{issue}</div>)}
      <div style={boundary}>
        Approval is intentionally unavailable in the app. Creating, replacing, or rolling back the
        manifest does not approve it and does not change any artifact status. Approval must be
        performed separately through <code>verification_manifest_approve</code>.
      </div>
      {status.can_rollback && status.manifest_digest && <button
        style={secondary}
        disabled={busy}
        onClick={() => void rollback()}
      >Restore previous manifest</button>}
    </section>}

    {preview && <section aria-label="Verification manifest preview">
      {preview.conflicts.map((conflict) => <div role="alert" key={conflict} style={errorStyle}>{conflict}</div>)}
      {preview.guidance.map((item) => <div key={item} style={warningStyle}>{item}</div>)}
      <h3>Discovered gates</h3>
      <div style={{ display: "grid", gap: 10 }}>
        {preview.candidates.map((candidate) => <GateCard
          key={`${candidate.gate.id}:${candidate.provenance}`}
          gate={candidate.gate}
          provenance={candidate.provenance}
          confidence={candidate.confidence}
          environmentPolicy={candidate.environment_policy}
          mutationPolicy={candidate.mutation_policy}
          securityNotes={candidate.security_notes}
          selected={candidate.selected}
        />)}
      </div>
      {preview.candidates.length === 0 && <p style={muted}>No repository-native test command was discovered.</p>}

      <h3>Proposed manifest</h3>
      <div style={card}>
        <div style={mono}>Digest: {preview.proposed_digest}</div>
        <pre style={codeBlock}>{preview.proposed_toml}</pre>
      </div>
      <details>
        <summary>Diff from current manifest</summary>
        <pre style={codeBlock}>{preview.diff || "No changes"}</pre>
      </details>
      <p style={boundary}>
        This writes configuration only. It will not run a gate, approve the manifest, or change
        lifecycle state.
      </p>
      <button
        style={primary}
        disabled={busy || preview.conflicts.length > 0}
        onClick={() => void write()}
      >
        {preview.status.state === "absent" ? "Create manifest" : "Replace manifest"}
      </button>
    </section>}
  </div>;
}

function GateCard(props: {
  gate: VerificationGate;
  provenance: string;
  confidence: string;
  environmentPolicy: string;
  mutationPolicy: string;
  securityNotes: string[];
  selected: boolean;
}) {
  const gate = props.gate;
  return <article style={card}>
    <div style={{ display: "flex", justifyContent: "space-between", gap: 12 }}>
      <strong>{gate.id}</strong>
      <span style={{ color: props.selected ? "#70c99a" : "#d9b86d", fontSize: 12 }}>
        {props.selected ? "Selected" : "Review required"} · {props.confidence}
      </span>
    </div>
    <div style={mono}>{[gate.program, ...gate.args].join(" ")}</div>
    <dl style={details}>
      <Info label="Working directory" value={gate.cwd} />
      <Info label="Timeout" value={`${gate.timeout_seconds ?? "default"} seconds`} />
      <Info label="Output limit" value={String(gate.output_limit_bytes ?? "default")} />
      <Info label="Expected exit" value={String(gate.expected_exit_code ?? 0)} />
      <Info label="Result matcher" value={gate.result_matcher ?? "None"} />
      <Info label="Provenance" value={props.provenance} />
      <Info label="Environment" value={props.environmentPolicy} />
      <Info label="Mutation policy" value={props.mutationPolicy} />
      <Info label="Fingerprint exclusions" value={gate.fingerprint_exclude?.join(", ") || "None"} />
    </dl>
    {props.securityNotes.map((note) => <div key={note} style={muted}>{note}</div>)}
  </article>;
}

function Info({ label, value }: { label: string; value: string }) {
  return <div><dt style={{ color: "var(--text-tertiary)", fontSize: 11 }}>{label}</dt><dd style={{ margin: "2px 0 0", overflowWrap: "anywhere" }}>{value}</dd></div>;
}

function message(value: unknown) {
  return value instanceof Error ? value.message : String(value);
}

const muted: React.CSSProperties = { color: "var(--text-tertiary)", fontSize: 12.5, lineHeight: 1.55 };
const mono: React.CSSProperties = { color: "var(--text-tertiary)", fontFamily: "var(--font-mono)", fontSize: 10.5, overflowWrap: "anywhere" };
const card: React.CSSProperties = { marginBottom: 12, padding: 15, border: "1px solid var(--border-secondary)", borderRadius: 10, background: "var(--bg-tertiary)" };
const details: React.CSSProperties = { display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(180px, 1fr))", gap: 10, margin: "12px 0" };
const secondary: React.CSSProperties = { border: "1px solid var(--border-secondary)", borderRadius: 7, background: "var(--bg-secondary)", color: "var(--text-secondary)", padding: "7px 11px", cursor: "pointer" };
const primary: React.CSSProperties = { ...secondary, background: "var(--accent-primary)", color: "white" };
const errorStyle: React.CSSProperties = { padding: 10, marginBottom: 12, borderRadius: 7, background: "rgba(224,88,74,.10)", color: "#e9857b", fontSize: 12 };
const warningStyle: React.CSSProperties = { ...errorStyle, background: "rgba(224,162,58,.10)", color: "#d9b86d" };
const boundary: React.CSSProperties = { ...muted, padding: 10, borderLeft: "3px solid var(--accent-primary)", background: "rgba(80,120,220,.08)" };
const codeBlock: React.CSSProperties = { whiteSpace: "pre-wrap", overflowWrap: "anywhere", maxHeight: 360, overflow: "auto", padding: 12, borderRadius: 7, background: "var(--bg-primary)", fontSize: 11 };
