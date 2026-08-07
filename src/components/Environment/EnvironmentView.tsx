import { useCallback, useEffect, useState } from "react";
import {
  getHarnessApprovalStatus,
  getHarnessDrift,
  getVerificationManifestStatus,
  planHarnessConfiguration,
} from "../../lib/commands";
import type {
  HarnessApprovalStatus,
  HarnessConfigurationPlan,
  HarnessDriftEntry,
  VerificationManifestStatus,
} from "../../types";

/**
 * Read-only consultation of the governed project environment (#87): the
 * harness manifest approval state, the deterministic native-file plan, drift,
 * and the verification manifest status. Every mutation — approve, revoke,
 * apply, manifest creation and rollback — is a Project Lead action performed
 * through the `lmbrain` MCP verbs and is intentionally unavailable here.
 */
export function EnvironmentView() {
  const [approval, setApproval] = useState<HarnessApprovalStatus | null>(null);
  const [plan, setPlan] = useState<HarnessConfigurationPlan | null>(null);
  const [drift, setDrift] = useState<HarnessDriftEntry[]>([]);
  const [verification, setVerification] = useState<VerificationManifestStatus | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setBusy(true);
    setError(null);
    try {
      const status = await getHarnessApprovalStatus();
      setApproval(status);
      if (status.state === "unconfigured") {
        setPlan(null);
        setDrift([]);
      } else {
        setPlan(await planHarnessConfiguration());
        setDrift(await getHarnessDrift());
      }
    } catch (reason) {
      setError(message(reason));
    }
    try {
      setVerification(await getVerificationManifestStatus());
    } catch (reason) {
      setError((previous) => previous ?? message(reason));
    } finally {
      setBusy(false);
    }
  }, []);

  useEffect(() => {
    const timer = window.setTimeout(() => void refresh(), 0);
    return () => window.clearTimeout(timer);
  }, [refresh]);

  return <div style={{ height: "100%", overflow: "auto" }}>
    <div style={{ maxWidth: 980, margin: "0 auto", padding: "var(--page-top) var(--page-gutter) 70px" }}>
      <div style={{ display: "flex", alignItems: "flex-start", justifyContent: "space-between", gap: 12 }}>
        <div>
          <h1 style={{ margin: "0 0 6px", fontSize: "var(--text-2xl)" }}>Environment</h1>
          <p style={muted}>
            Read-only status of the governed project environment. The Project Lead manages
            approval, materialization, and the verification manifest through the LMBrain MCP
            verbs (<code>harness_manifest_approve</code>, <code>harness_config_apply</code>,{" "}
            <code>verification_manifest_approve</code>, …); this page never mutates anything.
          </p>
        </div>
        <button onClick={() => void refresh()} disabled={busy} style={secondary}>Refresh</button>
      </div>

      {error && <div role="alert" style={errorStyle}>{error}</div>}
      {busy && !approval && !verification && <p role="status" style={muted}>Loading environment status…</p>}

      <h2 style={{ marginTop: 22 }}>Project environment</h2>
      {approval?.state === "unconfigured" && <Card>
        <h3 style={{ marginTop: 0 }}>No harness manifest</h3>
        <p style={muted}>
          This project has not opted into governed harness configuration. The Project Lead can
          create it with <code>harness_config_validate</code> and <code>harness_config_set</code>.
        </p>
      </Card>}
      {approval && approval.state !== "unconfigured" && <>
        <Card>
          <strong>Approval: {approval.state}</strong>
          <div style={mono}>{approval.manifest_digest}</div>
          <dl style={details}>
            <Info label="Approved digest" value={approval.approved_digest ?? "None"} />
            <Info label="Approved at" value={approval.approved_at ?? "Not approved"} />
            <Info label="Approved by" value={approval.approved_by ?? "—"} />
          </dl>
        </Card>
        {plan?.has_conflicts && <div role="alert" style={errorStyle}>
          Native configuration conflicts exist; the Lead must resolve them before approval or apply.
        </div>}
        {drift.length > 0 && <div role="alert" style={warningStyle}>
          Drift detected in {drift.map((entry) => entry.path).join(", ")}.
        </div>}
        <div style={{ display: "grid", gap: 10 }}>
          {plan?.hosts.map((host) => <Card key={host.host}>
            <div style={{ display: "flex", justifyContent: "space-between" }}>
              <strong>{host.host}</strong>
              <span style={{ color: host.ready ? "#70c99a" : "#e0a23a" }}>
                {host.ready ? "Ready" : "Needs attention"}
              </span>
            </div>
            <div style={{ marginTop: 8, fontSize: "var(--text-sm)", color: "var(--text-secondary)" }}>
              Capabilities: {host.supported_capabilities.join(", ")}
            </div>
            {host.lsp && <div style={{ fontSize: "var(--text-xs)" }}>
              LSP: {host.lsp.state} · prerequisite {host.lsp.prerequisite_ready ? "ready" : "missing"}
            </div>}
            {host.browser_mcp && <div style={{
              fontSize: "var(--text-xs)",
              color: host.browser_mcp.state === "prerequisite-ready" ? "#70c99a" : "#e0a23a",
            }}>
              Browser MCP ({host.browser_mcp.provider}): {host.browser_mcp.state} · {host.browser_mcp.detail}
            </div>}
            {host.tools.map((tool) => <div
              key={tool.tool}
              style={{ fontSize: "var(--text-xs)", color: tool.available ? "#70c99a" : "#e0a23a" }}
            >
              {tool.tool}: {tool.available ? "available" : "missing"}
            </div>)}
            {host.native_files.map((file) => <div
              key={file.path}
              style={{ marginTop: 8, padding: 8, borderRadius: 7, background: "rgba(255,255,255,.03)", fontSize: "var(--text-xs)" }}
            >
              <strong>{file.action}</strong> <span style={mono}>{file.path}</span>
              <div style={muted}>{file.detail}</div>
              <div style={mono}>Owned: {file.owned_paths.join(", ")}</div>
            </div>)}
          </Card>)}
        </div>
      </>}

      <h2 style={{ marginTop: 26 }}>Verification</h2>
      {verification && <Card>
        <strong>Manifest: {verification.state}</strong>
        <div style={mono}>{verification.manifest_digest ?? "No current digest"}</div>
        <p style={muted}>{verification.next_action}</p>
        <dl style={details}>
          <Info label="Gates" value={String(verification.gate_count)} />
          <Info label="Approved digest" value={verification.approved_digest ?? "None"} />
          <Info label="Approved at" value={verification.approved_at ?? "Not approved"} />
        </dl>
        {verification.issues.map((issue) => <div key={issue} style={warningStyle}>{issue}</div>)}
        <div style={boundary}>
          The Project Lead manages this manifest through{" "}
          <code>verification_manifest_init</code>, <code>verification_manifest_set</code>,{" "}
          <code>verification_manifest_approve</code>, and{" "}
          <code>verification_manifest_rollback</code>. <code>spec_verify</code> executes only
          gates referenced by the approved manifest.
        </div>
      </Card>}
      {!verification && !busy && <p style={muted}>Verification status unavailable.</p>}
    </div>
  </div>;
}

