import { useEffect, useState } from "react";
import * as commands from "../../lib/commands";
import type { GitDetails, GitHubDashboard, GitFile, GitHubWorkflowRun, GitWorktree } from "../../types";
import { GitDiffModal } from "./GitDiffModal";
import { WorkflowRunModal } from "./WorkflowRunModal";
import { GitStatusSection } from "./GitStatusSection";
import { GitFilesSection } from "./GitFilesSection";
import { GitHubSection } from "./GitHubSection";
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
  const [hasToken, setHasToken] = useState(false);
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
    } catch (err) {
      setError(errorMessage(err, "Failed to load Git details"));
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
      .catch((err: unknown) => {
        if (active) setError(errorMessage(err, "Failed to load Git details"));
      })
      .finally(() => {
        if (active) setLoading(false);
      });

    return () => {
      active = false;
    };
  }, []);

  const scopedWorktree = worktrees.find((worktree) => worktree.name === fileScope) ?? null;
  const effectiveScope = scopedWorktree ? fileScope : "";

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

        {/* Main grid */}
        <div className="repository-grid">
          {/* Left Column: Local Git details */}
          <div className="repository-column" style={{ display: "flex", flexDirection: "column", gap: 24 }}>
            <GitStatusSection gitDetails={gitDetails} worktrees={worktrees} />
            <GitFilesSection
              gitDetails={gitDetails}
              worktrees={worktrees}
              effectiveScope={effectiveScope}
              onScopeChange={setFileScope}
              onSelectFile={setSelectedFile}
            />
          </div>

          {/* Right Column: GitHub integration details */}
          <GitHubSection
            githubDashboard={githubDashboard}
            hasToken={hasToken}
            onTokenChanged={loadData}
            onSelectRun={setSelectedRun}
          />
        </div>
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
