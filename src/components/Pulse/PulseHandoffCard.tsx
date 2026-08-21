import { useWorkspace } from "../../hooks/useWorkspace";
import type { Handoff } from "../../types";

export interface PulseHandoffCardProps {
  handoff: Handoff;
}

export function PulseHandoffCard({ handoff }: PulseHandoffCardProps) {
  const { openDetailArtifact } = useWorkspace();
  return (
    <div
      style={{
        background: "var(--bg-tertiary)",
        border: "1px solid #2a2731",
        borderRadius: 12,
        padding: 15,
        borderTop: "2px solid var(--accent)",
      }}
    >
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          marginBottom: 8,
        }}
      >
        <span
          style={{
            fontFamily: "var(--font-mono)",
            fontSize: "var(--text-sm)",
            color: "#bcaef6",
            fontWeight: 500,
          }}
        >
          {handoff.id}
        </span>
      </div>
      <div
        style={{
          fontSize: "var(--text-md)",
          fontWeight: 700,
          marginBottom: 10,
          color: "var(--text-primary)",
        }}
      >
        {handoff.title}
      </div>
      <button
        type="button"
        onClick={() => openDetailArtifact({ title: handoff.title, path: handoff.path })}
        style={{
          width: "100%",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          gap: 7,
          background: "linear-gradient(180deg,#8676f7,#6e5bf2)",
          border: "none",
          color: "#fff",
          borderRadius: 8,
          padding: 8,
          fontSize: "var(--text-sm)",
          fontWeight: 600,
          cursor: "pointer",
        }}
      >
        <i className="material-symbols-outlined" style={{ fontSize: 16 }}>
          open_in_full
        </i>
        Open handoff
      </button>
    </div>
  );
}
