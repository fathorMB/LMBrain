import { useState } from "react";
import { useWorkspace } from "../../hooks/useWorkspace";
import type { AgentHost, ModelRoute } from "../../types";
import { SessionTerminal } from "./SessionTerminal";
import { SessionTab } from "./SessionTab";
import { EmptySessionsState } from "./EmptySessionsState";
import { NewSessionModal } from "./NewSessionModal";

interface SessionsViewProps {
  active: boolean;
}

export function SessionsView({ active }: SessionsViewProps) {
  const {
    state,
    createSession,
    closeSession,
    setActiveSession,
  } = useWorkspace();
  const [modalOpen, setModalOpen] = useState(false);

  const handleCreateSession = async (params: {
    host: AgentHost;
    route: ModelRoute;
    model?: string;
    label?: string;
  }) => {
    await createSession({
      host: params.host,
      route: params.route,
      model: params.model,
      codex_bin:
        params.host === "codex"
          ? localStorage.getItem("lmbrain.codexBin")?.trim() || undefined
          : undefined,
      label: params.label,
    });
    setModalOpen(false);
  };

  const handleCloseTab = (id: string) => {
    closeSession(id);
  };

  return (
    <div
      style={{
        height: "100%",
        display: "flex",
        flexDirection: "column",
        background: "var(--bg-primary)",
        color: "var(--text-primary)",
        position: "relative",
      }}
    >
      {/* Header bar */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          padding: "var(--space-3) var(--space-5)",
          borderBottom: "1px solid rgba(57, 49, 70, 0.6)",
          background: "var(--bg-secondary)",
          flexShrink: 0,
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
          <div
            style={{
              width: 32,
              height: 32,
              borderRadius: 10,
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              background: "linear-gradient(140deg, #8e7af8, #4e7df7)",
              color: "white",
            }}
          >
            <i className="material-symbols-outlined" style={{ fontSize: 18 }}>
              terminal
            </i>
          </div>
          <div>
            <div
              style={{
                fontSize: "var(--text-lg)",
                fontWeight: 700,
                color: "var(--text-primary)",
                letterSpacing: "-.02em",
              }}
            >
              Agent Sessions
            </div>
            <div
              style={{
                fontSize: "var(--text-xs)",
                color: "var(--text-tertiary)",
              }}
            >
              Run interactive coding agents in dedicated PTY terminals
            </div>
          </div>
        </div>

        <button
          type="button"
          aria-label="New session"
          onClick={() => setModalOpen(true)}
          style={{
            display: "inline-flex",
            alignItems: "center",
            justifyContent: "center",
            gap: 8,
            border: "none",
            borderRadius: 12,
            background: "linear-gradient(135deg, #886ff7, #4d80f6)",
            color: "#fff",
            fontSize: "var(--text-md)",
            fontWeight: 700,
            padding: "8px 12px",
            cursor: "pointer",
          }}
        >
          <i className="material-symbols-outlined" aria-hidden="true" style={{ fontSize: 16 }}>
            add
          </i>
          New session
        </button>
      </div>

      {/* Tab strip */}
      {state.sessions.length > 0 && (
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: 0,
            padding: "0 var(--space-3)",
            minHeight: 40,
            background: "rgba(10, 9, 13, 0.5)",
            borderBottom: "1px solid rgba(57, 49, 70, 0.6)",
            flexShrink: 0,
            overflowX: "auto",
          }}
        >
          {state.sessions.map((session) => (
            <SessionTab
              key={session.id}
              session={session}
              active={session.id === state.activeSessionId}
              onSelect={() => setActiveSession(session.id)}
              onClose={() => handleCloseTab(session.id)}
            />
          ))}
        </div>
      )}

      {/* Terminal area */}
      <div style={{ flex: 1, minHeight: 0, position: "relative" }}>
        {state.sessions.length === 0 && (
          <EmptySessionsState active={active} onCreate={() => setModalOpen(true)} />
        )}

        {state.sessions.map((session) => {
          const sessionActive = session.id === state.activeSessionId;
          return (
            <div
              key={session.id}
              aria-hidden={!sessionActive}
              style={{
                position: "absolute",
                inset: 0,
                display: sessionActive ? "flex" : "none",
                flexDirection: "column",
              }}
            >
              <SessionTerminal
                sessionId={session.id}
                active={active && sessionActive}
                host={session.host}
              />
            </div>
          );
        })}
      </div>

      {/* New session modal */}
      <NewSessionModal
        isOpen={modalOpen}
        onClose={() => setModalOpen(false)}
        onSubmit={handleCreateSession}
      />
    </div>
  );
}
