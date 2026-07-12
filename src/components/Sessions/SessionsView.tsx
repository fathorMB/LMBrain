import { useState, type CSSProperties } from "react";
import { listOllamaModels } from "../../lib/commands";
import { useWorkspace } from "../../hooks/useWorkspace";
import type { AgentHost, ModelRoute, OllamaModel, SessionInfo } from "../../types";
import { SessionTerminal } from "./SessionTerminal";

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
  const [host, setHost] = useState<AgentHost>("claude");
  const [route, setRoute] = useState<ModelRoute>("native");
  const [model, setModel] = useState("");
  const [label, setLabel] = useState("");
  const [models, setModels] = useState<OllamaModel[]>([]);
  const [modelsLoading, setModelsLoading] = useState(false);
  const [modelsError, setModelsError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [submitError, setSubmitError] = useState<string | null>(null);

  const refreshModels = async () => {
    setModelsLoading(true);
    setModelsError(null);
    try {
      const nextModels = await listOllamaModels();
      setModels(nextModels);
      setModel((current) => {
        if (current && nextModels.some((entry) => entry.name === current)) {
          return current;
        }
        return nextModels[0]?.name ?? "";
      });
    } catch (error) {
      setModelsError(typeof error === "string" ? error : "Unable to list local Ollama models");
    } finally {
      setModelsLoading(false);
    }
  };

  const ensureModelsLoaded = () => {
    if (models.length === 0 && !modelsLoading) {
      refreshModels();
    }
  };

  const selectHost = (next: AgentHost) => {
    setHost(next);
    if (next === "pi" || next === "opencode") {
      setRoute("ollama");
      ensureModelsLoaded();
      return;
    }
    if (next === "codex") {
      setRoute("native");
      return;
    }
    if (route === "ollama") ensureModelsLoaded();
  };

  const openModal = () => {
    setSubmitError(null);
    setModalOpen(true);
    if (route === "ollama") {
      ensureModelsLoaded();
    }
  };

  const handleCreateSession = async () => {
    setSubmitting(true);
    setSubmitError(null);
    try {
      await createSession({
        host,
        route,
        model: route === "ollama" ? model : undefined,
        codex_bin:
          host === "codex"
            ? localStorage.getItem("lmbrain.codexBin")?.trim() || undefined
            : undefined,
        label,
      });
      setLabel("");
      setModalOpen(false);
    } catch (error) {
      setSubmitError(typeof error === "string" ? error : "Failed to start session");
    } finally {
      setSubmitting(false);
    }
  };

  const handleCloseTab = (id: string) => {
    closeSession(id);
  };

  return (
    <div
      style={{
        display: active ? "flex" : "none",
        position: "relative",
        flexDirection: "column",
        height: "100%",
        overflow: "hidden",
        background:
          "radial-gradient(circle at top left, rgba(106,79,240,0.18), transparent 26%), #0c0b0f",
      }}
    >
      {/* Header */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          gap: 16,
          minHeight: 48,
          padding: "0 18px",
          borderBottom: "1px solid rgba(57, 49, 70, 0.8)",
          background: "rgba(10, 9, 13, 0.7)",
          backdropFilter: "blur(8px)",
          flexShrink: 0,
        }}
      >
        <div
          style={{
            fontSize: 15,
            fontWeight: 700,
            color: "var(--text-primary)",
            letterSpacing: "-.02em",
          }}
        >
          Sessions
        </div>
        <button type="button" aria-label="New session" onClick={openModal} style={primaryButtonStyle}>
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
            padding: "0 12px",
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
          <EmptySessionsState active={active} onCreate={openModal} />
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
      {modalOpen && (
        <div
          style={{
            position: "absolute",
            inset: 0,
            background: "rgba(7, 6, 10, 0.64)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            padding: 24,
            zIndex: 100,
          }}
        >
          <div
            role="dialog"
            aria-modal="true"
            aria-labelledby="start-session-title"
            style={{
              width: 460,
              maxWidth: "100%",
              borderRadius: 18,
              background: "#121018",
              border: "1px solid #2b2436",
              boxShadow: "0 28px 80px rgba(0, 0, 0, 0.45)",
              padding: 22,
            }}
          >
            <div
              id="start-session-title"
              style={{
                fontSize: 18,
                fontWeight: 700,
                color: "var(--text-primary)",
                marginBottom: 16,
              }}
            >
              Start session
            </div>

            <div style={fieldLabelStyle}>Agent</div>
            <div style={{ display: "flex", gap: 8, marginBottom: 14 }}>
              <ModeButton label="Claude" selected={host === "claude"} onClick={() => selectHost("claude")} />
              <ModeButton label="Codex" selected={host === "codex"} onClick={() => selectHost("codex")} />
              <ModeButton label="Pi" selected={host === "pi"} onClick={() => selectHost("pi")} />
              <ModeButton label="OpenCode" selected={host === "opencode"} onClick={() => selectHost("opencode")} />
            </div>

            <div style={fieldLabelStyle}>Connection</div>
            <div style={{ display: "flex", gap: 8, marginBottom: 14 }}>
              {host !== "pi" && host !== "opencode" && <ModeButton label="Native" selected={route === "native"} onClick={() => setRoute("native")} />}
              {host !== "codex" && <ModeButton label="Ollama" selected={route === "ollama"} onClick={() => { setRoute("ollama"); ensureModelsLoaded(); }} />}
            </div>

            {route === "ollama" && (
              <>
                <div style={fieldLabelStyle}>Model</div>
                <div style={{ display: "flex", gap: 8, marginBottom: 14 }}>
                  <select
                    value={model}
                    onChange={(event) => setModel(event.target.value)}
                    style={selectStyle}
                  >
                    {models.length === 0 ? (
                      <option value="">
                        {modelsLoading ? "Loading models..." : "No models available"}
                      </option>
                    ) : (
                      models.map((entry) => (
                        <option key={entry.name} value={entry.name}>
                          {entry.name}
                        </option>
                      ))
                    )}
                  </select>
                  <button onClick={refreshModels} style={secondaryButtonStyle} disabled={modelsLoading}>
                    <i className="material-symbols-outlined" style={{ fontSize: 18 }}>
                      autorenew
                    </i>
                  </button>
                </div>
                {modelsError && (
                  <div style={errorTextStyle}>{modelsError}</div>
                )}
                {!modelsError && model && (
                  <div
                    style={{
                      marginTop: -4,
                      marginBottom: 14,
                      fontSize: 11.5,
                      color: "var(--text-tertiary)",
                    }}
                  >
                    {models.find((entry) => entry.name === model)?.cloud ? "Cloud-backed model" : "Local model"}
                  </div>
                )}
              </>
            )}

            <div style={fieldLabelStyle}>Label (optional)</div>
            <input
              value={label}
              onChange={(event) => setLabel(event.target.value)}
              placeholder="Frontend debugging"
              style={inputStyle}
            />

            {submitError && <div style={{ ...errorTextStyle, marginTop: 14 }}>{submitError}</div>}

            <div
              style={{
                display: "flex",
                justifyContent: "flex-end",
                gap: 10,
                marginTop: 22,
              }}
            >
              <button
                onClick={() => setModalOpen(false)}
                style={secondaryButtonWideStyle}
              >
                Cancel
              </button>
              <button
                onClick={handleCreateSession}
                disabled={submitting || (route === "ollama" && !model)}
                style={{
                  ...primaryButtonStyle,
                  opacity: submitting || (route === "ollama" && !model) ? 0.6 : 1,
                }}
              >
                {submitting ? "Starting..." : "Start session"}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

function SessionTab({
  session,
  active,
  onSelect,
  onClose,
}: {
  session: SessionInfo;
  active: boolean;
  onSelect: () => void;
  onClose: () => void;
}) {
  const statusColor =
    session.status === "running"
      ? "#4bd18a"
      : session.exit_code === 0
        ? "#9fb3c8"
        : "#f28a8a";

  return (
    <div
      onClick={onSelect}
      style={{
        display: "flex",
        alignItems: "center",
        gap: 8,
        height: 40,
        padding: "0 12px",
        borderRight: "1px solid rgba(57, 49, 70, 0.5)",
        borderBottom: active ? "2px solid #7c6cf6" : "2px solid transparent",
        background: active ? "rgba(124,108,246,0.06)" : "transparent",
        cursor: "pointer",
        minWidth: 0,
        maxWidth: 200,
        userSelect: "none",
        flexShrink: 0,
      }}
    >
      <div
        style={{
          width: 7,
          height: 7,
          borderRadius: "50%",
          background: statusColor,
          flex: "none",
        }}
      />
      <div
        style={{
          fontSize: 12.5,
          fontWeight: active ? 700 : 500,
          color: active ? "var(--text-primary)" : "var(--text-tertiary)",
          whiteSpace: "nowrap",
          overflow: "hidden",
          textOverflow: "ellipsis",
          flex: 1,
          minWidth: 0,
        }}
      >
        {session.label || session.host}
      </div>
      <div
        style={{
          fontSize: 10,
          color: "var(--text-muted)",
          whiteSpace: "nowrap",
          flex: "none",
        }}
      >
        {session.status === "running"
          ? session.host
          : session.exit_code === null
            ? "exited"
            : `exit ${session.exit_code}`}
      </div>
      <button
        onClick={(e) => {
          e.stopPropagation();
          onClose();
        }}
        style={{
          width: 20,
          height: 20,
          borderRadius: 5,
          border: "none",
          background: "transparent",
          color: "var(--text-muted)",
          cursor: "pointer",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          flex: "none",
          opacity: 0.6,
        }}
        onMouseEnter={(e) => { e.currentTarget.style.opacity = "1"; e.currentTarget.style.background = "rgba(255,255,255,0.06)"; }}
        onMouseLeave={(e) => { e.currentTarget.style.opacity = "0.6"; e.currentTarget.style.background = "transparent"; }}
      >
        <i className="material-symbols-outlined" style={{ fontSize: 13 }}>
          close
        </i>
      </button>
    </div>
  );
}

function EmptySessionsState({
  active,
  onCreate,
}: {
  active: boolean;
  onCreate: () => void;
}) {
  if (!active) {
    return null;
  }

  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        padding: 24,
      }}
    >
      <div
        style={{
          width: 460,
          maxWidth: "100%",
          textAlign: "center",
          borderRadius: 22,
          border: "1px solid rgba(60, 51, 74, 0.8)",
          background: "linear-gradient(180deg, rgba(22, 18, 29, 0.96), rgba(13, 11, 17, 0.96))",
          boxShadow: "0 24px 80px rgba(0, 0, 0, 0.34)",
          padding: "34px 30px",
        }}
      >
        <div
          style={{
            width: 58,
            height: 58,
            margin: "0 auto 16px",
            borderRadius: 18,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            background: "linear-gradient(140deg, #8e7af8, #4e7df7)",
            color: "white",
          }}
        >
          <i className="material-symbols-outlined" style={{ fontSize: 28 }}>
            terminal
          </i>
        </div>
        <div
          style={{
            fontSize: 23,
            fontWeight: 800,
            color: "var(--text-primary)",
            letterSpacing: "-.03em",
          }}
        >
          No active sessions yet
        </div>
        <div
          style={{
            marginTop: 10,
            fontSize: 13,
            lineHeight: 1.6,
            color: "var(--text-tertiary)",
          }}
        >
          Start Claude Code, Codex, Pi, or OpenCode through Ollama with a tools-capable model.
        </div>
        <div style={{ marginTop: 22 }}>
          <button onClick={onCreate} style={primaryButtonStyle}>
            Start your first session
          </button>
        </div>
      </div>
    </div>
  );
}

