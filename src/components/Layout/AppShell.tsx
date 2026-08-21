import { lazy, Suspense, useState } from "react";
import { Sidebar } from "./Sidebar";
import { TopBar } from "./TopBar";
import { useWorkspace } from "../../hooks/useWorkspace";

const RepositoryPicker = lazy(() =>
  import("../Picker/RepositoryPicker").then((m) => ({ default: m.RepositoryPicker }))
);
const ProjectPulse = lazy(() =>
  import("../Pulse/ProjectPulse").then((m) => ({ default: m.ProjectPulse }))
);
const WikiView = lazy(() =>
  import("../Wiki/WikiView").then((m) => ({ default: m.WikiView }))
);
const TaskboardView = lazy(() =>
  import("../Taskboard/TaskboardView").then((m) => ({ default: m.TaskboardView }))
);
const SpecDetail = lazy(() =>
  import("../Spec/SpecDetail").then((m) => ({ default: m.SpecDetail }))
);
const ReviewsList = lazy(() =>
  import("../Reviews/ReviewsList").then((m) => ({ default: m.ReviewsList }))
);
const OperationsView = lazy(() =>
  import("../Operations/OperationsView").then((m) => ({ default: m.OperationsView }))
);
const DebtsView = lazy(() =>
  import("../Debts/DebtsView").then((m) => ({ default: m.DebtsView }))
);
const DreamsView = lazy(() =>
  import("../Dreams/DreamsView").then((m) => ({ default: m.DreamsView }))
);
const FeedbackView = lazy(() =>
  import("../Feedback/FeedbackView").then((m) => ({ default: m.FeedbackView }))
);
const DecisionsList = lazy(() =>
  import("../Decisions/DecisionsList").then((m) => ({ default: m.DecisionsList }))
);
const AgentsView = lazy(() =>
  import("../Agents/AgentsView").then((m) => ({ default: m.AgentsView }))
);
const McpView = lazy(() =>
  import("../Agents/McpView").then((m) => ({ default: m.McpView }))
);
const RepositoryView = lazy(() =>
  import("../Repository/RepositoryView").then((m) => ({ default: m.RepositoryView }))
);
const EnvironmentView = lazy(() =>
  import("../Environment/EnvironmentView").then((m) => ({ default: m.EnvironmentView }))
);
const SkillsView = lazy(() =>
  import("../Skills/SkillsView").then((m) => ({ default: m.SkillsView }))
);
const DesignView = lazy(() =>
  import("../Design/DesignView").then((m) => ({ default: m.DesignView }))
);
const SettingsView = lazy(() =>
  import("../Settings/SettingsView").then((m) => ({ default: m.SettingsView }))
);
const RoadmapView = lazy(() =>
  import("../Roadmap/RoadmapView").then((m) => ({ default: m.RoadmapView }))
);
const InsightsView = lazy(() =>
  import("../Insights/InsightsView").then((m) => ({ default: m.InsightsView }))
);
const SessionsView = lazy(() =>
  import("../Sessions/SessionsView").then((m) => ({ default: m.SessionsView }))
);
const CommandPalette = lazy(() =>
  import("../CommandPalette").then((m) => ({ default: m.CommandPalette }))
);
const ArtifactDetailModal = lazy(() =>
  import("./ArtifactDetailModal").then((m) => ({ default: m.ArtifactDetailModal }))
);
const LeaveWorkspaceModal = lazy(() =>
  import("./LeaveWorkspaceModal").then((m) => ({ default: m.LeaveWorkspaceModal }))
);
const WindowCloseConfirmModal = lazy(() =>
  import("./WindowCloseConfirmModal").then((m) => ({ default: m.WindowCloseConfirmModal }))
);

export function AppShell() {
  const { state, setWorkspaceNotice } = useWorkspace();
  const [viewRefreshRevision, setViewRefreshRevision] = useState(0);
  const [sessionsLoaded, setSessionsLoaded] = useState(state.view === "sessions");
  if (state.view === "sessions" && !sessionsLoaded) {
    setSessionsLoaded(true);
  }

  if (state.screen === "picker") {
    return (
      <Suspense fallback={null}>
        <RepositoryPicker />
      </Suspense>
    );
  }

  const renderView = () => {
    switch (state.view) {
      case "pulse":
        return <ProjectPulse />;
      case "sessions":
        return null;
      case "harnesses":
        return <SettingsView initialTab="harnesses" />;
      case "wiki":
        return <WikiView />;
      case "taskboard":
        return <TaskboardView />;
      case "spec":
        return <SpecDetail />;
      case "reviews":
        return <ReviewsList />;
      case "operations":
        return <OperationsView />;
      case "debts":
        return <DebtsView />;
      case "dreams":
        return <DreamsView />;
      case "feedback":
        return <FeedbackView />;
      case "decisions":
        return <DecisionsList />;
      case "agents":
        return <AgentsView />;
      case "mcp":
        return <McpView />;
      case "repository":
        return <RepositoryView />;
      case "environment":
        return <EnvironmentView />;
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
              fontSize: "var(--text-sm)",
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
              onClick={() => setWorkspaceNotice(null)}
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
            <Suspense fallback={null}>
              {renderView()}
            </Suspense>
          </div>
          <div
            style={{
              position: "absolute",
              inset: 0,
              display: state.currentWorkspace && state.view === "sessions" ? "block" : "none",
            }}
          >
            {sessionsLoaded && (
              <Suspense fallback={null}>
                <SessionsView active={state.view === "sessions"} />
              </Suspense>
            )}
          </div>
        </div>
      </div>

      {/* Command Palette */}
      {state.cmdkOpen && (
        <Suspense fallback={null}>
          <CommandPalette />
        </Suspense>
      )}

      {/* Artifact Detail Modal */}
      {state.detailArtifact && (
        <Suspense fallback={null}>
          <ArtifactDetailModal key={state.detailArtifact.path} />
        </Suspense>
      )}

      {/* Leave Workspace Confirmation Modal */}
      {state.showExitConfirm && (
        <Suspense fallback={null}>
          <LeaveWorkspaceModal />
        </Suspense>
      )}
      {state.showWindowCloseConfirm && (
        <Suspense fallback={null}>
          <WindowCloseConfirmModal />
        </Suspense>
      )}
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
            fontSize: "var(--text-md)",
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
