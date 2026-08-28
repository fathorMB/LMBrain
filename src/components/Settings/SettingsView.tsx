import { getVersion } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { HarnessesView } from "../Harnesses/HarnessesView";
import { useWorkspace } from "../../hooks/useWorkspace";

type SettingsTab = "general" | "harnesses" | "about";
const tabs: Array<{ id: SettingsTab; label: string }> = [
  { id: "general", label: "General" }, { id: "harnesses", label: "Harnesses" },
  { id: "about", label: "About" },
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
    <header style={{ padding: "var(--page-top) var(--page-gutter) 0", borderBottom: "1px solid var(--border-primary)" }}>
      <h1 style={{ margin: "0 0 16px", fontSize: "var(--text-2xl)" }}>Settings</h1>
      <div role="tablist" aria-label="Settings sections" style={{ display: "flex", gap: 4, overflowX: "auto" }}>
        {tabs.map((item) => <button key={item.id} id={`settings-tab-${item.id}`} role="tab" aria-selected={tab === item.id} aria-controls={`settings-panel-${item.id}`} tabIndex={tab === item.id ? 0 : -1} onClick={() => select(item.id)} style={tabStyle(tab === item.id)}>{item.label}</button>)}
      </div>
    </header>
    <section role="tabpanel" id={`settings-panel-${tab}`} aria-labelledby={`settings-tab-${tab}`} style={{ minHeight: 0 }}>
      {tab === "general" && <GeneralPanel />}
      {tab === "harnesses" && <HarnessesView />}
      {tab === "about" && <AboutPanel />}
    </section>
  </div>;
}

function GeneralPanel() {
  const [enabled, setEnabled] = useState(false);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => { let cancelled = false; invoke<{ enabled: boolean }>("get_claude_eli5_preference").then((preference) => { if (!cancelled) setEnabled(preference.enabled); }).catch((reason) => { if (!cancelled) setError(String(reason)); }).finally(() => { if (!cancelled) setLoading(false); }); return () => { cancelled = true; }; }, []);
  const change = async (next: boolean) => { setError(null); setLoading(true); try { const preference = await invoke<{ enabled: boolean }>("set_claude_eli5_preference", { enabled: next }); setEnabled(preference.enabled); } catch (reason) { setError(String(reason)); } finally { setLoading(false); } };
  return <Panel><h2>General</h2><Card><h3 style={{ marginTop: 0 }}>Claude Code ELI5 output</h3><p style={muted}>Use LMBrain’s local ELI5 communication style for new Claude Code sessions in this workspace. It is stored only on this machine, is off by default, and does not change shared project settings or other harnesses.</p><label style={{ display: "flex", gap: 10, alignItems: "center", cursor: loading ? "wait" : "pointer" }}><input type="checkbox" checked={enabled} disabled={loading} onChange={(event) => void change(event.target.checked)} /> Enable ELI5 for Claude Code</label><p style={{ ...muted, marginBottom: 0 }}>Enabling installs or verifies a user-level style and merges only <code>outputStyle: ELI5</code> into <code>.claude/settings.local.json</code>. New Claude sessions pick it up; disabling removes only unchanged LMBrain-managed entries.</p>{error && <p role="alert" style={{ color: "var(--danger-primary, #d9534f)", marginBottom: 0 }}>{error}</p>}</Card><p style={muted}>Machine-local harness selection lives under Harnesses. The governed project environment and verification manifest are consultable read-only from the Environment page in the sidebar; the Project Lead manages them through the LMBrain MCP verbs.</p></Panel>;
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

function Panel({ children, wide }: { children: React.ReactNode; wide?: boolean }) { return <div style={{ maxWidth: wide ? 980 : 720, margin: "0 auto", padding: "24px 30px 70px" }}>{children}</div>; }
function Card({ children }: { children: React.ReactNode }) { return <div style={{ marginBottom: 12, padding: 15, border: "1px solid var(--border-secondary)", borderRadius: 10, background: "var(--bg-tertiary)" }}>{children}</div>; }
function Info({ label, value }: { label: string; value: string }) { return <Card><div style={{ fontSize: "var(--text-xs)", color: "var(--text-tertiary)" }}>{label}</div><div style={{ marginTop: 4, fontFamily: "var(--font-mono)" }}>{value}</div></Card>; }
function tabFromHash(): SettingsTab { const candidate = window.location.hash.replace(/^#settings\//, "") as SettingsTab; return tabs.some((tab) => tab.id === candidate) ? candidate : "general"; }
function tabStyle(active: boolean): React.CSSProperties { return { border: 0, borderBottom: `2px solid ${active ? "var(--accent-primary)" : "transparent"}`, background: "transparent", color: active ? "var(--text-primary)" : "var(--text-tertiary)", padding: "9px 12px", cursor: "pointer", fontWeight: active ? 700 : 500 }; }
const muted: React.CSSProperties = { color: "var(--text-tertiary)", fontSize: "var(--text-sm)", lineHeight: 1.55 };
