export interface SpecMetaPillProps {
  icon: string;
  label: string;
  value: string;
}

export function SpecMetaPill({ icon, label, value }: SpecMetaPillProps) {
  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: 8,
        background: "var(--bg-tertiary)",
        border: "1px solid #262330",
        borderRadius: 9,
        padding: "8px 12px",
      }}
    >
      <i
        className="material-symbols-outlined"
        style={{ fontSize: 16, color: "var(--accent-light)" }}
      >
        {icon}
      </i>
      <div>
        <div
          style={{
            fontSize: "var(--text-2xs)",
            color: "var(--text-tertiary)",
            textTransform: "uppercase",
            letterSpacing: ".06em",
          }}
        >
          {label}
        </div>
        <div style={{ fontSize: "var(--text-sm)", fontWeight: 600 }}>{value}</div>
      </div>
    </div>
  );
}
