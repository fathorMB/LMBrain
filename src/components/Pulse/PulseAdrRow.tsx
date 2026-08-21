import { useWorkspace } from "../../hooks/useWorkspace";
import type { Adr } from "../../types";

export interface PulseAdrRowProps {
  adr: Adr;
}

export function PulseAdrRow({ adr }: PulseAdrRowProps) {
  const { openDetailArtifact } = useWorkspace();
  const statusColors: Record<string, { color: string; bg: string }> = {
    accepted: { color: "#46b07d", bg: "rgba(70,176,125,.12)" },
    proposed: { color: "#8a8d99", bg: "rgba(139,141,152,.12)" },
    superseded: { color: "#e0a23a", bg: "rgba(224,162,58,.12)" },
    deprecated: { color: "#e0584a", bg: "rgba(224,88,74,.12)" },
  };
  const sc = statusColors[adr.status] ?? { color: "#8a8d99", bg: "rgba(139,141,152,.12)" };

  return (
    <button
      type="button"
      onClick={() => openDetailArtifact({ title: adr.title, path: adr.path })}
      style={{
        width: "100%",
        display: "flex",
        alignItems: "center",
        gap: 10,
        padding: "11px 13px",
        borderBottom: "1px solid #201d26",
        borderLeft: "none",
        borderRight: "none",
        borderTop: "none",
        background: "transparent",
        textAlign: "left",
        cursor: "pointer",
        fontFamily: "inherit",
      }}
      onMouseEnter={(e) => {
        e.currentTarget.style.background = "#181520";
      }}
      onMouseLeave={(e) => {
        e.currentTarget.style.background = "transparent";
      }}
    >
      <span
        style={{
          fontFamily: "var(--font-mono)",
          fontSize: "var(--text-xs)",
          color: "#bcaef6",
        }}
      >
        {adr.id}
      </span>
      <span
        style={{
          flex: 1,
          fontSize: "var(--text-sm)",
          color: "var(--text-primary)",
        }}
      >
        {adr.title}
      </span>
      <span
        style={{
          fontSize: "var(--text-2xs)",
          fontWeight: 600,
          color: sc.color,
          background: sc.bg,
          borderRadius: 4,
          padding: "1px 6px",
        }}
      >
        {adr.status.toUpperCase()}
      </span>
    </button>
  );
}
