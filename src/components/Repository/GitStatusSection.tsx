import type { GitDetails, GitWorktree } from "../../types";

export interface GitStatusSectionProps {
  gitDetails: GitDetails | null;
  worktrees: GitWorktree[];
}

export function GitStatusSection({ gitDetails, worktrees }: GitStatusSectionProps) {
  return (
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
          margin: "0 0 16px",
          color: "var(--text-primary)",
          display: "flex",
          alignItems: "center",
          gap: 8,
        }}
      >
        <i className="material-symbols-outlined" style={{ fontSize: 18, color: "var(--text-tertiary)" }}>
          schema
        </i>
        Local Git Status
      </h2>

      {gitDetails ? (
        <div style={{ display: "flex", flexDirection: "column", gap: 14 }}>
          <div className="repository-metadata-grid">
            {/* Branch */}
            <div
              style={{
                background: "var(--bg-primary)",
                padding: "10px 14px",
                borderRadius: 8,
                border: "1px solid var(--border-primary)",
              }}
            >
              <div
                style={{
                  fontSize: "var(--text-2xs)",
                  textTransform: "uppercase",
                  letterSpacing: ".06em",
                  color: "var(--text-muted)",
                  marginBottom: 4,
                }}
              >
                Current Branch
              </div>
              <div
                className="repository-ellipsis"
                title={gitDetails.branch}
                style={{
                  fontSize: "var(--text-md)",
                  fontWeight: 700,
                  color: "#bcaef6",
                  fontFamily: "var(--font-mono)",
                }}
              >
                {gitDetails.branch}
              </div>
            </div>

            {/* Commit */}
            <div
              style={{
                background: "var(--bg-primary)",
                padding: "10px 14px",
                borderRadius: 8,
                border: "1px solid var(--border-primary)",
              }}
            >
              <div
                style={{
                  fontSize: "var(--text-2xs)",
                  textTransform: "uppercase",
                  letterSpacing: ".06em",
                  color: "var(--text-muted)",
                  marginBottom: 4,
                }}
              >
                Active Commit
              </div>
              <div
                style={{
                  fontSize: "var(--text-md)",
                  fontWeight: 700,
                  color: "var(--text-secondary)",
                  fontFamily: "var(--font-mono)",
                }}
              >
                {gitDetails.current_commit}
              </div>
            </div>
          </div>

          {/* Sync State info */}
          <div
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "space-between",
              background: "var(--bg-primary)",
              padding: "10px 14px",
              borderRadius: 8,
              border: "1px solid var(--border-primary)",
            }}
          >
            <span style={{ fontSize: "var(--text-sm)", color: "var(--text-secondary)" }}>
              Tracking Branch Status
            </span>
            {gitDetails.ahead === 0 && gitDetails.behind === 0 ? (
              <span
                style={{
                  fontSize: "var(--text-xs)",
                  fontWeight: 600,
                  color: "var(--text-muted)",
                  background: "rgba(255,255,255,.05)",
                  padding: "3px 8px",
                  borderRadius: 5,
                }}
              >
                In sync with remote
              </span>
            ) : (
              <div style={{ display: "flex", gap: 8 }}>
                {gitDetails.ahead > 0 && (
                  <span
                    style={{
                      fontSize: "var(--text-xs)",
                      fontWeight: 700,
                      color: "#10b981",
                      background: "rgba(16,185,129,.1)",
                      padding: "3px 8px",
                      borderRadius: 5,
                    }}
                  >
                    ↑ {gitDetails.ahead} ahead
                  </span>
                )}
                {gitDetails.behind > 0 && (
                  <span
                    style={{
                      fontSize: "var(--text-xs)",
                      fontWeight: 700,
                      color: "#ef4444",
                      background: "rgba(239,68,68,.1)",
                      padding: "3px 8px",
                      borderRadius: 5,
                    }}
                  >
                    ↓ {gitDetails.behind} behind
                  </span>
                )}
              </div>
            )}
          </div>

          {/* Remote URL info */}
          {gitDetails.remote_url && (
            <div
              className="repository-remote repository-ellipsis"
              title={gitDetails.remote_url}
              style={{ fontSize: "var(--text-sm)", color: "var(--text-tertiary)" }}
            >
              Remote:{" "}
              <span style={{ fontFamily: "var(--font-mono)", color: "var(--text-secondary)" }}>
                {gitDetails.remote_url}
              </span>
            </div>
          )}

          {/* Linked worktrees */}
          {worktrees.length > 0 && (
            <div>
              <div
                style={{
                  fontSize: "var(--text-2xs)",
                  textTransform: "uppercase",
                  letterSpacing: ".06em",
                  color: "var(--text-muted)",
                  margin: "4px 0 8px",
                }}
              >
                Worktrees · {worktrees.length}
              </div>
              <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
                {worktrees.map((worktree) => (
                  <div
                    key={worktree.name}
                    style={{
                      background: "var(--bg-primary)",
                      padding: "10px 14px",
                      borderRadius: 8,
                      border: "1px solid var(--border-primary)",
                    }}
                  >
                    <div style={{ display: "flex", alignItems: "center", gap: 8, flexWrap: "wrap" }}>
                      <span
                        className="repository-ellipsis"
                        title={worktree.path}
                        style={{
                          fontSize: "var(--text-sm)",
                          fontWeight: 700,
                          color: "#bcaef6",
                          fontFamily: "var(--font-mono)",
                        }}
                      >
                        {worktree.branch ?? worktree.name}
                      </span>
                      {worktree.head && (
                        <span
                          style={{
                            fontSize: "var(--text-xs)",
                            color: "var(--text-secondary)",
                            fontFamily: "var(--font-mono)",
                          }}
                        >
                          {worktree.head}
                        </span>
                      )}
                      {worktree.prunable && (
                        <span
                          style={{
                            fontSize: "var(--text-2xs)",
                            fontWeight: 700,
                            color: "#ef4444",
                            background: "rgba(239,68,68,.1)",
                            padding: "2px 6px",
                            borderRadius: 4,
                          }}
                        >
                          PRUNABLE
                        </span>
                      )}
                      {worktree.locked && (
                        <span
                          style={{
                            fontSize: "var(--text-2xs)",
                            fontWeight: 700,
                            color: "#f59e0b",
                            background: "rgba(245,158,11,.1)",
                            padding: "2px 6px",
                            borderRadius: 4,
                          }}
                        >
                          LOCKED
                        </span>
                      )}
                      {worktree.details && (
                        <span style={{ marginLeft: "auto", fontSize: "var(--text-xs)", color: "var(--text-tertiary)" }}>
                          {worktree.details.files.length} changes
                        </span>
                      )}
                    </div>
                    <div
                      className="repository-ellipsis"
                      title={worktree.path}
                      style={{
                        marginTop: 3,
                        fontSize: "var(--text-2xs)",
                        color: "var(--text-muted)",
                        fontFamily: "var(--font-mono)",
                      }}
                    >
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
  );
}
