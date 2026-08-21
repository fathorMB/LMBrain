import type { SessionInfo } from "../../types";

export interface SessionTabProps {
  session: SessionInfo;
  active: boolean;
  onSelect: () => void;
  onClose: () => void;
}

export function SessionTab({
  session,
  active,
  onSelect,
  onClose,
}: SessionTabProps) {
  const statusColor =
    session.status === "running"
      ? "#4bd18a"
      : session.exit_code === 0
        ? "#9fb3c8"
        : "#f28a8a";

  return (
    <button
      type="button"
      onClick={onSelect}
      aria-selected={active}
      role="tab"
      style={{
        display: "flex",
        alignItems: "center",
        gap: 8,
        height: 40,
        padding: "0 var(--space-3)",
        borderTop: "none",
        borderLeft: "none",
        borderRight: "1px solid rgba(57, 49, 70, 0.5)",
        borderBottom: active ? "2px solid #7c6cf6" : "2px solid transparent",
        background: active ? "rgba(124,108,246,0.06)" : "transparent",
        cursor: "pointer",
        minWidth: 0,
        maxWidth: 200,
        userSelect: "none",
        flexShrink: 0,
        fontFamily: "inherit",
        textAlign: "left",
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
          fontSize: "var(--text-sm)",
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
          fontSize: "var(--text-2xs)",
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
        type="button"
        aria-label={`Close session ${session.label || session.host}`}
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
        onMouseEnter={(e) => {
          e.currentTarget.style.opacity = "1";
          e.currentTarget.style.background = "rgba(255,255,255,0.06)";
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.opacity = "0.6";
          e.currentTarget.style.background = "transparent";
        }}
      >
        <i className="material-symbols-outlined" style={{ fontSize: 13 }}>
          close
        </i>
      </button>
    </button>
  );
}