function Card({ children }: { children: React.ReactNode }) {
  return <div style={{ marginBottom: 12, padding: 15, border: "1px solid var(--border-secondary)", borderRadius: 10, background: "var(--bg-tertiary)" }}>{children}</div>;
}

function Info({ label, value }: { label: string; value: string }) {
  return <div><dt style={{ color: "var(--text-tertiary)", fontSize: "var(--text-xs)" }}>{label}</dt><dd style={{ margin: "2px 0 0", overflowWrap: "anywhere" }}>{value}</dd></div>;
}

function message(value: unknown) {
  return value instanceof Error ? value.message : String(value);
}

const muted: React.CSSProperties = { color: "var(--text-tertiary)", fontSize: "var(--text-sm)", lineHeight: 1.55 };
const mono: React.CSSProperties = { color: "var(--text-tertiary)", fontFamily: "var(--font-mono)", fontSize: "var(--text-xs)", overflowWrap: "anywhere" };
const details: React.CSSProperties = { display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(180px, 1fr))", gap: 10, margin: "12px 0" };
const secondary: React.CSSProperties = { border: "1px solid var(--border-secondary)", borderRadius: 7, background: "var(--bg-secondary)", color: "var(--text-secondary)", padding: "7px 11px", cursor: "pointer", flex: "none" };
const errorStyle: React.CSSProperties = { padding: 10, marginBottom: 12, borderRadius: 7, background: "rgba(224,88,74,.10)", color: "#e9857b", fontSize: "var(--text-sm)" };
const warningStyle: React.CSSProperties = { ...errorStyle, background: "rgba(224,162,58,.10)", color: "#d9b86d" };
const boundary: React.CSSProperties = { ...muted, padding: 10, borderLeft: "3px solid var(--accent-primary)", background: "rgba(80,120,220,.08)" };
