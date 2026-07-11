import { useCallback, useEffect, useMemo, useState } from "react";
import { useWorkspace } from "../../hooks/useWorkspace";
import { probeHarnesses, updateHarness } from "../../lib/commands";
import type { AgentHost, HarnessStatus, HarnessUpdateResult } from "../../types";

const CODEX_BIN_SETTING = "lmbrain.codexBin";

export function HarnessesView() {
  const { state } = useWorkspace();
  const [statuses, setStatuses] = useState<HarnessStatus[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [confirming, setConfirming] = useState<HarnessStatus | null>(null);
  const [updating, setUpdating] = useState<AgentHost | null>(null);
  const [results, setResults] = useState<Partial<Record<AgentHost, HarnessUpdateResult>>>({});
  const [updateErrors, setUpdateErrors] = useState<Partial<Record<AgentHost, string>>>({});
  const [copiedHost, setCopiedHost] = useState<AgentHost | null>(null);
  const codexBin = localStorage.getItem(CODEX_BIN_SETTING)?.trim() || undefined;

  const loadStatuses = useCallback(async () => {
    setLoading(true);
    setLoadError(null);
    try {
      setStatuses(await probeHarnesses(codexBin));
    } catch (error) {
      setLoadError(errorMessage(error, "Unable to inspect local harnesses."));
    } finally {
      setLoading(false);
    }
  }, [codexBin]);

  useEffect(() => {
    void loadStatuses();
  }, [loadStatuses]);

  const runningByHost = useMemo(() => {
    const running: Partial<Record<AgentHost, string[]>> = {};
    for (const session of state.sessions) {
      if (session.status !== "running") continue;
      (running[session.host] ??= []).push(session.label);
    }
    return running;
  }, [state.sessions]);

  const runUpdate = async (status: HarnessStatus) => {
    setConfirming(null);
    setUpdating(status.host);
    setUpdateErrors((current) => ({ ...current, [status.host]: undefined }));
    try {
      const result = await updateHarness({
        host: status.host,
        codex_bin: status.host === "codex" ? codexBin : undefined,
      });
      setResults((current) => ({ ...current, [status.host]: result }));
      setStatuses((current) => current.map((item) => item.host === status.host ? result.after : item));
    } catch (error) {
      setUpdateErrors((current) => ({
        ...current,
        [status.host]: errorMessage(error, `Unable to update ${status.label}.`),
      }));
    } finally {
      setUpdating(null);
    }
  };

  const copyInstallCommand = async (status: HarnessStatus) => {
    try {
      await navigator.clipboard.writeText(status.install_command);
      setCopiedHost(status.host);
      setTimeout(() => setCopiedHost(null), 1800);
    } catch {
      setUpdateErrors((current) => ({ ...current, [status.host]: "Could not copy the install command." }));
    }
  };

  return (
    <div style={{ overflowY: "auto", height: "100%" }}>
      <div style={{ maxWidth: 1040, margin: "0 auto", padding: "24px 36px 70px" }}>
        <div style={{ display: "flex", alignItems: "start", justifyContent: "space-between", gap: 20, marginBottom: 22 }}>
          <div>
            <h1 style={{ fontSize: 24, fontWeight: 800, letterSpacing: "-.025em", margin: "0 0 5px" }}>
              Local Harnesses
            </h1>
            <p style={{ fontSize: 13, color: "var(--text-tertiary)", lineHeight: 1.5, margin: 0, maxWidth: 690 }}>
              User-level agent CLIs available to LMBrain. Updates are manual, use each harness&apos;s supported self-updater, and never modify project files.
            </p>
          </div>
          <button type="button" onClick={() => void loadStatuses()} disabled={loading || updating !== null} style={secondaryButtonStyle}>
            <i className={`material-symbols-outlined${loading ? " lmbrain-loading-spinner" : ""}`} aria-hidden="true" style={{ fontSize: 15 }}>
              refresh
            </i>
            Reprobe
          </button>
        </div>

        <div role="note" style={{ display: "flex", gap: 9, padding: "10px 12px", marginBottom: 16, border: "1px solid rgba(91,141,239,.2)", borderRadius: 8, background: "rgba(91,141,239,.07)", color: "var(--text-secondary)", fontSize: 11.5, lineHeight: 1.5 }}>
          <i className="material-symbols-outlined" aria-hidden="true" style={{ fontSize: 16, color: "#72a1ff" }}>shield</i>
          LMBrain never updates these tools automatically. A confirmed update may access the network and change software in your user profile. Matching running sessions must be closed first.
        </div>

        {loadError && <div role="alert" style={errorBannerStyle}>{loadError}</div>}

        {loading && statuses.length === 0 ? (
          <div role="status" style={{ padding: 34, textAlign: "center", color: "var(--text-tertiary)" }}>
            Inspecting local harnesses...
          </div>
        ) : (
          <div style={{ display: "grid", gridTemplateColumns: "repeat(2, minmax(0, 1fr))", gap: 14 }}>
            {statuses.map((status) => (
              <HarnessCard
                key={status.host}
                status={status}
                runningSessions={runningByHost[status.host] ?? []}
                updating={updating === status.host}
                anotherUpdateRunning={updating !== null && updating !== status.host}
                result={results[status.host]}
                error={updateErrors[status.host]}
                copied={copiedHost === status.host}
                onUpdate={() => setConfirming(status)}
                onCopyInstall={() => void copyInstallCommand(status)}
              />
            ))}
          </div>
        )}
      </div>

      {confirming && (
        <UpdateConfirmation
          status={confirming}
          onCancel={() => setConfirming(null)}
          onConfirm={() => void runUpdate(confirming)}
        />
      )}
    </div>
  );
}

function HarnessCard({
  status,
  runningSessions,
  updating,
  anotherUpdateRunning,
  result,
  error,
  copied,
  onUpdate,
  onCopyInstall,
}: {
  status: HarnessStatus;
  runningSessions: string[];
  updating: boolean;
  anotherUpdateRunning: boolean;
  result?: HarnessUpdateResult;
  error?: string;
  copied: boolean;
  onUpdate: () => void;
  onCopyInstall: () => void;
}) {
  const presentation = statusPresentation(status.state);
  const blocked = runningSessions.length > 0;
  return (
    <section style={{ display: "flex", flexDirection: "column", minHeight: 310, padding: 16, borderRadius: 10, border: "1px solid var(--border-secondary)", borderTop: `2px solid ${presentation.color}`, background: "var(--bg-tertiary)" }}>
      <div style={{ display: "flex", alignItems: "start", justifyContent: "space-between", gap: 12 }}>
        <div>
          <div style={{ fontSize: 16, fontWeight: 800, color: "var(--text-primary)" }}>{status.label}</div>
          <div style={{ display: "flex", alignItems: "center", gap: 6, marginTop: 5, fontSize: 11.5, color: presentation.color }}>
            <span aria-hidden="true" style={{ width: 7, height: 7, borderRadius: "50%", background: presentation.color }} />
            {presentation.label}
          </div>
        </div>
        {status.version && <span style={{ fontFamily: "var(--font-mono)", fontSize: 12, color: "#c9bdf9" }}>{status.version}</span>}
      </div>

      <div style={{ marginTop: 16, flex: 1 }}>
        <Detail label="Executable" value={status.executable ?? "Not found"} mono />
        <Detail label="Last checked" value={formatProbeTime(status.probed_at)} />
        {status.detail && <div role={status.state === "error" ? "alert" : undefined} style={{ marginTop: 10, fontSize: 11, lineHeight: 1.45, color: status.state === "error" ? "#e9857b" : "var(--text-tertiary)" }}>{status.detail}</div>}
        {blocked && (
          <div role="alert" style={{ marginTop: 10, padding: "7px 8px", borderRadius: 6, background: "rgba(224,162,58,.08)", color: "#d9b86d", fontSize: 10.5, lineHeight: 1.4 }}>
            Close running session{runningSessions.length === 1 ? "" : "s"}: {runningSessions.join(", ")}
          </div>
        )}
        {error && <div role="alert" style={{ marginTop: 10, color: "#e9857b", fontSize: 11, lineHeight: 1.45 }}>{error}</div>}
        {result && <UpdateResultDetails result={result} />}
      </div>

      {status.state === "installed" ? (
        <button type="button" disabled={blocked || updating || anotherUpdateRunning} onClick={onUpdate} style={{ ...primaryButtonStyle, opacity: blocked || anotherUpdateRunning ? 0.55 : 1 }}>
          <i className={`material-symbols-outlined${updating ? " lmbrain-loading-spinner" : ""}`} aria-hidden="true" style={{ fontSize: 15 }}>
            {updating ? "progress_activity" : "system_update_alt"}
          </i>
          {updating ? "Updating..." : blocked ? "Close sessions first" : "Check & update"}
        </button>
      ) : (
        <div style={{ display: "flex", gap: 7 }}>
          <button type="button" onClick={onCopyInstall} style={{ ...secondaryButtonStyle, flex: 1 }}>
            <i className="material-symbols-outlined" aria-hidden="true" style={{ fontSize: 14 }}>{copied ? "check" : "content_copy"}</i>
            {copied ? "Copied" : "Copy install command"}
          </button>
          <a href={status.install_url} target="_blank" rel="noreferrer" aria-label={`Open ${status.label} installation guide`} style={{ ...secondaryButtonStyle, textDecoration: "none", padding: "7px 9px" }}>
            <i className="material-symbols-outlined" aria-hidden="true" style={{ fontSize: 15 }}>open_in_new</i>
          </a>
        </div>
      )}
    </section>
  );
}

function UpdateResultDetails({ result }: { result: HarnessUpdateResult }) {
  const label = result.success
    ? result.already_current ? "Already up to date" : "Update verified"
    : result.timed_out ? "Update timed out" : "Update failed";
  const color = result.success ? "#70c99a" : "#e9857b";
  return (
    <details style={{ marginTop: 11, border: "1px solid rgba(255,255,255,.07)", borderRadius: 6, background: "rgba(255,255,255,.025)" }}>
      <summary style={{ cursor: "pointer", padding: "7px 8px", color, fontSize: 10.5, fontWeight: 700 }}>{label}</summary>
      <div style={{ padding: "0 8px 8px", fontSize: 10.5, color: "var(--text-tertiary)", lineHeight: 1.45 }}>
        <div>{result.before.version ?? "unknown"} → {result.after.version ?? "unknown"}</div>
        {(result.stdout || result.stderr) && (
          <pre style={{ margin: "7px 0 0", maxHeight: 140, overflow: "auto", whiteSpace: "pre-wrap", overflowWrap: "anywhere", color: "var(--text-secondary)", fontFamily: "var(--font-mono)", fontSize: 9.5 }}>
            {[result.stdout, result.stderr].filter(Boolean).join("\n")}
          </pre>
        )}
      </div>
    </details>
  );
}

function UpdateConfirmation({ status, onCancel, onConfirm }: { status: HarnessStatus; onCancel: () => void; onConfirm: () => void }) {
  return (
    <div role="presentation" style={{ position: "fixed", inset: 0, zIndex: 12000, display: "flex", alignItems: "center", justifyContent: "center", padding: 24, background: "rgba(4,3,6,.72)", backdropFilter: "blur(5px)" }}>
      <div role="dialog" aria-modal="true" aria-labelledby="harness-update-title" style={{ width: "min(500px, 100%)", padding: 20, borderRadius: 12, border: "1px solid #332d3e", background: "#15111b", boxShadow: "0 20px 70px rgba(0,0,0,.5)" }}>
        <h2 id="harness-update-title" style={{ margin: "0 0 9px", fontSize: 18 }}>Update {status.label}?</h2>
        <p style={{ margin: "0 0 12px", color: "var(--text-secondary)", fontSize: 12.5, lineHeight: 1.55 }}>
          This runs the harness&apos;s own self-updater against <span style={{ fontFamily: "var(--font-mono)" }}>{status.executable}</span>. It may access the network and replace software in your user profile.
        </p>
        <div style={{ padding: "9px 10px", borderRadius: 7, background: "rgba(224,162,58,.08)", color: "#d9b86d", fontSize: 11.5, lineHeight: 1.5 }}>
          LMBrain will not elevate privileges, run arbitrary package-manager commands, or modify the workspace. Do not close the app until the updater finishes.
        </div>
        <div style={{ display: "flex", justifyContent: "flex-end", gap: 8, marginTop: 17 }}>
          <button type="button" onClick={onCancel} style={secondaryButtonStyle}>Cancel</button>
          <button type="button" onClick={onConfirm} style={primaryButtonStyle}>Confirm update</button>
        </div>
      </div>
    </div>
  );
}

function Detail({ label, value, mono }: { label: string; value: string; mono?: boolean }) {
  return (
    <div style={{ marginBottom: 9 }}>
      <div style={{ fontSize: 9.5, textTransform: "uppercase", letterSpacing: ".07em", color: "var(--text-muted)", marginBottom: 3 }}>{label}</div>
      <div title={value} style={{ fontSize: 10.5, color: "var(--text-secondary)", fontFamily: mono ? "var(--font-mono)" : undefined, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{value}</div>
    </div>
  );
}

function statusPresentation(state: HarnessStatus["state"]) {
  if (state === "installed") return { label: "Installed", color: "#46b07d" };
  if (state === "missing") return { label: "Not installed", color: "#8f8896" };
  return { label: "Probe error", color: "#e0584a" };
}

function formatProbeTime(value: string) {
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
}

function errorMessage(error: unknown, fallback: string) {
  if (typeof error === "string" && error.trim()) return error;
  if (error instanceof Error && error.message) return error.message;
  return fallback;
}

const primaryButtonStyle = {
  display: "inline-flex",
  alignItems: "center",
  justifyContent: "center",
  gap: 6,
  border: "none",
  borderRadius: 7,
  background: "linear-gradient(135deg,#806cf6,#557ff2)",
  color: "#fff",
  padding: "7px 11px",
  fontSize: 11.5,
  fontWeight: 700,
  cursor: "pointer",
} as const;

const secondaryButtonStyle = {
  display: "inline-flex",
  alignItems: "center",
  justifyContent: "center",
  gap: 6,
  border: "1px solid #302a39",
  borderRadius: 7,
  background: "#19151f",
  color: "var(--text-secondary)",
  padding: "7px 11px",
  fontSize: 11,
  fontWeight: 650,
  cursor: "pointer",
} as const;

const errorBannerStyle = {
  marginBottom: 14,
  padding: "10px 12px",
  border: "1px solid rgba(224,88,74,.25)",
  borderRadius: 8,
  background: "rgba(224,88,74,.08)",
  color: "#e9857b",
  fontSize: 12,
} as const;
