import { useWorkspace } from "../../hooks/useWorkspace";

function StatBox({ value, label }: { value: number; label: string }) {
  return (
    <div
      style={{
        background: "#141217",
        border: "1px solid #232029",
        borderRadius: 9,
        padding: "10px 12px",
      }}
    >
      <div
        style={{
          fontSize: 19,
          fontWeight: 700,
          fontFamily: "var(--font-mono)",
        }}
      >
        {value}
      </div>
      <div style={{ fontSize: "var(--text-xs)", color: "var(--text-tertiary)" }}>{label}</div>
    </div>
  );
}

export function WorkspacePreview() {
  const { state, initializeWorkspaceKit } = useWorkspace();
  const ws = state.currentWorkspace;
  if (!ws) return null;

  if (ws.health === "none") {
    return (
      <div style={{ display: "flex", flexDirection: "column", height: "100%", justifyContent: "center" }}>
        <div
          style={{
            width: 48,
            height: 48,
            borderRadius: 13,
            background: "rgba(224,162,58,.12)",
            border: "1px solid rgba(224,162,58,.28)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            marginBottom: 15,
          }}
        >
          <i className="material-symbols-outlined" style={{ fontSize: 24, color: "#e0a23a" }}>
            neurology
          </i>
        </div>
        <h2 style={{ fontSize: "var(--text-xl)", fontWeight: 700, margin: "0 0 7px", color: "var(--text-primary)" }}>
          Initialize this project brain?
        </h2>
        <p style={{ fontSize: "var(--text-md)", lineHeight: 1.55, color: "var(--text-secondary)", margin: "0 0 18px", maxWidth: 350 }}>
          <span style={{ fontFamily: "var(--font-mono)", color: "#bcaef6" }}>{ws.path}</span> does not contain an LMBrain kit. Initializing creates a new{" "}
          <span style={{ fontFamily: "var(--font-mono)", color: "#bcaef6" }}>.lmbrain/</span> directory in this repository; existing files are not changed.
        </p>
        <button
          type="button"
          onClick={() => initializeWorkspaceKit(ws.path)}
          style={{
            alignSelf: "flex-start",
            display: "flex",
            alignItems: "center",
            gap: 8,
            background: "linear-gradient(180deg,#8676f7,#6e5bf2)",
            color: "#fff",
            border: "none",
            borderRadius: 10,
            padding: "11px 14px",
            fontSize: "var(--text-md)",
            fontWeight: 600,
            cursor: "pointer",
          }}
        >
          <i className="material-symbols-outlined" style={{ fontSize: 18 }}>
            add_circle
          </i>
          Initialize LMBrain kit
        </button>
      </div>
    );
  }

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%" }}>
      <span
        style={{
          fontSize: "var(--text-xs)",
          letterSpacing: ".09em",
          textTransform: "uppercase",
          color: "var(--text-tertiary)",
          fontWeight: 600,
        }}
      >
        Selected workspace
      </span>
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: 12,
          margin: "13px 0 20px",
        }}
      >
        <div
          style={{
            width: 40,
            height: 40,
            borderRadius: 11,
            background: "linear-gradient(150deg,#2c2738,#211d2b)",
            border: "1px solid #36303f",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            fontFamily: "var(--font-mono)",
            fontWeight: 600,
            color: "#a596f5",
            fontSize: "var(--text-xl)",
          }}
        >
          {ws.name.charAt(0).toUpperCase()}
        </div>
        <div style={{ minWidth: 0 }}>
          <div
            style={{
              fontSize: "var(--text-lg)",
              fontWeight: 700,
              letterSpacing: "-.01em",
              color: "var(--text-primary)",
            }}
          >
            {ws.name}
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
            {ws.path}
          </div>
        </div>
      </div>

      <div
        style={{
          display: "grid",
          gridTemplateColumns: "1fr 1fr",
          gap: 8,
          marginBottom: 16,
        }}
      >
        <StatBox value={ws.spec_count} label="specifications" />
        <StatBox value={ws.decision_count} label="decisions" />
        <StatBox value={ws.agent_count} label="agent profiles" />
      </div>

      <div
        style={{
          display: "flex",
          flexDirection: "column",
          gap: 8,
          fontSize: "var(--text-sm)",
          color: "#b6b1bb",
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
          <i className="material-symbols-outlined" style={{ fontSize: 16, color: "var(--green)" }}>
            check_circle
          </i>
          <span
            style={{
              fontFamily: "var(--font-mono)",
              color: "#bcaef6",
              fontSize: "var(--text-sm)",
            }}
          >
            .lmbrain/
          </span>{" "}
          detected · {ws.health === "ok" ? "readable" : ws.health}
        </div>
        {ws.kit_version && (
          <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
            <i className="material-symbols-outlined" style={{ fontSize: 16, color: "var(--green)" }}>
              check_circle
            </i>
            Kit version {ws.kit_version}
          </div>
        )}
        {ws.diagnostics.map((d, i) => (
          <div key={i} style={{ display: "flex", alignItems: "center", gap: 8 }}>
            <i
              className="material-symbols-outlined"
              style={{
                fontSize: 16,
                color: d.severity === "error" ? "var(--red)" : "var(--yellow)",
              }}
            >
              {d.severity === "error" ? "error" : "warning"}
            </i>
            <span style={{ fontSize: "var(--text-sm)" }}>{d.message}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
