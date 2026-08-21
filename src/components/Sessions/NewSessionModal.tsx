import { useState, type CSSProperties } from "react";
import { listOllamaModels } from "../../lib/commands";
import type { AgentHost, ModelRoute, OllamaModel } from "../../types";

export interface NewSessionModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (params: {
    host: AgentHost;
    route: ModelRoute;
    model?: string;
    label?: string;
  }) => Promise<void>;
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
      type="button"
      onClick={onClick}
      aria-pressed={selected}
      style={{
        flex: 1,
        borderRadius: 12,
        border: `1px solid ${selected ? "#6f5df0" : "#2c2538"}`,
        background: selected ? "rgba(111, 93, 240, 0.16)" : "#16131d",
        color: selected ? "var(--text-primary)" : "var(--text-secondary)",
        padding: "11px 12px",
        fontSize: "var(--text-md)",
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
  fontSize: "var(--text-md)",
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
  fontSize: "var(--text-md)",
  fontWeight: 600,
  padding: "10px 14px",
  cursor: "pointer",
};

const fieldLabelStyle: CSSProperties = {
  fontSize: "var(--text-xs)",
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
  fontSize: "var(--text-md)",
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
  fontSize: "var(--text-md)",
  outline: "none",
};

const errorTextStyle: CSSProperties = {
  fontSize: "var(--text-sm)",
  color: "#f28a8a",
  marginTop: 6,
};

export function NewSessionModal({ isOpen, onClose, onSubmit }: NewSessionModalProps) {
  const [host, setHost] = useState<AgentHost>("claude");
  const [route, setRoute] = useState<ModelRoute>("native");
  const [model, setModel] = useState("");
  const [label, setLabel] = useState("");
  const [models, setModels] = useState<OllamaModel[]>([]);
  const [modelsLoading, setModelsLoading] = useState(false);
  const [modelsError, setModelsError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [submitError, setSubmitError] = useState<string | null>(null);

  if (!isOpen) return null;

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

  const handleStart = async () => {
    setSubmitting(true);
    setSubmitError(null);
    try {
      await onSubmit({
        host,
        route,
        model: route === "ollama" ? model : undefined,
        label,
      });
      setLabel("");
    } catch (error) {
      setSubmitError(typeof error === "string" ? error : "Failed to start session");
    } finally {
      setSubmitting(false);
    }
  };

  return (
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
            fontSize: "var(--text-xl)",
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
          {host !== "pi" && host !== "opencode" && (
            <ModeButton label="Native" selected={route === "native"} onClick={() => setRoute("native")} />
          )}
          {host !== "codex" && (
            <ModeButton
              label="Ollama"
              selected={route === "ollama"}
              onClick={() => {
                setRoute("ollama");
                ensureModelsLoaded();
              }}
            />
          )}
        </div>

        {route === "ollama" && (
          <>
            <div style={fieldLabelStyle}>Model</div>
            <div style={{ display: "flex", gap: 8, marginBottom: 14 }}>
              <select
                className="app-select"
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
              <button
                type="button"
                onClick={refreshModels}
                style={secondaryButtonStyle}
                disabled={modelsLoading}
              >
                <i className="material-symbols-outlined" style={{ fontSize: 18 }}>
                  autorenew
                </i>
              </button>
            </div>
            {modelsError && <div style={errorTextStyle}>{modelsError}</div>}
            {!modelsError && model && (
              <div
                style={{
                  marginTop: -4,
                  marginBottom: 14,
                  fontSize: "var(--text-xs)",
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
            type="button"
            onClick={onClose}
            style={secondaryButtonWideStyle}
          >
            Cancel
          </button>
          <button
            type="button"
            onClick={handleStart}
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
  );
}
