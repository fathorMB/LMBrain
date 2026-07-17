import { getVersion } from "@tauri-apps/api/app";
import { useCallback, useEffect, useState } from "react";
import { HarnessesView } from "../Harnesses/HarnessesView";
import { useWorkspace } from "../../hooks/useWorkspace";
import {
  applyHarnessConfiguration,
  approveHarnessManifest,
  getHarnessApprovalStatus,
  getHarnessDrift,
  planHarnessConfiguration,
  revokeHarnessManifestApproval,
} from "../../lib/commands";
import type { HarnessApprovalStatus, HarnessConfigurationPlan, HarnessDriftEntry } from "../../types";

type SettingsTab = "general" | "harnesses" | "project-environment" | "about";
const tabs: Array<{ id: SettingsTab; label: string }> = [
  { id: "general", label: "General" }, { id: "harnesses", label: "Harnesses" },
  { id: "project-environment", label: "Project environment" }, { id: "about", label: "About" },
];

export function SettingsView({ initialTab }: { initialTab?: SettingsTab }) {
  const [tab, setTab] = useState<SettingsTab>(() => initialTab ?? tabFromHash());
  useEffect(() => {
    if (initialTab) window.history.replaceState(null, "", `${window.location.pathname}${window.location.search}#settings/${initialTab}`);
    const sync = () => setTab(tabFromHash());
    window.addEventListener("hashchange", sync);
    return () => window.removeEventListener("hashchange", sync);
  }, [initialTab]);
  const select = (next: SettingsTab) => {
    setTab(next);
    window.history.replaceState(null, "", `${window.location.pathname}${window.location.search}#settings/${next}`);
  };
  return <div style={{ height: "100%", overflow: "auto" }}>
    <header style={{ padding: "22px 30px 0", borderBottom: "1px solid var(--border-primary)" }}>
      <h1 style={{ margin: "0 0 16px", fontSize: 24 }}>Settings</h1>
      <div role="tablist" aria-label="Settings sections" style={{ display: "flex", gap: 4, overflowX: "auto" }}>
        {tabs.map((item) => <button key={item.id} id={`settings-tab-${item.id}`} role="tab" aria-selected={tab === item.id} aria-controls={`settings-panel-${item.id}`} tabIndex={tab === item.id ? 0 : -1} onClick={() => select(item.id)} style={tabStyle(tab === item.id)}>{item.label}</button>)}
      </div>
    </header>
    <section role="tabpanel" id={`settings-panel-${tab}`} aria-labelledby={`settings-tab-${tab}`} style={{ minHeight: 0 }}>
      {tab === "general" && <GeneralPanel />}
      {tab === "harnesses" && <HarnessesView />}
      {tab === "project-environment" && <ProjectEnvironmentPanel />}
      {tab === "about" && <AboutPanel />}
    </section>
  </div>;
}

function GeneralPanel() {
  return <Panel><h2>General</h2><p style={muted}>There are currently no application-wide preferences. Machine-local harness selection lives under Harnesses; project intent and approval live under Project environment.</p></Panel>;
}

function AboutPanel() {
  const { state } = useWorkspace();
  const workspace = state.currentWorkspace;
  // The product version comes from the build metadata chain (package.json ->
  // tauri.conf.json -> binary); component versions such as the MCP crate are
  // deliberately not shown as the application version.
  const [appVersion, setAppVersion] = useState<string | null>(null);
  useEffect(() => {
    let cancelled = false;
    getVersion()
      .then((version) => { if (!cancelled) setAppVersion(version); })
      .catch(() => { if (!cancelled) setAppVersion(null); });
    return () => { cancelled = true; };
  }, []);
  return <Panel><h2>About</h2><Info label="Application" value={appVersion ? `LMBrain ${appVersion}` : "Unknown"} /><Info label="Project kit" value={workspace?.project_kit_version ?? workspace?.kit_version ?? "No workspace"} /><Info label="Bundled kit" value={workspace?.bundled_kit_version ?? "Unknown"} /></Panel>;
}

