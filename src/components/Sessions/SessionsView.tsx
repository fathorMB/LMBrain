import { useMemo, useRef, useState, type CSSProperties, type RefObject } from "react";
import { Rnd } from "react-rnd";
import { listOllamaModels } from "../../lib/commands";
import { useWorkspace } from "../../hooks/useWorkspace";
import type { OllamaModel, SessionMode, SessionWindowState } from "../../types";
import { SessionTerminal } from "./SessionTerminal";

interface SessionsViewProps {
  active: boolean;
}

export function SessionsView({ active }: SessionsViewProps) {
  const {
    state,
    createSession,
    closeSession,
    updateSessionGeometry,
    bringSessionToFront,
  } = useWorkspace();
  const [modalOpen, setModalOpen] = useState(false);
  const [mode, setMode] = useState<SessionMode>("claude");
  const [model, setModel] = useState("");
  const [label, setLabel] = useState("");
  const [models, setModels] = useState<OllamaModel[]>([]);
  const [modelsLoading, setModelsLoading] = useState(false);
  const [modelsError, setModelsError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [submitError, setSubmitError] = useState<string | null>(null);

  const canvasRef = useRef<HTMLDivElement | null>(null);

  const sortedSessions = useMemo(
    () => [...state.sessions].sort((left, right) => left.zIndex - right.zIndex),
    [state.sessions]
  );

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

  // Lazily discover Ollama models from a user action (opening the modal in Ollama
  // mode, or switching to Ollama) rather than from an effect, which would trigger
  // a synchronous setState during render.
  const ensureModelsLoaded = () => {
    if (models.length === 0 && !modelsLoading) {
      refreshModels();
    }
  };

  const selectMode = (next: SessionMode) => {
    setMode(next);
    if (next === "ollama") {
      ensureModelsLoaded();
    }
  };

  const openModal = () => {
    setSubmitError(null);
    setModalOpen(true);
    if (mode === "ollama") {
      ensureModelsLoaded();
    }
  };

  const handleCreateSession = async () => {
    setSubmitting(true);
    setSubmitError(null);
    try {
      await createSession({
        mode,
        model: mode === "ollama" ? model : undefined,
        codex_bin:
          mode === "codex"
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

  return (
    <div
      style={{
        display: active ? "flex" : "none",
        flexDirection: "column",
        height: "100%",
        background:
          "radial-gradient(circle at top left, rgba(106,79,240,0.18), transparent 26%), #0c0b0f",
      }}
    >
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          gap: 16,
          padding: "18px 22px 16px",
          borderBottom: "1px solid rgba(57, 49, 70, 0.8)",
          background: "rgba(10, 9, 13, 0.7)",
          backdropFilter: "blur(8px)",
        }}
      >
        <div>
          <div
            style={{
              fontSize: 22,
              fontWeight: 800,
              letterSpacing: "-.03em",
              color: "var(--text-primary)",
            }}
          >
            Sessions
          </div>
          <div
            style={{
              marginTop: 5,
              fontSize: 12.5,
              color: "var(--text-tertiary)",
            }}
          >
            Launch interactive agent terminals in the current workspace.
          </div>
        </div>
        <button
          onClick={openModal}
          style={primaryButtonStyle}
        >
          <i className="material-symbols-outlined" style={{ fontSize: 18 }}>
            add
          </i>
          New session
        </button>
      </div>

      <div
        ref={canvasRef}
        style={{
          position: "relative",
          flex: 1,
          minHeight: 0,
          overflow: "hidden",
        }}
      >
        {sortedSessions.length === 0 && (
          <EmptySessionsState active={active} onCreate={openModal} />
        )}

        {sortedSessions.map((session) => (
          <SessionWindow
            key={session.id}
            session={session}
            active={active}
            canvasRef={canvasRef}
            onClose={() => closeSession(session.id)}
            onBringToFront={() => bringSessionToFront(session.id)}
            onGeometryChange={(geometry) =>
              updateSessionGeometry(session.id, geometry)
            }
          />
        ))}
      </div>

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
          }}
        >
          <div
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
              style={{
                fontSize: 18,
                fontWeight: 700,
                color: "var(--text-primary)",
                marginBottom: 16,
              }}
            >
              Start session
            </div>

            <div style={fieldLabelStyle}>Launch mode</div>
            <div style={{ display: "flex", gap: 8, marginBottom: 14 }}>
              <ModeButton label="Claude" selected={mode === "claude"} onClick={() => selectMode("claude")} />
              <ModeButton label="Ollama" selected={mode === "ollama"} onClick={() => selectMode("ollama")} />
              <ModeButton label="Codex" selected={mode === "codex"} onClick={() => selectMode("codex")} />
            </div>

            {mode === "ollama" && (
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
                disabled={submitting || (mode === "ollama" && !model)}
                style={{
                  ...primaryButtonStyle,
                  opacity: submitting || (mode === "ollama" && !model) ? 0.6 : 1,
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

function SessionWindow({
  session,
  active,
  canvasRef,
  onClose,
  onBringToFront,
  onGeometryChange,
}: {
  session: SessionWindowState;
  active: boolean;
  canvasRef: RefObject<HTMLDivElement | null>;
  onClose: () => void;
  onBringToFront: () => void;
  onGeometryChange: (geometry: {
    x: number;
    y: number;
    width: number;
    height: number;
  }) => void;
}) {
  const statusColor =
    session.status === "running"
      ? "#4bd18a"
      : session.exit_code === 0
        ? "#9fb3c8"
        : "#f28a8a";

  // Hand-rolled dragging: react-rnd's own dragging does not work reliably under
  // React 19 here (resize does), so we drive the position from header mouse events.
  const [dragPos, setDragPos] = useState<{ x: number; y: number } | null>(null);
  const position = dragPos ?? { x: session.geometry.x, y: session.geometry.y };

  const startDrag = (event: React.MouseEvent) => {
    if ((event.target as HTMLElement).closest("button")) {
      return;
    }
    onBringToFront();
    event.preventDefault();
    const startX = event.clientX;
    const startY = event.clientY;
    const origin = { x: session.geometry.x, y: session.geometry.y };
    const canvas = canvasRef.current;
    const maxX = canvas ? canvas.clientWidth - session.geometry.width : Number.MAX_SAFE_INTEGER;
    const maxY = canvas ? canvas.clientHeight - session.geometry.height : Number.MAX_SAFE_INTEGER;
    const clamp = (value: number, max: number) => Math.max(0, Math.min(value, Math.max(0, max)));
    let latest = origin;
    const onMove = (move: MouseEvent) => {
      latest = {
        x: clamp(origin.x + (move.clientX - startX), maxX),
        y: clamp(origin.y + (move.clientY - startY), maxY),
      };
      setDragPos(latest);
    };
    const onUp = () => {
      document.removeEventListener("mousemove", onMove);
      document.removeEventListener("mouseup", onUp);
      setDragPos(null);
      onGeometryChange({ ...session.geometry, x: latest.x, y: latest.y });
    };
    document.addEventListener("mousemove", onMove);
    document.addEventListener("mouseup", onUp);
  };

  return (
    <Rnd
      bounds="parent"
      disableDragging
      size={{ width: session.geometry.width, height: session.geometry.height }}
      position={position}
      minWidth={420}
      minHeight={240}
      onResizeStart={onBringToFront}
      onResizeStop={(_, __, element, ___, resizePosition) => {
        onGeometryChange({
          x: resizePosition.x,
          y: resizePosition.y,
          width: element.offsetWidth,
          height: element.offsetHeight,
        });
      }}
      style={{
        zIndex: session.zIndex,
        borderRadius: 16,
        overflow: "hidden",
        border: "1px solid #2d2538",
        boxShadow:
          session.status === "running"
            ? "0 18px 50px rgba(0, 0, 0, 0.38)"
            : "0 12px 34px rgba(0, 0, 0, 0.28)",
        background: "#0f0d13",
      }}
    >
      <div
        onMouseDown={onBringToFront}
        style={{
          display: "flex",
          flexDirection: "column",
          height: "100%",
        }}
      >
        <div
          onMouseDown={startDrag}
          style={{
            display: "flex",
            alignItems: "center",
            gap: 12,
            padding: "10px 12px",
            background: "linear-gradient(180deg, #17131f, #14111a)",
            borderBottom: "1px solid #2a2434",
            cursor: "move",
          }}
        >
          <div
            style={{
              width: 10,
              height: 10,
              borderRadius: "50%",
              background: statusColor,
              boxShadow: `0 0 0 4px ${statusColor}20`,
              flex: "none",
            }}
          />
          <div style={{ flex: 1, minWidth: 0 }}>
            <div
              style={{
                fontSize: 13,
                fontWeight: 700,
                color: "var(--text-primary)",
                whiteSpace: "nowrap",
                overflow: "hidden",
                textOverflow: "ellipsis",
              }}
            >
              {session.label}
            </div>
            <div
              style={{
                fontSize: 11,
                color: "var(--text-tertiary)",
                marginTop: 2,
                whiteSpace: "nowrap",
                overflow: "hidden",
                textOverflow: "ellipsis",
              }}
            >
              {session.mode === "ollama" && session.model
                ? `ollama launch claude --model ${session.model}`
                : session.mode === "codex"
                  ? "codex"
                  : "claude"}
            </div>
          </div>
          <div
            style={{
              fontSize: 10.5,
              fontWeight: 700,
              letterSpacing: ".05em",
              textTransform: "uppercase",
              color: statusColor,
              background: `${statusColor}15`,
              border: `1px solid ${statusColor}33`,
              borderRadius: 999,
              padding: "4px 8px",
            }}
          >
            {session.status === "running"
              ? "running"
              : session.exit_code === null
                ? "exited"
                : `exit ${session.exit_code}`}
          </div>
          <button
            onClick={onClose}
            style={{
              width: 28,
              height: 28,
              borderRadius: 8,
              border: "1px solid #342d40",
              background: "#16121d",
              color: "var(--text-secondary)",
              cursor: "pointer",
            }}
          >
            <i className="material-symbols-outlined" style={{ fontSize: 16 }}>
              close
            </i>
          </button>
        </div>
        <SessionTerminal sessionId={session.id} active={active} />
      </div>
    </Rnd>
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
          Start Claude Code, Codex, or route Claude Code through Ollama with a tools-capable model.
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
  padding: "10px 14px",
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
