import { useWorkspace } from "../../hooks/useWorkspace";
import type { AppView } from "../../types";

interface NavItem {
  key: AppView;
  icon: string;
  label: string;
  badge: string | null;
}

const NAV_ITEMS: NavItem[] = [
  { key: "pulse", icon: "monitoring", label: "Project Pulse", badge: null },
  { key: "sessions", icon: "terminal", label: "Sessions", badge: null },
  { key: "wiki", icon: "menu_book", label: "Wiki", badge: null },
  { key: "taskboard", icon: "view_kanban", label: "Board", badge: null },
  { key: "roadmap", icon: "flag", label: "Roadmap", badge: null },
  { key: "insights", icon: "query_stats", label: "Insights", badge: null },
  { key: "reviews", icon: "rate_review", label: "Reviews", badge: null },
  { key: "decisions", icon: "account_balance", label: "Decisions", badge: null },
  { key: "design", icon: "design_services", label: "Design", badge: null },
  { key: "agents", icon: "smart_toy", label: "Agents", badge: null },
  { key: "mcp", icon: "integration_instructions", label: "MCP", badge: null },
  { key: "repository", icon: "schema", label: "Repository", badge: null },
  { key: "skills", icon: "psychology_alt", label: "Skills", badge: null },
];

export function Sidebar() {
  const { state, navigateTo, triggerLeaveWorkspace, toggleCmdk } = useWorkspace();

  return (
    <div
      style={{
        width: 236,
        flex: "none",
        background: "var(--bg-secondary)",
        borderRight: "1px solid var(--border-primary)",
        display: "flex",
        flexDirection: "column",
        padding: "11px 11px 9px",
      }}
    >

      <div
        style={{
          fontSize: 10,
          letterSpacing: ".1em",
          textTransform: "uppercase",
          color: "var(--text-muted)",
          fontWeight: 600,
          padding: "0 9px 7px",
        }}
      >
        Workspace
      </div>

      <div style={{ display: "flex", flexDirection: "column", gap: 2 }}>
        {NAV_ITEMS.map((item) => {
          const active = state.view === item.key;
          return (
            <div
              key={item.key}
              onClick={() => navigateTo(item.key)}
              style={{
                display: "flex",
                alignItems: "center",
                gap: 11,
                padding: "7px 10px",
                borderRadius: 8,
                fontSize: 13,
                fontWeight: active ? 600 : 500,
                cursor: "pointer",
                color: active ? "var(--text-primary)" : "var(--text-secondary)",
                background: active ? "var(--bg-active)" : "transparent",
              }}
              onMouseEnter={(e) => {
                if (!active)
                  e.currentTarget.style.background = "var(--bg-hover)";
              }}
              onMouseLeave={(e) => {
                if (!active) e.currentTarget.style.background = "transparent";
              }}
            >
              <i
                className="material-symbols-outlined"
                style={{
                  fontSize: 19,
                  fontVariationSettings: active ? "'wght' 400" : "'wght' 300",
                  color: active ? "var(--accent-light)" : "var(--text-tertiary)",
                }}
              >
                {item.icon}
              </i>
              <span style={{ flex: 1 }}>{item.label}</span>
              {item.badge && (
                <span
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: 10,
                    color: "var(--text-tertiary)",
                    background: "#1a1722",
                    border: "1px solid #2b2833",
                    borderRadius: 5,
                    padding: "1px 6px",
                  }}
                >
                  {item.badge}
                </span>
              )}
            </div>
          );
        })}
      </div>

      <div style={{ flex: 1 }} />

      {/* Search */}
      <div
        onClick={toggleCmdk}
        style={{
          display: "flex",
          alignItems: "center",
          gap: 9,
          padding: "8px 10px",
          borderRadius: 9,
          cursor: "pointer",
          color: "var(--text-secondary)",
          fontSize: 13,
          fontWeight: 500,
        }}
        onMouseEnter={(e) => {
          e.currentTarget.style.background = "var(--bg-hover)";
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.background = "transparent";
        }}
      >
        <i
          className="material-symbols-outlined"
          style={{
            fontSize: 19,
            fontVariationSettings: "'wght' 300",
            color: "var(--text-tertiary)",
          }}
        >
          search
        </i>
        <span style={{ flex: 1 }}>Search</span>
        <span
          style={{
            fontFamily: "var(--font-mono)",
            fontSize: 10,
            color: "var(--text-tertiary)",
            border: "1px solid #2b2833",
            borderRadius: 5,
            padding: "1px 5px",
          }}
        >
          ⌘K
        </span>
      </div>

      {/* Settings */}
      <div
        onClick={() => navigateTo("settings")}
        style={{
          display: "flex",
          alignItems: "center",
          gap: 9,
          padding: "8px 10px",
          borderRadius: 9,
          cursor: "pointer",
          color: "var(--text-secondary)",
          fontSize: 13,
          fontWeight: 500,
        }}
        onMouseEnter={(e) => {
          e.currentTarget.style.background = "var(--bg-hover)";
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.background = "transparent";
        }}
      >
        <i
          className="material-symbols-outlined"
          style={{
            fontSize: 19,
            fontVariationSettings: "'wght' 300",
            color: "var(--text-tertiary)",
          }}
        >
          settings
        </i>
        Settings
      </div>

      {/* Leave Workspace */}
      <div
        onClick={triggerLeaveWorkspace}
        style={{
          display: "flex",
          alignItems: "center",
          gap: 9,
          padding: "8px 10px",
          borderRadius: 9,
          cursor: "pointer",
          color: "#f87171",
          fontSize: 13,
          fontWeight: 500,
          marginTop: 4,
        }}
        onMouseEnter={(e) => {
          e.currentTarget.style.background = "rgba(248, 113, 113, 0.08)";
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.background = "transparent";
        }}
      >
        <i
          className="material-symbols-outlined"
          style={{
            fontSize: 19,
            fontVariationSettings: "'wght' 300",
            color: "#f87171",
          }}
        >
          logout
        </i>
        Leave workspace
      </div>
    </div>
  );
}
