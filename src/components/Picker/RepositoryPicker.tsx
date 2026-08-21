import { useState, useEffect } from "react";
import { useWorkspace } from "../../hooks/useWorkspace";
import { listRecentWorkspaces } from "../../lib/commands";
import type { WorkspaceSummary } from "../../types";
import { WorkspacePreview } from "./WorkspacePreview";
import { RecentWorkspacesList } from "./RecentWorkspacesList";

export function RepositoryPicker() {
  const { state, openWorkspace } = useWorkspace();
  const [recentItems, setRecentItems] = useState<WorkspaceSummary[]>([]);

  useEffect(() => {
    listRecentWorkspaces().then(setRecentItems).catch(() => {});
  }, []);

  const handleOpenWorkspace = async () => {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Choose a repository folder",
      });
      if (selected) {
        await openWorkspace(selected as string);
      }
    } catch (err) {
      console.error("Failed to open folder picker:", err);
    }
  };

  const handleOpenRecent = async (path: string) => {
    await openWorkspace(path);
  };

  return (
    <div
      style={{
        height: "100vh",
        width: "100vw",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        padding: 36,
        background:
          "radial-gradient(1100px 620px at 26% -12%, #1b1624 0%, #0b0a0d 56%)",
      }}
    >
      <div style={{ width: 1010, maxWidth: "100%" }}>
        {/* Header */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: 12,
            marginBottom: 22,
          }}
        >
          <img
            src="/favicon.svg"
            alt=""
            aria-hidden="true"
            style={{
              width: 36,
              height: 36,
              borderRadius: 10,
              boxShadow: "0 6px 20px -4px rgba(106,79,240,.55)",
            }}
          />
          <div>
            <div
              style={{
                fontSize: "var(--text-lg)",
                fontWeight: 700,
                letterSpacing: "-.01em",
              }}
            >
              LMBrain
            </div>
            <div
              style={{
                fontSize: "var(--text-xs)",
                color: "var(--text-tertiary)",
                fontFamily: "var(--font-mono)",
              }}
            >
              local-first project brain
            </div>
          </div>
        </div>

        {/* Two-column card */}
        <div
          style={{
            display: "grid",
            gridTemplateColumns: "1fr 1.08fr",
            borderRadius: 18,
            border: "1px solid var(--border-primary)",
            background: "var(--bg-secondary)",
            boxShadow: "0 24px 80px -12px rgba(0,0,0,.6)",
            overflow: "hidden",
          }}
        >
          {/* Left: Actions & Recent */}
          <div
            style={{
              padding: "26px 24px",
              borderRight: "1px solid var(--border-primary)",
              background: "#100e14",
            }}
          >
            <div
              style={{
                fontSize: "var(--text-xl)",
                fontWeight: 700,
                letterSpacing: "-.02em",
                marginBottom: 6,
              }}
            >
              Open project brain
            </div>
            <p
              style={{
                fontSize: "var(--text-md)",
                lineHeight: 1.5,
                color: "var(--text-tertiary)",
                margin: "0 0 20px",
              }}
            >
              Open any existing repository to read its specifications and .lmbrain
              directory. Files are read in place — your repo is never copied or
              uploaded.
            </p>

            <button
              type="button"
              onClick={handleOpenWorkspace}
              style={{
                width: "100%",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                gap: 9,
                background: "linear-gradient(180deg,#8676f7,#6e5bf2)",
                color: "#fff",
                border: "none",
                borderRadius: 10,
                padding: 12,
                fontSize: "var(--text-md)",
                fontWeight: 600,
                cursor: "pointer",
                boxShadow: "0 8px 20px -7px rgba(110,91,242,.75)",
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.filter = "brightness(1.08)";
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.filter = "none";
              }}
            >
              <i className="material-symbols-outlined" style={{ fontSize: 19 }}>
                folder_open
              </i>
              Choose repository folder…
            </button>

            <RecentWorkspacesList
              recentItems={recentItems}
              onOpenRecent={handleOpenRecent}
            />
          </div>

          {/* Right: Workspace preview */}
          <div
            style={{
              padding: "26px 28px",
              display: "flex",
              flexDirection: "column",
              minHeight: 392,
            }}
          >
            {state.currentWorkspace ? (
              <WorkspacePreview />
            ) : (
              <div
                style={{
                  display: "flex",
                  flexDirection: "column",
                  height: "100%",
                  textAlign: "center",
                  alignItems: "center",
                  justifyContent: "center",
                }}
              >
                <div
                  style={{
                    width: 48,
                    height: 48,
                    borderRadius: 13,
                    background: "rgba(224,162,58,.12)",
                    border: "1px solid rgba(224,162,58,.28)",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    marginBottom: 15,
                  }}
                >
                  <i
                    className="material-symbols-outlined"
                    style={{ fontSize: 24, color: "#e0a23a" }}
                  >
                    folder_off
                  </i>
                </div>
                <h2
                  style={{
                    fontSize: "var(--text-xl)",
                    fontWeight: 700,
                    margin: "0 0 7px",
                    color: "var(--text-primary)",
                  }}
                >
                  Select a workspace
                </h2>
                <p
                  style={{
                    fontSize: "var(--text-md)",
                    lineHeight: 1.55,
                    color: "var(--text-secondary)",
                    margin: 0,
                    maxWidth: 300,
                  }}
                >
                  Choose a repository folder or select one from your recent
                  workspaces to get started.
                </p>
              </div>
            )}
          </div>
        </div>

        <div
          style={{
            textAlign: "center",
            marginTop: 16,
            fontSize: "var(--text-xs)",
            color: "var(--text-muted)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            gap: 7,
          }}
        >
          <i className="material-symbols-outlined" style={{ fontSize: 15 }}>
            lock
          </i>
          Everything stays on this machine · No account required · Markdown is
          the source of truth
        </div>
      </div>
      {state.loading && (
        <div
          role="status"
          aria-live="polite"
          aria-label="Preparing workspace"
          style={{
            position: "fixed",
            inset: 0,
            zIndex: 1000,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            background: "rgba(8, 7, 11, 0.78)",
            backdropFilter: "blur(8px)",
          }}
        >
          <div
            style={{
              width: 420,
              maxWidth: "calc(100vw - 48px)",
              padding: "28px 30px",
              borderRadius: 18,
              border: "1px solid #332d40",
              background: "#15121b",
              boxShadow: "0 30px 90px rgba(0,0,0,.55)",
              textAlign: "center",
            }}
          >
            <i
              className="material-symbols-outlined lmbrain-loading-spinner"
              style={{ fontSize: 34, color: "#8f7df8" }}
            >
              progress_activity
            </i>
            <div style={{ marginTop: 14, fontSize: "var(--text-xl)", fontWeight: 700 }}>
              Preparing project brain
            </div>
            <div style={{ marginTop: 8, color: "#aaa3b2", fontSize: "var(--text-md)" }}>
              {state.loadingMessage}
            </div>
            {state.loadingPath && (
              <div
                style={{
                  marginTop: 12,
                  color: "#706a76",
                  fontFamily: "var(--font-mono)",
                  fontSize: "var(--text-xs)",
                  whiteSpace: "nowrap",
                  overflow: "hidden",
                  textOverflow: "ellipsis",
                }}
                title={state.loadingPath}
              >
                {state.loadingPath}
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
