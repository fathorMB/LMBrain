export interface SpecLifecycleRailProps {
  status: string;
}

export function SpecLifecycleRail({ status }: SpecLifecycleRailProps) {
  const stages = [
    "proposed",
    "ready",
    "in-progress",
    "review",
    "accepted",
  ];
  const icons: Record<string, string> = {
    proposed: "check",
    ready: "flag",
    "in-progress": "bolt",
    review: "rate_review",
    accepted: "verified",
  };
  const currentIdx = stages.indexOf(status);

  return (
    <div
      style={{
        background: "#100e14",
        border: "1px solid #201d26",
        borderRadius: 12,
        padding: "16px 18px",
        marginBottom: 18,
      }}
    >
      <div
        style={{
          fontSize: "var(--text-xs)",
          letterSpacing: ".09em",
          textTransform: "uppercase",
          color: "var(--text-tertiary)",
          fontWeight: 600,
          marginBottom: 14,
        }}
      >
        Lifecycle
      </div>
      <div style={{ display: "flex", alignItems: "center" }}>
        {stages.map((stage, i) => {
          const isActive = i <= currentIdx;
          const isCurrent = i === currentIdx;
          return (
            <div key={stage} style={{ display: "flex", alignItems: "center", flex: 1 }}>
              <div
                style={{
                  display: "flex",
                  flexDirection: "column",
                  alignItems: "center",
                  gap: 7,
                  flex: "none",
                }}
              >
                <div
                  style={{
                    width: isCurrent ? 30 : 26,
                    height: isCurrent ? 30 : 26,
                    borderRadius: "50%",
                    background: isActive
                      ? isCurrent
                        ? "var(--accent)"
                        : "rgba(124,108,246,.15)"
                      : "#15131a",
                    border: isActive
                      ? isCurrent
                        ? "none"
                        : "1.5px solid var(--accent)"
                      : "1.5px solid #2e2a36",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    boxShadow: isCurrent
                      ? "0 0 0 4px rgba(124,108,246,.18)"
                      : "none",
                  }}
                >
                  <i
                    className="material-symbols-outlined"
                    style={{
                      fontSize: isCurrent ? 17 : 15,
                      color: isActive
                        ? isCurrent
                          ? "#fff"
                          : "var(--accent-light)"
                        : "var(--text-tertiary)",
                    }}
                  >
                    {icons[stage] || "circle"}
                  </i>
                </div>
                <span
                  style={{
                    fontSize: isCurrent ? 11.5 : 11,
                    color: isCurrent
                      ? "var(--text-primary)"
                      : isActive
                        ? "var(--text-secondary)"
                        : "var(--text-tertiary)",
                    fontWeight: isCurrent ? 700 : 400,
                  }}
                >
                  {stage}
                </span>
              </div>
              {i < stages.length - 1 && (
                <div
                  style={{
                    flex: 1,
                    height: 2,
                    background: isActive ? "var(--accent)" : "#26222d",
                    marginBottom: 18,
                  }}
                />
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
