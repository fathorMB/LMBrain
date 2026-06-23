import { useWorkspace } from "../../hooks/useWorkspace";

export function TopBar() {
  const { state, toggleCmdk } = useWorkspace();
  const { gitInfo, watcherActive } = state;

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
