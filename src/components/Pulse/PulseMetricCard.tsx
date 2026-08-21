export interface PulseMetricCardProps {
  count: number;
  label: string;
  accent: string;
}

export function PulseMetricCard({ count, label, accent }: PulseMetricCardProps) {
  return (
    <div
      style={{
        background: "var(--bg-tertiary)",
        border: "1px solid var(--border-secondary)",
        borderRadius: 12,
        padding: 14,
        position: "relative",
        overflow: "hidden",
      }}
    >
      <div
        style={{
          position: "absolute",
          top: 0,
          left: 0,
          width: 3,
          height: "100%",
          background: accent,
        }}
      />
      <div
        style={{
          fontSize: 27,
          fontWeight: 800,
          fontFamily: "var(--font-mono)",
          letterSpacing: "-.02em",
        }}
      >
        {count}
      </div>
      <div
        style={{
          fontSize: "var(--text-sm)",
          color: "var(--text-tertiary)",
          marginTop: 2,
        }}
      >
        {label}
      </div>
    </div>
  );
}
