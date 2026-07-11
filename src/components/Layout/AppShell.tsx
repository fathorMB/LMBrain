import { useState } from "react";
import { Sidebar } from "./Sidebar";
import { TopBar } from "./TopBar";
import { useWorkspace } from "../../hooks/useWorkspace";
import { RepositoryPicker } from "../Picker/RepositoryPicker";
import { ProjectPulse } from "../Pulse/ProjectPulse";
import { WikiView } from "../Wiki/WikiView";
import { TaskboardView } from "../Taskboard/TaskboardView";
import { SpecDetail } from "../Spec/SpecDetail";
import { ReviewsList } from "../Reviews/ReviewsList";
import { DecisionsList } from "../Decisions/DecisionsList";
import { AgentsMCPView } from "../Agents/AgentsMCPView";
import { SkillsView } from "../Skills/SkillsView";
import { DesignView } from "../Design/DesignView";
import { SettingsView } from "../Settings/SettingsView";
import { RoadmapView } from "../Roadmap/RoadmapView";
import { InsightsView } from "../Insights/InsightsView";
import { SessionsView } from "../Sessions/SessionsView";
import { HarnessesView } from "../Harnesses/HarnessesView";
import { CommandPalette } from "../CommandPalette";
import { ArtifactDetailModal } from "./ArtifactDetailModal";

export function AppShell() {
  const { state, dispatch } = useWorkspace();
  const [viewRefreshRevision, setViewRefreshRevision] = useState(0);

  if (state.screen === "picker") {
    return <RepositoryPicker />;
  }

  const renderView = () => {
    switch (state.view) {
      case "pulse":
        return <ProjectPulse />;
      case "sessions":
        return null;
      case "harnesses":
        return <HarnessesView />;
      case "wiki":
        return <WikiView />;
      case "taskboard":
        return <TaskboardView />;
      case "spec":
        return <SpecDetail />;
      case "reviews":
        return <ReviewsList />;
      case "decisions":
        return <DecisionsList />;
      case "agents":
        return <AgentsMCPView />;
      case "skills":
        return <SkillsView />;
      case "design":
        return <DesignView />;
      case "settings":
        return <SettingsView />;
      case "roadmap":
        return <RoadmapView />;
      case "insights":
        return <InsightsView />;
      case "search":
        return <PlaceholderView />;
      default:
        return <ProjectPulse />;
    }
  };

  return (
    <div
      style={{
        height: "100vh",
        width: "100vw",
        display: "flex",
        background: "var(--bg-primary)",
      }}
    >
      <Sidebar />
      <div style={{ flex: 1, minWidth: 0, minHeight: 0, display: "flex", flexDirection: "column" }}>
        <TopBar onViewReload={() => setViewRefreshRevision((revision) => revision + 1)} />
        {state.workspaceNotice && (
          <div
            role="alert"
            style={{
              display: "flex",
              alignItems: "center",
              gap: 9,
              padding: "8px 14px",
              borderBottom: "1px solid rgba(224,162,58,.35)",
              background: "rgba(224,162,58,.10)",
              color: "#d9b86d",
              fontSize: 12,
              flexShrink: 0,
            }}
          >
            <i className="material-symbols-outlined" style={{ fontSize: 17 }}>
              warning
            </i>
            <span style={{ flex: 1 }}>{state.workspaceNotice}</span>
            <button
              type="button"
              aria-label="Dismiss workspace warning"
              onClick={() => dispatch({ type: "SET_WORKSPACE_NOTICE", notice: null })}
              style={{
                border: "none",
                background: "transparent",
                color: "inherit",
                cursor: "pointer",
                padding: 2,
              }}
            >
              <i className="material-symbols-outlined" style={{ fontSize: 16 }}>
                close
              </i>
            </button>
          </div>
        )}
        <div
          style={{
            flex: 1,
            minHeight: 0,
            background: "#0c0b0f",
            position: "relative",
          }}
        >
          <div
            key={`${state.view}-${viewRefreshRevision}`}
            style={{
              height: "100%",
              minHeight: 0,
              overflowY: state.view === "sessions" ? "hidden" : "auto",
              display: state.view === "sessions" ? "none" : "block",
            }}
          >
            {renderView()}
          </div>
          <div
            style={{
              position: "absolute",
              inset: 0,
              display: state.currentWorkspace && state.view === "sessions" ? "block" : "none",
            }}
          >
            <SessionsView active={state.view === "sessions"} />
          </div>
        </div>
      </div>

      {/* Command Palette */}
      {state.cmdkOpen && <CommandPalette />}

      {/* Artifact Detail Modal */}
      {state.detailArtifact && <ArtifactDetailModal key={state.detailArtifact.path} />}
    </div>
  );
}

function PlaceholderView() {
  const { state } = useWorkspace();
  const titles: Record<string, string> = {
    roadmap: "Roadmap",
    search: "Search",
  };
  const icons: Record<string, string> = {
    roadmap: "flag",
    search: "search",
  };

  return (
    <div
      style={{
        height: "100%",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
      }}
    >
      <div style={{ textAlign: "center", maxWidth: 340 }}>
        <div
          style={{
            width: 52,
            height: 52,
            borderRadius: 14,
            background: "var(--bg-tertiary)",
            border: "1px solid #262330",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            margin: "0 auto 16px",
          }}
        >
          <i
            className="material-symbols-outlined"
            style={{ fontSize: 26, color: "var(--text-tertiary)" }}
          >
            {icons[state.view] || "widgets"}
          </i>
        </div>
        <h2
          style={{
            fontSize: 19,
            fontWeight: 700,
            margin: "0 0 7px",
            color: "var(--text-primary)",
          }}
        >
          {titles[state.view] || "Coming soon"}
        </h2>
        <p
          style={{
            fontSize: 13,
            color: "var(--text-tertiary)",
            lineHeight: 1.55,
            margin: 0,
          }}
        >
          This area is part of the LMBrain workspace. The five primary views —
          Pulse, Wiki, Taskboard, Reviews and Spec detail — are fully built out.
        </p>
      </div>
    </div>
  );
}
