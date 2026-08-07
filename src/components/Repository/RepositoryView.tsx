import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { CommitGraph } from "commit-graph";
import * as commands from "../../lib/commands";
import type { GitDetails, GitHubDashboard, GitFile, GitHubWorkflowRun, GitWorktree } from "../../types";
import { GitDiffModal } from "./GitDiffModal";
import { WorkflowRunModal } from "./WorkflowRunModal";
import { describeWorkflowRun, getWorkflowRunStatusStyle } from "../../lib/workflowRunStatus";
import { RefreshButton } from "../RefreshButton";
import "./RepositoryView.css";

interface RepositoryData {
  gitDetails: GitDetails;
  worktrees: GitWorktree[];
  githubDashboard: GitHubDashboard | null;
  hasToken: boolean;
}

function errorMessage(value: unknown, fallback: string): string {
  if (typeof value === "string") return value;
  if (value instanceof Error && value.message) return value.message;
  return fallback;
}

async function fetchRepositoryData(): Promise<RepositoryData> {
  const gitDetails = await commands.getGitDetails();
  const worktrees = await commands.getGitWorktrees().catch(() => [] as GitWorktree[]);
  const hasToken = await commands.getGitHubPatConfigured();
  let githubDashboard: GitHubDashboard | null = null;

  if (gitDetails.owner && gitDetails.repo) {
    try {
      githubDashboard = await commands.getGitHubDashboard(gitDetails.owner, gitDetails.repo);
    } catch (error) {
      console.warn("GitHub API fetch failed:", error);
    }
  }

  return { gitDetails, worktrees, githubDashboard, hasToken };
}

