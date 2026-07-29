import type { CSSProperties } from "react";
import "./RefreshButton.css";

interface RefreshButtonProps {
  loading: boolean;
  onClick: () => void | Promise<void>;
}

export function RefreshButton({ loading, onClick }: RefreshButtonProps) {
  return (
    <button
      type="button"
      onClick={() => void onClick()}
      disabled={loading}
      style={{ ...buttonStyle, opacity: loading ? 0.6 : 1 }}
    >
      <i
        aria-hidden="true"
        className={`material-symbols-outlined ${loading ? "spin-icon" : ""}`}
        style={{ fontSize: 16 }}
      >
        refresh
      </i>
      Refresh
    </button>
  );
}

const buttonStyle: CSSProperties = {
  display: "inline-flex",
  alignItems: "center",
  alignSelf: "flex-start",
  flexShrink: 0,
  gap: 6,
  border: "1px solid #302a39",
  borderRadius: 7,
  background: "#19151f",
  color: "var(--text-secondary)",
  padding: "7px 12px",
  fontSize: 12,
  fontWeight: 600,
  whiteSpace: "nowrap",
  cursor: "pointer",
};
