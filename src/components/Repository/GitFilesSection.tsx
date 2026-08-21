import type { GitDetails, GitFile, GitWorktree } from "../../types";

export interface GitFilesSectionProps {
  gitDetails: GitDetails | null;
  worktrees: GitWorktree[];
  effectiveScope: string;
  onScopeChange: (scope: string) => void;
  onSelectFile: (file: GitFile) => void;
}

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

export function GitFilesSection({
  gitDetails,
  worktrees,
  effectiveScope,
  onScopeChange,
  onSelectFile,
}: GitFilesSectionProps) {
  const scopedWorktree = worktrees.find((worktree) => worktree.name === effectiveScope) ?? null;
  const scopedFiles = scopedWorktree
    ? scopedWorktree.details?.files ?? []
    : gitDetails?.files ?? [];

  return (
    <div
      className="repository-card"
      style={{
        background: "var(--bg-tertiary)",
        border: "1px solid var(--border-secondary)",
        borderRadius: 12,
        padding: 18,
        flex: 1,
      }}
    >
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 16 }}>
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
            edit_document
          </i>
          Changed Files
        </h2>
        <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
          {worktrees.length > 0 && (
            <select
              className="app-select"
              aria-label="Changed files scope"
              value={effectiveScope}
              onChange={(event) => onScopeChange(event.target.value)}
              style={{
                height: 26,
                background: "var(--bg-primary)",
                color: "var(--text-secondary)",
                border: "1px solid var(--border-primary)",
                borderRadius: 5,
                fontSize: "var(--text-xs)",
                padding: "0 6px",
              }}
            >
              <option value="">main worktree</option>
              {worktrees
                .filter((worktree) => !worktree.prunable)
                .map((worktree) => (
                  <option key={worktree.name} value={worktree.name}>
                    {worktree.branch ?? worktree.name}
                  </option>
                ))}
            </select>
          )}
          {gitDetails && (
            <span
              style={{
                fontSize: "var(--text-xs)",
                color: "var(--text-tertiary)",
                background: "var(--bg-primary)",
                padding: "2px 6px",
                borderRadius: 5,
                border: "1px solid var(--border-primary)",
              }}
            >
              {scopedFiles.length} changes
            </span>
          )}
        </div>
      </div>

      <div className="repository-file-list">
        {gitDetails && scopedFiles.length === 0 && (
          <div
            style={{
              textAlign: "center",
              padding: "40px 0",
              color: "var(--text-tertiary)",
              fontSize: "var(--text-md)",
            }}
          >
            <i
              className="material-symbols-outlined"
              style={{ fontSize: 32, color: "#46b07d", marginBottom: 8, display: "block" }}
            >
              check_circle
            </i>
            Working directory clean.
          </div>
        )}

        {gitDetails &&
          scopedFiles.map((file) => (
            <button
              type="button"
              key={`${effectiveScope}:${file.diff_target}:${file.path}`}
              className="repository-file-row"
              aria-label={`View diff for ${file.path}, status ${file.status}, ${file.diff_target}`}
              onClick={() => onSelectFile(file)}
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
                  <div
                    className="repository-ellipsis"
                    title={file.original_path}
                    style={{ fontSize: "var(--text-2xs)", color: "var(--text-muted)", fontFamily: "var(--font-mono)" }}
                  >
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
  );
}