export function RepositoryView() {
  const [gitDetails, setGitDetails] = useState<GitDetails | null>(null);
  const [worktrees, setWorktrees] = useState<GitWorktree[]>([]);
  const [fileScope, setFileScope] = useState<string>("");
  const [githubDashboard, setGithubDashboard] = useState<GitHubDashboard | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  
  // PAT token config state
  const [hasToken, setHasToken] = useState(false);
  const [newToken, setNewToken] = useState("");
  const [showTokenInput, setShowTokenInput] = useState(false);
  const [savingToken, setSavingToken] = useState(false);
  const [selectedFile, setSelectedFile] = useState<GitFile | null>(null);
  const [selectedRun, setSelectedRun] = useState<GitHubWorkflowRun | null>(null);

  const loadData = async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await fetchRepositoryData();
      setGitDetails(data.gitDetails);
      setWorktrees(data.worktrees);
      setGithubDashboard(data.githubDashboard);
      setHasToken(data.hasToken);
    } catch (error) {
      setError(errorMessage(error, "Failed to load Git details"));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    let active = true;

    fetchRepositoryData()
      .then((data) => {
        if (!active) return;
        setGitDetails(data.gitDetails);
        setWorktrees(data.worktrees);
        setGithubDashboard(data.githubDashboard);
        setHasToken(data.hasToken);
      })
      .catch((error: unknown) => {
        if (active) setError(errorMessage(error, "Failed to load Git details"));
      })
      .finally(() => {
        if (active) setLoading(false);
      });

    return () => {
      active = false;
    };
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
      setHasToken(false);
      await loadData();
    } catch (error) {
      alert("Failed to delete token: " + errorMessage(error, "Unknown error"));
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

  const scopedWorktree = worktrees.find((worktree) => worktree.name === fileScope) ?? null;
  const effectiveScope = scopedWorktree ? fileScope : "";
  const scopedFiles = scopedWorktree
    ? scopedWorktree.details?.files ?? []
    : gitDetails?.files ?? [];

  return (
    <div className="repository-scroll">
      <div className="repository-page">
        
        {/* Header Section */}
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", marginBottom: 24 }}>
          <div>
            <h1 style={{ fontSize: "var(--text-2xl)", fontWeight: 800, letterSpacing: "-.025em", margin: "0 0 5px", color: "var(--text-primary)" }}>
              Repository Dashboard
            </h1>
            <p style={{ fontSize: "var(--text-md)", color: "var(--text-tertiary)", margin: 0 }}>
              Observe Git status and delivery information for this repository.
            </p>
          </div>
          
          <RefreshButton loading={loading} onClick={loadData} />
        </div>

        {error && (
          <div style={{ padding: 16, borderRadius: 9, background: "rgba(239,68,68,.08)", border: "1px solid rgba(239,68,68,.2)", color: "#f87171", fontSize: "var(--text-md)", marginBottom: 24 }}>
            <div style={{ fontWeight: 700, marginBottom: 4 }}>Error loading repository data</div>
            {error}
          </div>
        )}

        {/* main grids */}
        <div className="repository-grid">
          
          {/* Left Column: Local Git details */}
          <div className="repository-column" style={{ display: "flex", flexDirection: "column", gap: 24 }}>
            
            {/* Git Metadata Card */}
            <div className="repository-card" style={{ background: "var(--bg-tertiary)", border: "1px solid var(--border-secondary)", borderRadius: 12, padding: 18 }}>
              <h2 style={{ fontSize: "var(--text-lg)", fontWeight: 700, margin: "0 0 16px", color: "var(--text-primary)", display: "flex", alignItems: "center", gap: 8 }}>
                <i className="material-symbols-outlined" style={{ fontSize: 18, color: "var(--text-tertiary)" }}>schema</i>
                Local Git Status
              </h2>

              {gitDetails ? (
                <div style={{ display: "flex", flexDirection: "column", gap: 14 }}>
                  <div className="repository-metadata-grid">
                    
                    {/* Branch */}
                    <div style={{ background: "var(--bg-primary)", padding: "10px 14px", borderRadius: 8, border: "1px solid var(--border-primary)" }}>
                      <div style={{ fontSize: "var(--text-2xs)", textTransform: "uppercase", letterSpacing: ".06em", color: "var(--text-muted)", marginBottom: 4 }}>
                        Current Branch
                      </div>
                      <div className="repository-ellipsis" title={gitDetails.branch} style={{ fontSize: "var(--text-md)", fontWeight: 700, color: "#bcaef6", fontFamily: "var(--font-mono)" }}>
                        {gitDetails.branch}
                      </div>
                    </div>

                    {/* Commit */}
                    <div style={{ background: "var(--bg-primary)", padding: "10px 14px", borderRadius: 8, border: "1px solid var(--border-primary)" }}>
                      <div style={{ fontSize: "var(--text-2xs)", textTransform: "uppercase", letterSpacing: ".06em", color: "var(--text-muted)", marginBottom: 4 }}>
                        Active Commit
                      </div>
                      <div style={{ fontSize: "var(--text-md)", fontWeight: 700, color: "var(--text-secondary)", fontFamily: "var(--font-mono)" }}>
                        {gitDetails.current_commit}
                      </div>
                    </div>
                  </div>

                  {/* Sync State info */}
                  <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", background: "var(--bg-primary)", padding: "10px 14px", borderRadius: 8, border: "1px solid var(--border-primary)" }}>
                    <span style={{ fontSize: "var(--text-sm)", color: "var(--text-secondary)" }}>Tracking Branch Status</span>
                    {gitDetails.ahead === 0 && gitDetails.behind === 0 ? (
                      <span style={{ fontSize: "var(--text-xs)", fontWeight: 600, color: "var(--text-muted)", background: "rgba(255,255,255,.05)", padding: "3px 8px", borderRadius: 5 }}>
                        In sync with remote
                      </span>
                    ) : (
                      <div style={{ display: "flex", gap: 8 }}>
                        {gitDetails.ahead > 0 && (
                          <span style={{ fontSize: "var(--text-xs)", fontWeight: 700, color: "#10b981", background: "rgba(16,185,129,.1)", padding: "3px 8px", borderRadius: 5 }}>
                            ↑ {gitDetails.ahead} ahead
                          </span>
                        )}
                        {gitDetails.behind > 0 && (
                          <span style={{ fontSize: "var(--text-xs)", fontWeight: 700, color: "#ef4444", background: "rgba(239,68,68,.1)", padding: "3px 8px", borderRadius: 5 }}>
                            ↓ {gitDetails.behind} behind
                          </span>
                        )}
                      </div>
                    )}
                  </div>

                  {/* Remote URL info */}
                  {gitDetails.remote_url && (
                    <div className="repository-remote repository-ellipsis" title={gitDetails.remote_url} style={{ fontSize: "var(--text-sm)", color: "var(--text-tertiary)" }}>
                      Remote: <span style={{ fontFamily: "var(--font-mono)", color: "var(--text-secondary)" }}>{gitDetails.remote_url}</span>
                    </div>
                  )}

                  {/* Linked worktrees (agent workspaces) */}
                  {worktrees.length > 0 && (
                    <div>
                      <div style={{ fontSize: "var(--text-2xs)", textTransform: "uppercase", letterSpacing: ".06em", color: "var(--text-muted)", margin: "4px 0 8px" }}>
                        Worktrees · {worktrees.length}
                      </div>
                      <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
                        {worktrees.map((worktree) => (
                          <div
                            key={worktree.name}
                            style={{ background: "var(--bg-primary)", padding: "10px 14px", borderRadius: 8, border: "1px solid var(--border-primary)" }}
                          >
                            <div style={{ display: "flex", alignItems: "center", gap: 8, flexWrap: "wrap" }}>
                              <span className="repository-ellipsis" title={worktree.path} style={{ fontSize: "var(--text-sm)", fontWeight: 700, color: "#bcaef6", fontFamily: "var(--font-mono)" }}>
                                {worktree.branch ?? worktree.name}
                              </span>
                              {worktree.head && (
                                <span style={{ fontSize: "var(--text-xs)", color: "var(--text-secondary)", fontFamily: "var(--font-mono)" }}>
                                  {worktree.head}
                                </span>
                              )}
                              {worktree.prunable && (
                                <span style={{ fontSize: "var(--text-2xs)", fontWeight: 700, color: "#ef4444", background: "rgba(239,68,68,.1)", padding: "2px 6px", borderRadius: 4 }}>
                                  PRUNABLE
                                </span>
                              )}
                              {worktree.locked && (
                                <span style={{ fontSize: "var(--text-2xs)", fontWeight: 700, color: "#f59e0b", background: "rgba(245,158,11,.1)", padding: "2px 6px", borderRadius: 4 }}>
                                  LOCKED
                                </span>
                              )}
                              {worktree.details && (
                                <span style={{ marginLeft: "auto", fontSize: "var(--text-xs)", color: "var(--text-tertiary)" }}>
                                  {worktree.details.files.length} changes
                                </span>
                              )}
                            </div>
                            <div className="repository-ellipsis" title={worktree.path} style={{ marginTop: 3, fontSize: "var(--text-2xs)", color: "var(--text-muted)", fontFamily: "var(--font-mono)" }}>
                              {worktree.path}
                            </div>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}
                </div>
              ) : (
                <div style={{ color: "var(--text-tertiary)", fontSize: "var(--text-md)" }}>Loading git metadata...</div>
              )}
            </div>

            {/* Changed Files List */}
            <div className="repository-card" style={{ background: "var(--bg-tertiary)", border: "1px solid var(--border-secondary)", borderRadius: 12, padding: 18, flex: 1 }}>
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 16 }}>
                <h2 style={{ fontSize: "var(--text-lg)", fontWeight: 700, margin: 0, color: "var(--text-primary)", display: "flex", alignItems: "center", gap: 8 }}>
                  <i className="material-symbols-outlined" style={{ fontSize: 18, color: "var(--text-tertiary)" }}>edit_document</i>
                  Changed Files
                </h2>
                <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                  {worktrees.length > 0 && (
                    <select
                      className="app-select"
                      aria-label="Changed files scope"
                      value={effectiveScope}
                      onChange={(event) => setFileScope(event.target.value)}
                      style={{ height: 26, background: "var(--bg-primary)", color: "var(--text-secondary)", border: "1px solid var(--border-primary)", borderRadius: 5, fontSize: "var(--text-xs)", padding: "0 6px" }}
                    >
                      <option value="">main worktree</option>
                      {worktrees.filter((worktree) => !worktree.prunable).map((worktree) => (
                        <option key={worktree.name} value={worktree.name}>
                          {worktree.branch ?? worktree.name}
                        </option>
                      ))}
                    </select>
                  )}
                  {gitDetails && (
                    <span style={{ fontSize: "var(--text-xs)", color: "var(--text-tertiary)", background: "var(--bg-primary)", padding: "2px 6px", borderRadius: 5, border: "1px solid var(--border-primary)" }}>
                      {scopedFiles.length} changes
                    </span>
                  )}
                </div>
              </div>

              <div className="repository-file-list">
                {gitDetails && scopedFiles.length === 0 && (
                  <div style={{ textAlign: "center", padding: "40px 0", color: "var(--text-tertiary)", fontSize: "var(--text-md)" }}>
                    <i className="material-symbols-outlined" style={{ fontSize: 32, color: "#46b07d", marginBottom: 8, display: "block" }}>check_circle</i>
                    Working directory clean.
                  </div>
                )}

                {gitDetails && scopedFiles.map((file) => (
                  <button
                    type="button"
                    key={`${effectiveScope}:${file.diff_target}:${file.path}`}
                    className="repository-file-row"
                    aria-label={`View diff for ${file.path}, status ${file.status}, ${file.diff_target}`}
                    onClick={() => setSelectedFile(file)}
                    style={{ fontSize: "var(--text-sm)" }}
                  >
                    <div className="repository-file-copy">
                      <div
                        className="repository-ellipsis"
                        title={file.path}
                        style={{
                          fontFamily: "var(--font-mono)",
                          color: "var(--text-primary)",
                        }}
                      >
                        {file.path}
                      </div>
                      {file.original_path && (
                        <div className="repository-ellipsis" title={file.original_path} style={{ fontSize: "var(--text-2xs)", color: "var(--text-muted)", fontFamily: "var(--font-mono)" }}>
                          renamed from: {file.original_path}
                        </div>
                      )}
                    </div>

                    <span
                      style={{
                        fontSize: "var(--text-2xs)",
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
                  </button>
                ))}
              </div>
            </div>

          </div>

          {/* Right Column: GitHub integration details */}
          <div className="repository-column" style={{ display: "flex", flexDirection: "column", gap: 24 }}>
            
            {/* GitHub Token configuration panel */}
            <div className="repository-card" style={{ background: "var(--bg-tertiary)", border: "1px solid var(--border-secondary)", borderRadius: 12, padding: 18 }}>
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 14 }}>
                <h2 style={{ fontSize: "var(--text-lg)", fontWeight: 700, margin: 0, color: "var(--text-primary)", display: "flex", alignItems: "center", gap: 8 }}>
                  <i className="material-symbols-outlined" style={{ fontSize: 18, color: "var(--text-tertiary)" }}>key</i>
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
            <div className="repository-card" style={{ background: "var(--bg-tertiary)", border: "1px solid var(--border-secondary)", borderRadius: 12, padding: 18 }}>
              <h2 style={{ fontSize: "var(--text-lg)", fontWeight: 700, margin: "0 0 14px", color: "var(--text-primary)", display: "flex", alignItems: "center", gap: 8 }}>
                <i className="material-symbols-outlined" style={{ fontSize: 18, color: "var(--text-tertiary)" }}>call_merge</i>
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
                      <span className="repository-link-copy repository-ellipsis" style={{ fontSize: "var(--text-md)", fontWeight: 650, color: "var(--text-primary)", flex: 1, paddingRight: 8 }}>
                        #{pr.number} {pr.title}
                      </span>
                      {pr.draft && (
                        <span style={{ fontSize: "var(--text-2xs)", fontWeight: 700, color: "#9ca3af", background: "rgba(156,163,175,.15)", padding: "1px 5px", borderRadius: 4, flexShrink: 0 }}>
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
            <div className="repository-card" style={{ background: "var(--bg-tertiary)", border: "1px solid var(--border-secondary)", borderRadius: 12, padding: 18 }}>
              <h2 style={{ fontSize: "var(--text-lg)", fontWeight: 700, margin: "0 0 14px", color: "var(--text-primary)", display: "flex", alignItems: "center", gap: 8 }}>
                <i className="material-symbols-outlined" style={{ fontSize: 18, color: "var(--text-tertiary)" }}>cycle</i>
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

                {githubDashboard && githubDashboard.workflow_runs.map((run) => {
                  const s = getWorkflowRunStatusStyle(run.status, run.conclusion);
                  return (
                    <button
                      type="button"
                      key={run.id}
                      aria-label={`View details for run ${describeWorkflowRun(run)}`}
                      onClick={() => setSelectedRun(run)}
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
                          branch: <span style={{ fontFamily: "var(--font-mono)", color: "var(--text-secondary)" }}>{run.head_branch}</span>
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

        </div>

        <BranchGraphCard
          dashboard={githubDashboard}
          currentBranch={gitDetails?.branch ?? null}
        />

      </div>
      {selectedFile && (
        <GitDiffModal
          key={`${effectiveScope}:${selectedFile.diff_target}:${selectedFile.path}`}
          file={selectedFile}
          worktree={effectiveScope || undefined}
          onClose={() => setSelectedFile(null)}
        />
      )}
      {selectedRun && (
        <WorkflowRunModal
          key={selectedRun.id}
          run={selectedRun}
          onClose={() => setSelectedRun(null)}
        />
      )}
    </div>
  );
}

/*
                  <text x={x} y="18" textAnchor="middle" fill={isCurrent ? "#bcaef6" : "var(--text-secondary)"} fontSize="10" fontFamily="var(--font-mono)"><title>{branch.name}</title>{isCurrent ? "● " : isDefault ? "★ " : ""}{shortBranchName(branch.name)}</text>
                  {branchPoints.length > 0 ? <line x1={x} y1={branchPoints[0].y} x2={x} y2={branchPoints[branchPoints.length - 1].y + 18} stroke={branchPoints[0].color} strokeOpacity=".6" strokeWidth="2" /> : <text x={x} y="42" textAnchor="middle" fill="var(--text-muted)" fontSize="10">no unique commits</text>}
                </g>;
              })}
              {points.flatMap((point) => point.commit.parents.map((parentSha) => {
                const parent = points.find((candidate) => candidate.branch === point.branch && candidate.commit.sha === parentSha);
                if (!parent) return null;
                const curve = (point.y + parent.y) / 2;
                return <path key={`${point.commit.sha}-${parentSha}-${point.branch}`} d={`M ${point.x} ${point.y} C ${point.x} ${curve}, ${parent.x} ${curve}, ${parent.x} ${parent.y}`} fill="none" stroke={point.color} strokeOpacity=".65" strokeWidth="2" />;
              }))}
              {branches.map((branch) => {
                if (!branch.merge_base_sha || branch.name === dashboard.default_branch || branch.commits.length === 0) return null;
                const branchPoint = points.find((point) => point.branch === branch.name && point.commit.sha === branch.commits[branch.commits.length - 1].sha);
                const basePoint = points.find((point) => point.branch === dashboard.default_branch && point.commit.sha === branch.merge_base_sha);
                if (!branchPoint || !basePoint) return null;
                const curve = (branchPoint.y + basePoint.y) / 2;
                return <path key={`${branch.name}-merge-base`} d={`M ${branchPoint.x} ${branchPoint.y} C ${branchPoint.x} ${curve}, ${basePoint.x} ${curve}, ${basePoint.x} ${basePoint.y}`} fill="none" stroke={branchPoint.color} strokeOpacity=".65" strokeWidth="2" strokeDasharray="4 4" />;
              })}
              {points.map((point) => {
                const description = `${point.branch}, commit ${point.commit.sha.slice(0, 8)}${point.commit.message ? `, ${point.commit.message}` : ""}`;
                return <circle key={`${point.branch}-${point.commit.sha}`} cx={point.x} cy={point.y} r="7" fill={point.color} stroke="#11151d" strokeWidth="3" tabIndex={0} role="button" aria-label={description} onMouseEnter={(event) => showTooltip(point.branch, point.commit, point.color, event.currentTarget)} onFocus={(event) => showTooltip(point.branch, point.commit, point.color, event.currentTarget)} onMouseLeave={() => setHoveredCommit(null)} onBlur={() => setHoveredCommit(null)}><title>{description}</title></circle>;
              })}
            </svg>
            {hoveredCommit && <div role="tooltip" style={{ position: "absolute", left: hoveredCommit.left, top: hoveredCommit.top, transform: "translate(-50%, -100%)", width: "min(300px, calc(100% - 24px))", padding: "9px 10px", borderRadius: 8, background: "#1a1622", border: `1px solid ${hoveredCommit.color}`, boxShadow: "0 8px 24px rgba(0,0,0,.35)", pointerEvents: "none", zIndex: 2 }}><div style={{ color: "var(--text-primary)", fontSize: "var(--text-sm)", fontWeight: 650, lineHeight: 1.35 }}>{hoveredCommit.commit.message || "Untitled commit"}</div><div style={{ marginTop: 4, color: "var(--text-tertiary)", fontSize: "var(--text-xs)", fontFamily: "var(--font-mono)" }}>{hoveredCommit.branch} · {hoveredCommit.commit.sha.slice(0, 12)}{hoveredCommit.commit.author ? ` · ${hoveredCommit.commit.author}` : ""}</div></div>}
          </div>
          {dashboard.branches.length > branches.length && <div style={{ marginTop: 8, fontSize: "var(--text-xs)", color: "var(--text-muted)" }}>Showing the first {branches.length} branches in the graph; {dashboard.branches.length - branches.length} additional branches are not graphed.</div>}
        </>
      )}
    </div>
  );
}

function shortBranchName(name: string): string {
  return name.length > 16 ? `${name.slice(0, 14)}…` : name;
}

*/
const BRANCH_GRAPH_COLORS = ["#67e8f9", "#f87171", "#c084fc", "#facc15", "#4ade80", "#fb923c", "#60a5fa"];

function getBranchLegendColor(branchName: string, index: number, currentBranch: string, defaultBranch: string | null): string {
  if (branchName === currentBranch || branchName === defaultBranch) return BRANCH_GRAPH_COLORS[0];
  return BRANCH_GRAPH_COLORS[(index + 1) % BRANCH_GRAPH_COLORS.length];
}

function BranchGraphCard({ dashboard, currentBranch }: { dashboard: GitHubDashboard | null; currentBranch: string | null }) {
  const graphContainerRef = useRef<HTMLDivElement>(null);
  const [resolvedBranchColors, setResolvedBranchColors] = useState<Record<string, string>>({});
  const branches = dashboard ? [...dashboard.branches].sort((left, right) => left.name.localeCompare(right.name)).slice(0, 24) : [];
  const branchSignature = branches.map((branch) => `${branch.name}:${branch.sha}`).join("|");

  useLayoutEffect(() => {
    const container = graphContainerRef.current;
    if (!container || !dashboard) return;

    const colors: Record<string, string> = {};
    const effectBranches = [...dashboard.branches].sort((left, right) => left.name.localeCompare(right.name)).slice(0, 24);
    for (const branch of effectBranches) {
      const badge = [...container.querySelectorAll<HTMLElement>("[style]")].find(
        (element) => element.textContent?.trim() === branch.name && element.style.borderColor,
      );
      if (badge?.style.borderColor) colors[branch.name] = badge.style.borderColor;
    }
    if (Object.keys(colors).length > 0) {
      const frame = window.requestAnimationFrame(() => setResolvedBranchColors(colors));
      return () => window.cancelAnimationFrame(frame);
    }
  }, [branchSignature, dashboard]);

  if (!dashboard) {
    return <div className="repository-card" style={branchCardStyle}><BranchGraphHeading count={null} /><div style={branchEmptyStyle}>No remote branch data available.</div></div>;
  }

  const entries = buildBranchGraphEntries(branches);
  const graphBranch = currentBranch && branches.some((branch) => branch.name === currentBranch)
    ? currentBranch
    : dashboard.default_branch ?? branches[0]?.name ?? "";
  return (
    <div className="repository-card repository-branch-card" style={branchCardStyle}>
      <BranchGraphHeading count={dashboard.branches.length} />
      <p style={{ margin: "0 0 12px", fontSize: "var(--text-sm)", color: "var(--text-tertiary)" }}>Remote branch history, read-only. The graph is calculated from real commit parent SHAs.</p>
      <div aria-label="Branch color legend" style={{ display: "flex", flexWrap: "wrap", gap: "6px 14px", marginBottom: 14 }}>
        {branches.map((branch, index) => (
          <span key={branch.name} style={{ display: "inline-flex", alignItems: "center", gap: 6, color: "var(--text-secondary)", fontSize: "var(--text-xs)" }}>
            <span aria-hidden="true" style={{ width: 9, height: 9, borderRadius: "50%", background: resolvedBranchColors[branch.name] ?? getBranchLegendColor(branch.name, index, graphBranch, dashboard.default_branch), boxShadow: `0 0 0 2px ${(resolvedBranchColors[branch.name] ?? getBranchLegendColor(branch.name, index, graphBranch, dashboard.default_branch))}33` }} />
            <span className="repository-ellipsis" title={branch.name}>{branch.name}</span>
          </span>
        ))}
      </div>
      {dashboard.branches_error && <div role="alert" style={branchEmptyStyle}>Remote branch history is unavailable. Other GitHub data remains available.</div>}
      {!dashboard.branches_error && branches.length === 0 && <div style={branchEmptyStyle}>No remote branches found.</div>}
      {!dashboard.branches_error && branches.length > 0 && (
        <>
          <div ref={graphContainerRef} aria-label="Remote branch commit graph" style={{ overflowX: "auto", border: "1px solid var(--border-primary)", borderRadius: 9, background: "#11151d", padding: 12 }}>
            <CommitGraph
              commits={entries.commits}
              branchHeads={entries.branchHeads}
              currentBranch={graphBranch}
              fullSha={false}
              graphStyle={{
                commitSpacing: 58,
                branchSpacing: 22,
                nodeRadius: 5,
                branchColors: BRANCH_GRAPH_COLORS,
              }}
            />
          </div>
          {dashboard.branches.length > branches.length && <div style={{ marginTop: 8, fontSize: "var(--text-xs)", color: "var(--text-muted)" }}>Showing the first {branches.length} branches in the graph; {dashboard.branches.length - branches.length} additional branches are not graphed.</div>}
        </>
      )}
    </div>
  );
}

type CommitGraphCommit = {
  sha: string;
  commit: { author: { name: string; date: string }; message: string };
  parents: Array<{ sha: string }>;
  html_url?: string;
};

type CommitGraphBranch = {
  name: string;
  commit: { sha: string };
  link?: string;
};

function buildBranchGraphEntries(branches: GitHubDashboard["branches"]): { commits: CommitGraphCommit[]; branchHeads: CommitGraphBranch[] } {
  const commits = new Map<string, CommitGraphCommit>();

  for (const branch of branches) {
    for (const commit of branch.commits) {
      if (commits.has(commit.sha)) continue;
      commits.set(commit.sha, {
        sha: commit.sha,
        commit: {
          author: { name: commit.author ?? "Unknown", date: commit.date ?? "" },
          message: commit.message,
        },
        parents: commit.parents.map((parent) => ({ sha: parent })),
      });
    }
  }

  return {
    commits: [...commits.values()].sort((left, right) => Date.parse(right.commit.author.date) - Date.parse(left.commit.author.date)),
    branchHeads: branches.map((branch) => ({
      name: branch.name,
      commit: { sha: branch.sha },
    })),
  };
}

const branchCardStyle = {
  background: "var(--bg-tertiary)",
  border: "1px solid var(--border-secondary)",
  borderRadius: 12,
  padding: 18,
};

const branchEmptyStyle = {
  padding: "20px 0",
  textAlign: "center" as const,
  color: "var(--text-tertiary)",
  fontSize: "var(--text-md)",
};

function BranchGraphHeading({ count }: { count: number | null }) {
  return (
    <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 8 }}>
      <h2 style={{ fontSize: "var(--text-lg)", fontWeight: 700, margin: 0, color: "var(--text-primary)", display: "flex", alignItems: "center", gap: 8 }}>
        <i className="material-symbols-outlined" style={{ fontSize: 18, color: "var(--text-tertiary)" }}>account_tree</i>
        GitHub Branch Graph
      </h2>
      {count !== null && <span style={{ fontSize: "var(--text-xs)", color: "var(--text-tertiary)" }}>{count} remote</span>}
    </div>
  );
}