function ModeButton({
  label,
  selected,
  onClick,
}: {
  label: string;
  selected: boolean;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      aria-pressed={selected}
      style={{
        flex: 1,
        borderRadius: 12,
        border: `1px solid ${selected ? "#6f5df0" : "#2c2538"}`,
        background: selected ? "rgba(111, 93, 240, 0.16)" : "#16131d",
        color: selected ? "var(--text-primary)" : "var(--text-secondary)",
        padding: "11px 12px",
        fontSize: 13,
        fontWeight: 600,
        cursor: "pointer",
      }}
    >
      {label}
    </button>
  );
}

const primaryButtonStyle: CSSProperties = {
  display: "inline-flex",
  alignItems: "center",
  justifyContent: "center",
  gap: 8,
  border: "none",
  borderRadius: 12,
  background: "linear-gradient(135deg, #886ff7, #4d80f6)",
  color: "#fff",
  fontSize: 13,
  fontWeight: 700,
  padding: "8px 12px",
  cursor: "pointer",
};

const secondaryButtonStyle: CSSProperties = {
  width: 44,
  borderRadius: 12,
  border: "1px solid #2c2538",
  background: "#16131d",
  color: "var(--text-secondary)",
  cursor: "pointer",
};

const secondaryButtonWideStyle: CSSProperties = {
  borderRadius: 12,
  border: "1px solid #2c2538",
  background: "#16131d",
  color: "var(--text-secondary)",
  fontSize: 13,
  fontWeight: 600,
  padding: "10px 14px",
  cursor: "pointer",
};

const fieldLabelStyle: CSSProperties = {
  fontSize: 11,
  letterSpacing: ".08em",
  textTransform: "uppercase",
  color: "var(--text-muted)",
  fontWeight: 700,
  marginBottom: 8,
};

const inputStyle: CSSProperties = {
  width: "100%",
  borderRadius: 12,
  border: "1px solid #2c2538",
  background: "#0f0d14",
  color: "var(--text-primary)",
  padding: "11px 12px",
  fontSize: 13,
  outline: "none",
  boxSizing: "border-box",
};

const selectStyle: CSSProperties = {
  flex: 1,
  borderRadius: 12,
  border: "1px solid #2c2538",
  background: "#0f0d14",
  color: "var(--text-primary)",
  padding: "11px 12px",
  fontSize: 13,
  outline: "none",
};

const errorTextStyle: CSSProperties = {
  fontSize: 12,
  color: "#f28a8a",
  marginTop: 6,
};
