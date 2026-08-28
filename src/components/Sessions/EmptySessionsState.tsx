export interface EmptySessionsStateProps {
  active: boolean;
  onCreate: () => void;
}

export function EmptySessionsState({
  active,
  onCreate,
}: EmptySessionsStateProps) {
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
            fontSize: "var(--text-md)",
            lineHeight: 1.6,
            color: "var(--text-tertiary)",
          }}
        >
          Start Claude Code, Codex, or Pi through Ollama with a tools-capable model.
        </div>
        <div style={{ marginTop: 22 }}>
          <button
            type="button"
            onClick={onCreate}
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
            Start your first session
          </button>
        </div>
      </div>
    </div>
  );
}
