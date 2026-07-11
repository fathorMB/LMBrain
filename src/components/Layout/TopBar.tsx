import { useEffect, useRef, useState } from "react";
import { useWorkspace } from "../../hooks/useWorkspace";

export function TopBar({ onViewReload }: { onViewReload: () => void }) {
  const { state, toggleCmdk, refreshWorkspaceData, refreshSessions } = useWorkspace();
  const { gitInfo, watcherActive } = state;
  const [refreshStatus, setRefreshStatus] = useState<"idle" | "refreshing" | "success" | "error">("idle");
  const feedbackTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(
    () => () => {
      if (feedbackTimeoutRef.current) clearTimeout(feedbackTimeoutRef.current);
    },
    []
  );

  const handleRefresh = async () => {
    if (refreshStatus === "refreshing") return;
    if (feedbackTimeoutRef.current) clearTimeout(feedbackTimeoutRef.current);
    setRefreshStatus("refreshing");
    try {
      await refreshWorkspaceData();
      if (state.view === "sessions") {
        await refreshSessions();
      } else {
        onViewReload();
      }
      setRefreshStatus("success");
      feedbackTimeoutRef.current = setTimeout(() => setRefreshStatus("idle"), 1800);
    } catch (error) {
      console.error("Failed to refresh current view:", error);
      setRefreshStatus("error");
    }
  };

  return (
    <div
      style={{
        height: 53,
        flex: "none",
        borderBottom: "1px solid var(--border-primary)",
        background: "#0e0c12",
        display: "flex",
        alignItems: "center",
        gap: 14,
        padding: "0 18px",
      }}
    >
      {/* Workspace name */}
      <div style={{ display: "flex", alignItems: "center", gap: 9 }}>
        <span style={{ fontSize: 13.5, fontWeight: 600 }}>
          {state.currentWorkspace?.name || "LMBrain"}
        </span>
        {gitInfo?.branch && (
          <span
            style={{
              display: "inline-flex",
              alignItems: "center",
              gap: 5,
              fontFamily: "var(--font-mono)",
              fontSize: 11.5,
              color: "var(--accent-light)",
              background: "rgba(124,108,246,.1)",
              border: "1px solid rgba(124,108,246,.22)",
              borderRadius: 6,
              padding: "2px 8px",
            }}
          >
            <i
              className="material-symbols-outlined"
              style={{ fontSize: 14 }}
            >
              graph_3
            </i>
            {gitInfo.branch}
          </span>
        )}
      </div>

      <div
        style={{
          width: 1,
          height: 20,
          background: "#26222d",
        }}
      />

      {/* Watcher status */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: 7,
          fontSize: 12,
          color: "var(--text-tertiary)",
        }}
      >
        <span
          style={{
            width: 7,
            height: 7,
            borderRadius: "50%",
            background: watcherActive ? "var(--green)" : "var(--text-muted)",
            boxShadow: watcherActive
              ? "0 0 7px var(--green)"
              : "none",
            animation: watcherActive
              ? "lmpulse 2.4s ease-in-out infinite"
              : "none",
          }}
        />
        File watcher {watcherActive ? "active" : "inactive"}
      </div>

      <div style={{ flex: 1 }} />

      {refreshStatus === "success" && (
        <span role="status" style={{ fontSize: 11, color: "var(--green)" }}>
          Updated
        </span>
      )}
      {refreshStatus === "error" && (
        <span role="alert" style={{ fontSize: 11, color: "var(--red)" }}>
          Refresh failed
        </span>
      )}

      <button
        type="button"
        aria-label={refreshStatus === "error" ? "Refresh failed. Retry current view" : "Refresh current view"}
        title="Refresh current view"
        disabled={refreshStatus === "refreshing"}
        onClick={() => void handleRefresh()}
        style={{
          width: 34,
          height: 34,
          borderRadius: 9,
          background: "#141118",
          border: "1px solid #262330",
          color: refreshStatus === "error" ? "var(--red)" : "var(--text-secondary)",
          cursor: refreshStatus === "refreshing" ? "wait" : "pointer",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          opacity: refreshStatus === "refreshing" ? 0.7 : 1,
        }}
      >
        <i
          className={`material-symbols-outlined${refreshStatus === "refreshing" ? " lmbrain-loading-spinner" : ""}`}
          aria-hidden="true"
          style={{ fontSize: 18 }}
        >
          refresh
        </i>
      </button>

      {/* Search bar */}
      <div
        onClick={toggleCmdk}
        style={{
          display: "flex",
          alignItems: "center",
          gap: 9,
          width: 264,
          background: "#141118",
          border: "1px solid #262330",
          borderRadius: 9,
          padding: "7px 11px",
          cursor: "text",
          color: "var(--text-tertiary)",
        }}
        onMouseEnter={(e) => {
          e.currentTarget.style.borderColor = "#36303f";
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.borderColor = "#262330";
        }}
      >
        <i className="material-symbols-outlined" style={{ fontSize: 17 }}>
          search
        </i>
        <span style={{ flex: 1, fontSize: 12.5 }}>
          Search specs, files…
        </span>
        <span
          style={{
            fontFamily: "var(--font-mono)",
            fontSize: 10,
            border: "1px solid #2b2833",
            borderRadius: 5,
            padding: "1px 5px",
          }}
        >
          ⌘K
        </span>
      </div>

      <button
        style={{
          width: 34,
          height: 34,
          borderRadius: 9,
          background: "#141118",
          border: "1px solid #262330",
          color: "var(--text-secondary)",
          cursor: "pointer",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
        }}
        onMouseEnter={(e) => {
          e.currentTarget.style.borderColor = "#36303f";
          e.currentTarget.style.color = "var(--text-primary)";
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.borderColor = "#262330";
          e.currentTarget.style.color = "var(--text-secondary)";
        }}
      >
        <i className="material-symbols-outlined" style={{ fontSize: 19 }}>
          notifications
        </i>
      </button>
    </div>
  );
}