function ProjectEnvironmentPanel() {
  const [approval, setApproval] = useState<HarnessApprovalStatus | null>(null);
  const [plan, setPlan] = useState<HarnessConfigurationPlan | null>(null);
  const [drift, setDrift] = useState<HarnessDriftEntry[]>([]);
  const [busy, setBusy] = useState(false);
  const [promptCopied, setPromptCopied] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const refresh = useCallback(async () => {
    setBusy(true); setError(null);
    try {
      const status = await getHarnessApprovalStatus(); setApproval(status);
      if (status.state === "unconfigured") { setPlan(null); setDrift([]); }
      else { setPlan(await planHarnessConfiguration()); setDrift(await getHarnessDrift()); }
    } catch (reason) { setError(message(reason)); }
    finally { setBusy(false); }
  }, []);
  useEffect(() => {
    const timer = window.setTimeout(() => void refresh(), 0);
    return () => window.clearTimeout(timer);
  }, [refresh]);
  const act = async (operation: () => Promise<unknown>) => { setBusy(true); setError(null); try { await operation(); await refresh(); } catch (reason) { setError(message(reason)); setBusy(false); } };
  const prompt = "Review .lmbrain/HARNESSES.json, then use harness_config_validate and harness_config_set to create or remediate the complete project harness manifest. Do not approve or materialize native configuration.";
  const copyPrompt = async () => {
    try {
      await navigator.clipboard.writeText(prompt);
      setPromptCopied(true);
      window.setTimeout(() => setPromptCopied(false), 1800);
    } catch {
      setError("Could not copy the Project Lead prompt.");
    }
  };
  return <Panel wide>
    <div style={{ display: "flex", alignItems: "flex-start", justifyContent: "space-between", gap: 12 }}><div><h2>Project environment</h2><p style={muted}>Versioned project intent with machine-local approval. Opening a project never approves or applies this configuration.</p></div><button onClick={() => void refresh()} disabled={busy} style={{ ...secondary, flex: "none", alignSelf: "flex-start" }}>Refresh</button></div>
    {error && <div role="alert" style={errorStyle}>{error}</div>}
    {!approval && busy && <p role="status" style={muted}>Loading project environment…</p>}
    {approval?.state === "unconfigured" && <Card><h3>No harness manifest</h3><p style={muted}>This project has not opted into governed harness configuration.</p><button style={secondary} onClick={() => void copyPrompt()} aria-live="polite">{promptCopied ? "Copied" : "Copy Project Lead prompt"}</button>{promptCopied && <span role="status" style={{ ...muted, marginLeft: 10 }}>Prompt copied to clipboard.</span>}</Card>}
    {approval && approval.state !== "unconfigured" && <>
      <Card><div style={{ display: "flex", justifyContent: "space-between", gap: 12 }}><div><strong>Approval: {approval.state}</strong><div style={mono}>{approval.manifest_digest}</div></div><div style={{ display: "flex", gap: 8 }}>
        {approval.state === "approved" ? <button disabled={busy} style={secondary} onClick={() => void act(revokeHarnessManifestApproval)}>Revoke</button> : <button disabled={busy || !approval.manifest_digest || Boolean(plan?.has_conflicts)} style={primary} onClick={() => void act(() => approveHarnessManifest(approval.manifest_digest!))}>Approve current digest</button>}
        <button disabled={busy || approval.state !== "approved" || Boolean(plan?.has_conflicts) || !plan?.hosts.every((host) => host.ready)} style={primary} onClick={() => void act(applyHarnessConfiguration)}>Apply</button>
      </div></div></Card>
      {plan?.has_conflicts && <div role="alert" style={errorStyle}>Resolve native configuration conflicts before approval or application.</div>}
      {drift.length > 0 && <div role="alert" style={warningStyle}>Drift detected in {drift.map((entry) => entry.path).join(", ")}. Review and retry explicitly.</div>}
      <div style={{ display: "grid", gap: 10 }}>{plan?.hosts.map((host) => <Card key={host.host}><div style={{ display: "flex", justifyContent: "space-between" }}><strong>{host.host}</strong><span style={{ color: host.ready ? "#70c99a" : "#e0a23a" }}>{host.ready ? "Ready" : "Needs attention"}</span></div><div style={{ marginTop: 8, fontSize: 12, color: "var(--text-secondary)" }}>Capabilities: {host.supported_capabilities.join(", ")}</div>{host.lsp && <div style={{ fontSize: 11.5 }}>LSP: {host.lsp.state} · prerequisite {host.lsp.prerequisite_ready ? "ready" : "missing"}</div>}{host.tools.map((tool) => <div key={tool.tool} style={{ fontSize: 11.5, color: tool.available ? "#70c99a" : "#e0a23a" }}>{tool.tool}: {tool.available ? "available" : "missing"}</div>)}{host.native_files.map((file) => <div key={file.path} style={{ marginTop: 8, padding: 8, borderRadius: 7, background: "rgba(255,255,255,.03)", fontSize: 11.5 }}><strong>{file.action}</strong> <span style={mono}>{file.path}</span><div style={muted}>{file.detail}</div><div style={mono}>Owned: {file.owned_paths.join(", ")}</div></div>)}</Card>)}</div>
    </>}
  </Panel>;
}

function Panel({ children, wide }: { children: React.ReactNode; wide?: boolean }) { return <div style={{ maxWidth: wide ? 980 : 720, margin: "0 auto", padding: "24px 30px 70px" }}>{children}</div>; }
function Card({ children }: { children: React.ReactNode }) { return <div style={{ marginBottom: 12, padding: 15, border: "1px solid var(--border-secondary)", borderRadius: 10, background: "var(--bg-tertiary)" }}>{children}</div>; }
function Info({ label, value }: { label: string; value: string }) { return <Card><div style={{ fontSize: 11, color: "var(--text-tertiary)" }}>{label}</div><div style={{ marginTop: 4, fontFamily: "var(--font-mono)" }}>{value}</div></Card>; }
function tabFromHash(): SettingsTab { const candidate = window.location.hash.replace(/^#settings\//, "") as SettingsTab; return tabs.some((tab) => tab.id === candidate) ? candidate : "general"; }
function message(value: unknown) { return value instanceof Error ? value.message : String(value); }
function tabStyle(active: boolean): React.CSSProperties { return { border: 0, borderBottom: `2px solid ${active ? "var(--accent-primary)" : "transparent"}`, background: "transparent", color: active ? "var(--text-primary)" : "var(--text-tertiary)", padding: "9px 12px", cursor: "pointer", fontWeight: active ? 700 : 500 }; }
const muted: React.CSSProperties = { color: "var(--text-tertiary)", fontSize: 12.5, lineHeight: 1.55 };
const mono: React.CSSProperties = { color: "var(--text-tertiary)", fontFamily: "var(--font-mono)", fontSize: 10.5, overflowWrap: "anywhere" };
const secondary: React.CSSProperties = { border: "1px solid var(--border-secondary)", borderRadius: 7, background: "var(--bg-secondary)", color: "var(--text-secondary)", padding: "7px 11px", cursor: "pointer" };
const primary: React.CSSProperties = { ...secondary, background: "var(--accent-primary)", color: "white" };
const errorStyle: React.CSSProperties = { padding: 10, marginBottom: 12, borderRadius: 7, background: "rgba(224,88,74,.10)", color: "#e9857b", fontSize: 12 };
const warningStyle: React.CSSProperties = { ...errorStyle, background: "rgba(224,162,58,.10)", color: "#d9b86d" };
