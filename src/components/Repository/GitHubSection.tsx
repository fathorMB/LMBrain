import { useState } from "react";
import * as commands from "../../lib/commands";
import type { GitHubDashboard, GitHubWorkflowRun } from "../../types";
import { describeWorkflowRun, getWorkflowRunStatusStyle } from "../../lib/workflowRunStatus";

export interface GitHubSectionProps {
  githubDashboard: GitHubDashboard | null;
  hasToken: boolean;
  onTokenChanged: () => void;
  onSelectRun: (run: GitHubWorkflowRun) => void;
}

function errorMessage(value: unknown, fallback: string): string {
  if (typeof value === "string") return value;
  if (value instanceof Error && value.message) return value.message;
  return fallback;
}

export function GitHubSection({
  githubDashboard,
  hasToken,
  onTokenChanged,
  onSelectRun,
}: GitHubSectionProps) {
  const [newToken, setNewToken] = useState("");
  const [showTokenInput, setShowTokenInput] = useState(false);
  const [savingToken, setSavingToken] = useState(false);

  const handleSaveToken = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newToken.trim()) return;

    setSavingToken(true);
    try {
      await commands.saveGitHubPat(newToken);
      setNewToken("");
      setShowTokenInput(false);
      onTokenChanged();
    } catch (error) {
      alert("Failed to save token: " + errorMessage(error, "Unknown error"));
    } finally {
      setSavingToken(false);
    }
  };

  const handleClearToken = async () => {
    if (!confirm("Are you sure you want to delete the stored GitHub PAT?")) return;
    try {
      await commands.deleteGitHubPat();
      onTokenChanged();
    } catch (error) {
      alert("Failed to delete token: " + errorMessage(error, "Unknown error"));
    }
  };

  return (
    <div className="repository-column" style={{ display: "flex", flexDirection: "column", gap: 24 }}>
      {/* GitHub Token configuration panel */}
      <div
        className="repository-card"
        style={{
          background: "var(--bg-tertiary)",
          border: "1px solid var(--border-secondary)",
          borderRadius: 12,
          padding: 18,
        }}
      >
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 14 }}>
          <h2
            style={{
              fontSize: "var(--text-lg)",
              fontWeight: 700,
              margin: 0,
              color: "var(--text-primary)",
              display: "flex",
              alignItems: "center",
              gap: 8,
            }}
          >
            <i className="material-symbols-outlined" style={{ fontSize: 18, color: "var(--text-tertiary)" }}>
              key
            </i>
            GitHub Authentication
          </h2>

          <span
            style={{
              fontSize: "var(--text-xs)",
              fontWeight: 700,
              color: hasToken ? "#10b981" : "#f59e0b",
              background: hasToken ? "rgba(16,185,129,.12)" : "rgba(245,158,11,.12)",
              borderRadius: 5,
              padding: "3px 8px",
            }}
          >
            {hasToken ? "SECURELY CONFIGURED" : "NO TOKEN"}
          </span>
        </div>

        {!hasToken && !showTokenInput && (
          <div>
            <p style={{ margin: "0 0 12px", fontSize: "var(--text-sm)", color: "var(--text-secondary)", lineHeight: 1.5 }}>
              No Personal Access Token (PAT) configured. Storing a PAT enables secure calls to the GitHub API for checking private repos, branch status, and PRs.
            </p>
            <button
              type="button"
              onClick={() => setShowTokenInput(true)}
              style={{
                border: "none",
                borderRadius: 7,
                background: "linear-gradient(135deg,#806cf6,#557ff2)",
                color: "#fff",
                padding: "6px 12px",
                fontSize: "var(--text-xs)",
                fontWeight: 700,
                cursor: "pointer",
              }}
            >
              Setup GitHub PAT Token
            </button>
          </div>
        )}

        {showTokenInput && (
          <form onSubmit={handleSaveToken} style={{ display: "flex", flexDirection: "column", gap: 10 }}>
            <div style={{ fontSize: "var(--text-sm)", color: "var(--text-secondary)" }}>
              Enter GitHub PAT (stored safely in your OS keyring):
            </div>
            <div style={{ display: "flex", gap: 8 }}>
              <input
                type="password"
                value={newToken}
                onChange={(e) => setNewToken(e.target.value)}
                placeholder="ghp_..."
                style={{
                  flex: 1,
                  background: "var(--bg-primary)",
                  border: "1px solid var(--border-primary)",
                  borderRadius: 6,
                  padding: "6px 10px",
                  fontSize: "var(--text-sm)",
                  color: "var(--text-primary)",
                  fontFamily: "var(--font-mono)",
                  outline: "none",
                }}
              />
              <button
                type="submit"
                disabled={savingToken || !newToken.trim()}
                style={{
                  border: "none",
                  borderRadius: 6,
                  background: "linear-gradient(135deg,#46b07d,#3a9368)",
                  color: "#fff",
                  padding: "6px 12px",
                  fontSize: "var(--text-xs)",
                  fontWeight: 700,
                  cursor: "pointer",
                  opacity: savingToken || !newToken.trim() ? 0.6 : 1,
                }}
              >
                Save
              </button>
              <button
                type="button"
                onClick={() => setShowTokenInput(false)}
                style={{
                  border: "1px solid #302a39",
                  borderRadius: 6,
                  background: "#19151f",
                  color: "var(--text-secondary)",
                  padding: "6px 12px",
                  fontSize: "var(--text-xs)",
                  fontWeight: 650,
                  cursor: "pointer",
                }}
              >
                Cancel
              </button>
            </div>
          </form>
        )}

        {hasToken && (
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
            <span style={{ fontSize: "var(--text-sm)", color: "var(--text-secondary)" }}>
              PAT token is securely stored in Windows Credential Manager.
            </span>
            <button
              type="button"
              onClick={handleClearToken}
              style={{
                border: "1px solid rgba(239,68,68,.3)",
                borderRadius: 6,
                background: "transparent",
                color: "#ef4444",
                padding: "5px 10px",
                fontSize: "var(--text-xs)",
                fontWeight: 600,
                cursor: "pointer",
              }}
            >
              Delete Token
            </button>
          </div>
        )}
      </div>

      {/* GitHub Pull Requests List */}
      <div
        className="repository-card"
        style={{
          background: "var(--bg-tertiary)",
          border: "1px solid var(--border-secondary)",
          borderRadius: 12,
          padding: 18,
        }}
      >
        <h2
          style={{
            fontSize: "var(--text-lg)",
            fontWeight: 700,
            margin: "0 0 14px",
            color: "var(--text-primary)",
            display: "flex",
            alignItems: "center",
            gap: 8,
          }}
        >
          <i className="material-symbols-outlined" style={{ fontSize: 18, color: "var(--text-tertiary)" }}>
            call_merge
          </i>
          GitHub Open Pull Requests
        </h2>

        <div style={{ display: "flex", flexDirection: "column", gap: 8, maxHeight: 260, overflowY: "auto" }}>
          {githubDashboard && githubDashboard.pull_requests.length === 0 && (
            <div style={{ padding: "20px 0", textAlign: "center", color: "var(--text-tertiary)", fontSize: "var(--text-md)" }}>
              No open pull requests found.
            </div>
          )}

          {!githubDashboard && (
            <div style={{ padding: "20px 0", textAlign: "center", color: "var(--text-tertiary)", fontSize: "var(--text-md)" }}>
              No remote delivery info (public rate limits or missing token).
            </div>
          )}

          {githubDashboard &&
            githubDashboard.pull_requests.map((pr) => (
              <a
                key={pr.number}
                href={pr.html_url}
                target="_blank"
                rel="noopener noreferrer"
                style={{
                  display: "block",
                  padding: 10,
                  background: "var(--bg-primary)",
                  border: "1px solid var(--border-primary)",
                  borderRadius: 8,
                  textDecoration: "none",
                  color: "inherit",
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.borderColor = "#9384f8";
                  e.currentTarget.style.background = "#1b1824";
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.borderColor = "var(--border-primary)";
                  e.currentTarget.style.background = "var(--bg-primary)";
                }}
              >
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", marginBottom: 4 }}>
                  <span
                    className="repository-link-copy repository-ellipsis"
                    style={{
                      fontSize: "var(--text-md)",
                      fontWeight: 650,
                      color: "var(--text-primary)",
                      flex: 1,
                      paddingRight: 8,
                    }}
                  >
                    #{pr.number} {pr.title}
                  </span>
                  {pr.draft && (
                    <span
                      style={{
                        fontSize: "var(--text-2xs)",
                        fontWeight: 700,
                        color: "#9ca3af",
                        background: "rgba(156,163,175,.15)",
                        padding: "1px 5px",
                        borderRadius: 4,
                        flexShrink: 0,
                      }}
                    >
                      DRAFT
                    </span>
                  )}
                </div>
                <div style={{ display: "flex", justifyContent: "space-between", fontSize: "var(--text-xs)", color: "var(--text-tertiary)" }}>
                  <span>Opened by {pr.user}</span>
                  <span>{new Date(pr.created_at).toLocaleDateString()}</span>
                </div>
              </a>
            ))}
        </div>
      </div>

      {/* GitHub Actions Workflows List */}
      <div
        className="repository-card"
        style={{
          background: "var(--bg-tertiary)",
          border: "1px solid var(--border-secondary)",
          borderRadius: 12,
          padding: 18,
        }}
      >
        <h2
          style={{
            fontSize: "var(--text-lg)",
            fontWeight: 700,
            margin: "0 0 14px",
            color: "var(--text-primary)",
            display: "flex",
            alignItems: "center",
            gap: 8,
          }}
        >
          <i className="material-symbols-outlined" style={{ fontSize: 18, color: "var(--text-tertiary)" }}>
            cycle
          </i>
          GitHub Actions Runs
        </h2>

        <div style={{ display: "flex", flexDirection: "column", gap: 8, maxHeight: 260, overflowY: "auto" }}>
          {githubDashboard && githubDashboard.workflow_runs.length === 0 && (
            <div style={{ padding: "20px 0", textAlign: "center", color: "var(--text-tertiary)", fontSize: "var(--text-md)" }}>
              No workflow runs found.
            </div>
          )}

          {!githubDashboard && (
            <div style={{ padding: "20px 0", textAlign: "center", color: "var(--text-tertiary)", fontSize: "var(--text-md)" }}>
              No workflow run statistics available.
            </div>
          )}

          {githubDashboard &&
            githubDashboard.workflow_runs.map((run) => {
              const s = getWorkflowRunStatusStyle(run.status, run.conclusion);
              return (
                <button
                  type="button"
                  key={run.id}
                  aria-label={`View details for run ${describeWorkflowRun(run)}`}
                  onClick={() => onSelectRun(run)}
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: 12,
                    padding: 10,
                    background: "var(--bg-primary)",
                    border: "1px solid var(--border-primary)",
                    borderRadius: 8,
                    textDecoration: "none",
                    color: "inherit",
                    font: "inherit",
                    textAlign: "left",
                    width: "100%",
                    cursor: "pointer",
                  }}
                  onMouseEnter={(e) => {
                    e.currentTarget.style.borderColor = "#9384f8";
                    e.currentTarget.style.background = "#1b1824";
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.borderColor = "var(--border-primary)";
                    e.currentTarget.style.background = "var(--bg-primary)";
                  }}
                >
                  <i
                    className={`material-symbols-outlined ${s.spin ? "spin-icon" : ""}`}
                    aria-hidden="true"
                    style={{ fontSize: 20, color: s.color }}
                  >
                    {s.icon}
                  </i>

                  <div style={{ flex: 1, minWidth: 0 }}>
                    <div className="repository-ellipsis" style={{ fontSize: "var(--text-sm)", fontWeight: 600, color: "var(--text-primary)" }}>
                      {run.name}
                    </div>
                    <div style={{ fontSize: "var(--text-xs)", color: "var(--text-tertiary)", marginTop: 2 }}>
                      branch:{" "}
                      <span style={{ fontFamily: "var(--font-mono)", color: "var(--text-secondary)" }}>
                        {run.head_branch}
                      </span>
                    </div>
                  </div>

                  <div style={{ display: "flex", flexDirection: "column", alignItems: "flex-end", flexShrink: 0 }}>
                    <span
                      style={{
                        fontSize: "var(--text-2xs)",
                        fontWeight: 700,
                        textTransform: "uppercase",
                        color: s.color,
                        background: s.bg,
                        padding: "2px 6px",
                        borderRadius: 4,
                        marginBottom: 4,
                      }}
                    >
                      {s.label}
                    </span>
                    <span style={{ fontSize: "var(--text-2xs)", color: "var(--text-tertiary)" }}>
                      {new Date(run.created_at).toLocaleDateString()}
                    </span>
                  </div>
                </button>
              );
            })}
        </div>
      </div>
    </div>
  );
}
