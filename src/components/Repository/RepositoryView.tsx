import { useEffect, useState } from "react";
import * as commands from "../../lib/commands";
import type { GitDetails, GitHubDashboard, GitFile } from "../../types";

export function RepositoryView() {
  const [gitDetails, setGitDetails] = useState<GitDetails | null>(null);
  const [githubDashboard, setGithubDashboard] = useState<GitHubDashboard | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  
  // PAT token config state
  const [hasToken, setHasToken] = useState(false);
  const [newToken, setNewToken] = useState("");
  const [showTokenInput, setShowTokenInput] = useState(false);
  const [savingToken, setSavingToken] = useState(false);

  const loadData = async () => {
    setLoading(true);
    setError(null);
    try {
      // 1. Fetch local Git details
      const git = await commands.getGitDetails();
      setGitDetails(git);

      // 2. Check if GitHub PAT is configured
      const tokenConfigured = await commands.getGitHubPatConfigured();
      setHasToken(tokenConfigured);

      // 3. If owner and repo are resolved, fetch GitHub dashboard data
      if (git.owner && git.repo) {
        try {
          const gh = await commands.getGitHubDashboard(git.owner, git.repo);
          setGithubDashboard(gh);
        } catch (err) {
          console.warn("GitHub API fetch failed:", err);
          // Don't fail the whole view if GitHub fails (could be public repository limit, etc.)
          setGithubDashboard(null);
        }
      }
    } catch (err: any) {
      setError(typeof err === "string" ? err : err.message || "Failed to load Git details");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadData();
  }, []);

  const handleSaveToken = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newToken.trim()) return;

    setSavingToken(true);
    try {
      await commands.saveGitHubPat(newToken);
      setHasToken(true);
      setNewToken("");
      setShowTokenInput(false);
      await loadData();
    } catch (err: any) {
      alert("Failed to save token: " + err);
    } finally {
      setSavingToken(false);
    }
  };

  const handleClearToken = async () => {
    if (!confirm("Are you sure you want to delete the stored GitHub PAT?")) return;
    try {
      await commands.deleteGitHubPat();
      setHasToken(false);
      await loadData();
    } catch (err: any) {
      alert("Failed to delete token: " + err);
    }
  };

  const getStatusColor = (status: GitFile["status"]) => {
    switch (status) {
      case "staged":
        return "#46b07d";
      case "unstaged":
        return "#f59e0b";
      case "untracked":
        return "#9ca3af";
      case "conflicted":
      case "deleted":
        return "#ef4444";
      case "renamed":
        return "#3b82f6";
      default:
        return "#9ca3af";
    }
  };

  const getRunStatusStyles = (status: string, conclusion: string | null) => {
    if (status === "completed") {
      if (conclusion === "success") {
        return { color: "#10b981", bg: "rgba(16,185,129,.12)", icon: "check_circle" };
      } else if (conclusion === "failure" || conclusion === "timed_out") {
        return { color: "#ef4444", bg: "rgba(239,68,68,.12)", icon: "error" };
      } else if (conclusion === "cancelled") {
        return { color: "#9ca3af", bg: "rgba(156,163,175,.12)", icon: "cancel" };
      }
    }
    // queued or in_progress
    return { color: "#6366f1", bg: "rgba(99,102,241,.12)", icon: "pending" };
  };

  return (
    <div style={{ overflowY: "auto", height: "100%", background: "var(--bg-primary)" }}>
      <div style={{ maxWidth: 1080, margin: "0 auto", padding: "24px 36px 70px" }}>
        
        {/* Header Section */}
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", marginBottom: 24 }}>
          <div>
            <h1 style={{ fontSize: 24, fontWeight: 800, letterSpacing: "-.025em", margin: "0 0 5px", color: "var(--text-primary)" }}>
              Repository Dashboard
            </h1>
            <p style={{ fontSize: 13.5, color: "var(--text-tertiary)", margin: 0 }}>
              Observe Git status and delivery information for this repository.
            </p>
          </div>
          
          <button
            onClick={loadData}
            disabled={loading}
            style={{
              display: "inline-flex",
              alignItems: "center",
              gap: 6,
              border: "1px solid #302a39",
              borderRadius: 7,
              background: "#19151f",
              color: "var(--text-secondary)",
              padding: "7px 12px",
              fontSize: 12,
              fontWeight: 600,
              cursor: "pointer",
              opacity: loading ? 0.6 : 1,
            }}
          >
            <i className={`material-symbols-outlined ${loading ? "spin-icon" : ""}`} style={{ fontSize: 16 }}>
              refresh
            </i>
            Refresh
            {loading && <style>{`.spin-icon { animation: spin 1s linear infinite; } @keyframes spin { to { transform: rotate(360deg); } }`}</style>}
          </button>
        </div>

        {error && (
          <div style={{ padding: 16, borderRadius: 9, background: "rgba(239,68,68,.08)", border: "1px solid rgba(239,68,68,.2)", color: "#f87171", fontSize: 13.5, marginBottom: 24 }}>
            <div style={{ fontWeight: 700, marginBottom: 4 }}>Error loading repository data</div>
            {error}
          </div>
        )}

        {/* main grids */}
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 24 }}>
          
          {/* Left Column: Local Git details */}
          <div style={{ display: "flex", flexDirection: "column", gap: 24 }}>
            
            {/* Git Metadata Card */}
            <div style={{ background: "var(--bg-tertiary)", border: "1px solid var(--border-secondary)", borderRadius: 12, padding: 18 }}>
              <h2 style={{ fontSize: 15, fontWeight: 700, margin: "0 0 16px", color: "var(--text-primary)", display: "flex", alignItems: "center", gap: 8 }}>
                <i className="material-symbols-outlined" style={{ fontSize: 18, color: "var(--text-tertiary)" }}>schema</i>
                Local Git Status
              </h2>

              {gitDetails ? (
                <div style={{ display: "flex", flexDirection: "column", gap: 14 }}>
                  <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12 }}>
                    
                    {/* Branch */}
                    <div style={{ background: "var(--bg-primary)", padding: "10px 14px", borderRadius: 8, border: "1px solid var(--border-primary)" }}>
                      <div style={{ fontSize: 10, textTransform: "uppercase", letterSpacing: ".06em", color: "var(--text-muted)", marginBottom: 4 }}>
                        Current Branch
                      </div>
                      <div style={{ fontSize: 13, fontWeight: 700, color: "#bcaef6", fontFamily: "var(--font-mono)", overflow: "hidden", textOverflow: "ellipsis" }}>
                        {gitDetails.branch}
                      </div>
                    </div>

                    {/* Commit */}
                    <div style={{ background: "var(--bg-primary)", padding: "10px 14px", borderRadius: 8, border: "1px solid var(--border-primary)" }}>
                      <div style={{ fontSize: 10, textTransform: "uppercase", letterSpacing: ".06em", color: "var(--text-muted)", marginBottom: 4 }}>
                        Active Commit
                      </div>
                      <div style={{ fontSize: 13, fontWeight: 700, color: "var(--text-secondary)", fontFamily: "var(--font-mono)" }}>
                        {gitDetails.current_commit}
                      </div>
                    </div>
                  </div>

                  {/* Sync State info */}
                  <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", background: "var(--bg-primary)", padding: "10px 14px", borderRadius: 8, border: "1px solid var(--border-primary)" }}>
                    <span style={{ fontSize: 12.5, color: "var(--text-secondary)" }}>Tracking Branch Status</span>
                    {gitDetails.ahead === 0 && gitDetails.behind === 0 ? (
                      <span style={{ fontSize: 11.5, fontWeight: 600, color: "var(--text-muted)", background: "rgba(255,255,255,.05)", padding: "3px 8px", borderRadius: 5 }}>
                        In sync with remote
                      </span>
                    ) : (
                      <div style={{ display: "flex", gap: 8 }}>
                        {gitDetails.ahead > 0 && (
                          <span style={{ fontSize: 11.5, fontWeight: 700, color: "#10b981", background: "rgba(16,185,129,.1)", padding: "3px 8px", borderRadius: 5 }}>
                            ↑ {gitDetails.ahead} ahead
                          </span>
                        )}
                        {gitDetails.behind > 0 && (
                          <span style={{ fontSize: 11.5, fontWeight: 700, color: "#ef4444", background: "rgba(239,68,68,.1)", padding: "3px 8px", borderRadius: 5 }}>
                            ↓ {gitDetails.behind} behind
                          </span>
                        )}
                      </div>
                    )}
                  </div>

                  {/* Remote URL info */}
                  {gitDetails.remote_url && (
                    <div style={{ fontSize: 12, color: "var(--text-tertiary)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                      Remote: <span style={{ fontFamily: "var(--font-mono)", color: "var(--text-secondary)" }}>{gitDetails.remote_url}</span>
                    </div>
                  )}
                </div>
              ) : (
                <div style={{ color: "var(--text-tertiary)", fontSize: 13 }}>Loading git metadata...</div>
              )}
            </div>

            {/* Changed Files List */}
            <div style={{ background: "var(--bg-tertiary)", border: "1px solid var(--border-secondary)", borderRadius: 12, padding: 18, flex: 1 }}>
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 16 }}>
                <h2 style={{ fontSize: 15, fontWeight: 700, margin: 0, color: "var(--text-primary)", display: "flex", alignItems: "center", gap: 8 }}>
                  <i className="material-symbols-outlined" style={{ fontSize: 18, color: "var(--text-tertiary)" }}>edit_document</i>
                  Changed Files
                </h2>
                {gitDetails && (
                  <span style={{ fontSize: 11.5, color: "var(--text-tertiary)", background: "var(--bg-primary)", padding: "2px 6px", borderRadius: 5, border: "1px solid var(--border-primary)" }}>
                    {gitDetails.files.length} changes
                  </span>
                )}
              </div>

              <div style={{ overflowY: "auto", maxHeight: 380, display: "flex", flexDirection: "column", gap: 6 }}>
                {gitDetails && gitDetails.files.length === 0 && (
                  <div style={{ textAlign: "center", padding: "40px 0", color: "var(--text-tertiary)", fontSize: 13 }}>
                    <i className="material-symbols-outlined" style={{ fontSize: 32, color: "#46b07d", marginBottom: 8, display: "block" }}>check_circle</i>
                    Working directory clean.
                  </div>
                )}

                {gitDetails && gitDetails.files.map((file, idx) => (
                  <div
                    key={idx}
                    style={{
                      display: "flex",
                      alignItems: "center",
                      justifyContent: "space-between",
                      padding: "8px 12px",
                      background: "var(--bg-primary)",
                      border: "1px solid var(--border-primary)",
                      borderRadius: 8,
                      fontSize: 12.5,
                    }}
                  >
                    <div style={{ flex: 1, minWidth: 0, paddingRight: 10 }}>
                      <div
                        title={file.path}
                        style={{
                          fontFamily: "var(--font-mono)",
                          color: "var(--text-primary)",
                          overflow: "hidden",
                          textOverflow: "ellipsis",
                          whiteSpace: "nowrap",
                        }}
                      >
                        {file.path}
                      </div>
                      {file.original_path && (
                        <div style={{ fontSize: 10, color: "var(--text-muted)", fontFamily: "var(--font-mono)" }}>
                          renamed from: {file.original_path}
                        </div>
                      )}
                    </div>

                    <span
                      style={{
                        fontSize: 10,
                        fontWeight: 700,
                        textTransform: "uppercase",
                        color: getStatusColor(file.status),
                        background: `${getStatusColor(file.status)}15`,
                        padding: "2px 6px",
                        borderRadius: 4,
                        flexShrink: 0,
                      }}
                    >
                      {file.status}
                    </span>
                  </div>
                ))}
              </div>
            </div>

          </div>

          {/* Right Column: GitHub integration details */}
          <div style={{ display: "flex", flexDirection: "column", gap: 24 }}>
            
            {/* GitHub Token configuration panel */}
            <div style={{ background: "var(--bg-tertiary)", border: "1px solid var(--border-secondary)", borderRadius: 12, padding: 18 }}>
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 14 }}>
                <h2 style={{ fontSize: 15, fontWeight: 700, margin: 0, color: "var(--text-primary)", display: "flex", alignItems: "center", gap: 8 }}>
                  <i className="material-symbols-outlined" style={{ fontSize: 18, color: "var(--text-tertiary)" }}>key</i>
                  GitHub Authentication
                </h2>
                
                <span
                  style={{
                    fontSize: 10.5,
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
                  <p style={{ margin: "0 0 12px", fontSize: 12.5, color: "var(--text-secondary)", lineHeight: 1.5 }}>
                    No Personal Access Token (PAT) configured. Storing a PAT enables secure calls to the GitHub API for checking private repos, branch status, and PRs.
                  </p>
                  <button
                    onClick={() => setShowTokenInput(true)}
                    style={{
                      border: "none",
                      borderRadius: 7,
                      background: "linear-gradient(135deg,#806cf6,#557ff2)",
                      color: "#fff",
                      padding: "6px 12px",
                      fontSize: 11.5,
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
                  <div style={{ fontSize: 12, color: "var(--text-secondary)" }}>
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
                        fontSize: 12.5,
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
                        fontSize: 11.5,
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
                        fontSize: 11.5,
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
                  <span style={{ fontSize: 12.5, color: "var(--text-secondary)" }}>
                    PAT token is securely stored in Windows Credential Manager.
                  </span>
                  <button
                    onClick={handleClearToken}
                    style={{
                      border: "1px solid rgba(239,68,68,.3)",
                      borderRadius: 6,
                      background: "transparent",
                      color: "#ef4444",
                      padding: "5px 10px",
                      fontSize: 11,
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
            <div style={{ background: "var(--bg-tertiary)", border: "1px solid var(--border-secondary)", borderRadius: 12, padding: 18 }}>
              <h2 style={{ fontSize: 15, fontWeight: 700, margin: "0 0 14px", color: "var(--text-primary)", display: "flex", alignItems: "center", gap: 8 }}>
                <i className="material-symbols-outlined" style={{ fontSize: 18, color: "var(--text-tertiary)" }}>call_merge</i>
                GitHub Open Pull Requests
              </h2>

              <div style={{ display: "flex", flexDirection: "column", gap: 8, maxHeight: 260, overflowY: "auto" }}>
                {githubDashboard && githubDashboard.pull_requests.length === 0 && (
                  <div style={{ padding: "20px 0", textAlign: "center", color: "var(--text-tertiary)", fontSize: 13 }}>
                    No open pull requests found.
                  </div>
                )}

                {!githubDashboard && (
                  <div style={{ padding: "20px 0", textAlign: "center", color: "var(--text-tertiary)", fontSize: 13 }}>
                    No remote delivery info (public rate limits or missing token).
                  </div>
                )}

                {githubDashboard && githubDashboard.pull_requests.map((pr) => (
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
                      <span style={{ fontSize: 13, fontWeight: 650, color: "var(--text-primary)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", flex: 1, paddingRight: 8 }}>
                        #{pr.number} {pr.title}
                      </span>
                      {pr.draft && (
                        <span style={{ fontSize: 9, fontWeight: 700, color: "#9ca3af", background: "rgba(156,163,175,.15)", padding: "1px 5px", borderRadius: 4, flexShrink: 0 }}>
                          DRAFT
                        </span>
                      )}
                    </div>
                    <div style={{ display: "flex", justifyContent: "space-between", fontSize: 11, color: "var(--text-tertiary)" }}>
                      <span>Opened by {pr.user}</span>
                      <span>{new Date(pr.created_at).toLocaleDateString()}</span>
                    </div>
                  </a>
                ))}
              </div>
            </div>

            {/* GitHub Actions Workflows List */}
            <div style={{ background: "var(--bg-tertiary)", border: "1px solid var(--border-secondary)", borderRadius: 12, padding: 18 }}>
              <h2 style={{ fontSize: 15, fontWeight: 700, margin: "0 0 14px", color: "var(--text-primary)", display: "flex", alignItems: "center", gap: 8 }}>
                <i className="material-symbols-outlined" style={{ fontSize: 18, color: "var(--text-tertiary)" }}>cycle</i>
                GitHub Actions Runs
              </h2>

              <div style={{ display: "flex", flexDirection: "column", gap: 8, maxHeight: 260, overflowY: "auto" }}>
                {githubDashboard && githubDashboard.workflow_runs.length === 0 && (
                  <div style={{ padding: "20px 0", textAlign: "center", color: "var(--text-tertiary)", fontSize: 13 }}>
                    No workflow runs found.
                  </div>
                )}

                {!githubDashboard && (
                  <div style={{ padding: "20px 0", textAlign: "center", color: "var(--text-tertiary)", fontSize: 13 }}>
                    No workflow run statistics available.
                  </div>
                )}

                {githubDashboard && githubDashboard.workflow_runs.map((run) => {
                  const s = getRunStatusStyles(run.status, run.conclusion);
                  return (
                    <a
                      key={run.id}
                      href={run.html_url}
                      target="_blank"
                      rel="noopener noreferrer"
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
                      <i className="material-symbols-outlined" style={{ fontSize: 20, color: s.color }}>
                        {s.icon}
                      </i>
                      
                      <div style={{ flex: 1, minWidth: 0 }}>
                        <div style={{ fontSize: 12.5, fontWeight: 600, color: "var(--text-primary)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                          {run.name}
                        </div>
                        <div style={{ fontSize: 10.5, color: "var(--text-tertiary)", marginTop: 2 }}>
                          branch: <span style={{ fontFamily: "var(--font-mono)", color: "var(--text-secondary)" }}>{run.head_branch}</span>
                        </div>
                      </div>
                      
                      <div style={{ display: "flex", flexDirection: "column", alignItems: "flex-end", flexShrink: 0 }}>
                        <span
                          style={{
                            fontSize: 9.5,
                            fontWeight: 700,
                            color: s.color,
                            background: s.bg,
                            padding: "2px 6px",
                            borderRadius: 4,
                            marginBottom: 4,
                          }}
                        >
                          {(run.conclusion || run.status).toUpperCase()}
                        </span>
                        <span style={{ fontSize: 10, color: "var(--text-tertiary)" }}>
                          {new Date(run.created_at).toLocaleDateString()}
                        </span>
                      </div>
                    </a>
                  );
                })}
              </div>
            </div>

          </div>

        </div>

      </div>
    </div>
  );
}
