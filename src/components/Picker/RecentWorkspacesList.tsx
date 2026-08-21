import type { WorkspaceSummary } from "../../types";

export interface RecentWorkspacesListProps {
  recentItems: WorkspaceSummary[];
  onOpenRecent: (path: string) => void;
}

const healthDot = (health: string) => {
  switch (health) {
    case "ok":
      return "#46b07d";
    case "warn":
      return "#e0a23a";
    default:
      return "var(--text-tertiary)";
  }
};

export function RecentWorkspacesList({
  recentItems,
  onOpenRecent,
}: RecentWorkspacesListProps) {
  return (
    <div>
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          margin: "26px 0 11px",
        }}
      >
        <span
          style={{
            fontSize: "var(--text-xs)",
            letterSpacing: ".09em",
            textTransform: "uppercase",
            color: "var(--text-tertiary)",
            fontWeight: 600,
          }}
        >
          Recent
        </span>
        <span
          style={{
            fontFamily: "var(--font-mono)",
            fontSize: "var(--text-xs)",
            color: "var(--text-muted)",
          }}
        >
          {recentItems.length} workspaces
        </span>
      </div>

      <div style={{ display: "flex", flexDirection: "column", gap: 3 }}>
        {recentItems.map((item) => (
          <button
            type="button"
            key={item.path}
            onClick={() => onOpenRecent(item.path)}
            aria-label={`Open recent workspace ${item.name} at ${item.path}`}
            style={{
              width: "100%",
              display: "flex",
              alignItems: "center",
              gap: 11,
              padding: "9px 10px",
              borderRadius: 9,
              cursor: "pointer",
              border: "1px solid transparent",
              background: "transparent",
              textAlign: "left",
              fontFamily: "inherit",
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.background = "#1b1822";
              e.currentTarget.style.borderColor = "#2b2833";
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.background = "transparent";
              e.currentTarget.style.borderColor = "transparent";
            }}
          >
            <div
              style={{
                width: 8,
                height: 8,
                borderRadius: "50%",
                flex: "none",
                background: healthDot(item.health),
                boxShadow: `0 0 9px ${healthDot(item.health)}`,
              }}
            />
            <div style={{ minWidth: 0, flex: 1 }}>
              <div
                style={{
                  fontSize: "var(--text-md)",
                  fontWeight: 600,
                  color: "var(--text-primary)",
                }}
              >
                {item.name}
              </div>
              <div
                style={{
                  fontFamily: "var(--font-mono)",
                  fontSize: "var(--text-xs)",
                  color: "var(--text-tertiary)",
                  whiteSpace: "nowrap",
                  overflow: "hidden",
                  textOverflow: "ellipsis",
                }}
              >
                {item.path}
              </div>
            </div>
            <div
              style={{
                fontSize: "var(--text-xs)",
                color: "var(--text-tertiary)",
                whiteSpace: "nowrap",
              }}
            >
              {item.last_opened}
            </div>
          </button>
        ))}
      </div>
    </div>
  );
}
