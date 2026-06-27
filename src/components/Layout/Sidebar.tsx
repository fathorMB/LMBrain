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
  { key: "reviews", icon: "rate_review", label: "Reviews", badge: null },
  { key: "decisions", icon: "account_balance", label: "Decisions", badge: null },
  { key: "agents", icon: "smart_toy", label: "Agents & MCP", badge: null },
];

export function Sidebar() {
  const { state, navigateTo, goToPicker, toggleCmdk } = useWorkspace();
  const ws = state.currentWorkspace;

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
      {/* Workspace switcher button */}
      <button
        onClick={goToPicker}
        title="Switch workspace"
        style={{
          display: "flex",
          alignItems: "center",
          gap: 10,
          padding: "8px 9px",
          borderRadius: 10,
          background: "#181520",
          border: "1px solid #272330",
          cursor: "pointer",
          textAlign: "left",
          width: "100%",
          color: "var(--text-primary)",
        }}
        onMouseEnter={(e) => {
          e.currentTarget.style.background = "#1e1a27";
          e.currentTarget.style.borderColor = "#352f42";
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.background = "#181520";
          e.currentTarget.style.borderColor = "#272330";
        }}
      >
        <div
          style={{
            width: 28,
            height: 28,
            borderRadius: 8,
            background: "linear-gradient(150deg,#9384f8,#6a4ff0)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            flex: "none",
            boxShadow: "0 3px 10px -3px rgba(106,79,240,.6)",
          }}
        >
          <i
            className="material-symbols-outlined"
            style={{ fontSize: 17, color: "#fff" }}
          >
            neurology
          </i>
        </div>
        <div style={{ flex: 1, minWidth: 0 }}>
          <div
            style={{
              fontSize: 13,
              fontWeight: 700,
              letterSpacing: "-.01em",
              color: "var(--text-primary)",
            }}
          >
            {ws?.name || "LMBrain"}
          </div>
          <div
            style={{
              fontSize: 10.5,
              color: "var(--text-tertiary)",
              fontFamily: "var(--font-mono)",
              whiteSpace: "nowrap",
              overflow: "hidden",
              textOverflow: "ellipsis",
            }}
          >
            {ws?.path || "No workspace"}
          </div>
        </div>
        <i
          className="material-symbols-outlined"
          style={{ fontSize: 18, color: "var(--text-tertiary)" }}
        >
          unfold_more
        </i>
      </button>

      <div
        style={{
          height: 1,
          background: "var(--border-primary)",
          margin: "11px 2px",
        }}
      />

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
    </div>
  );
}
