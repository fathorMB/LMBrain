import { useEffect } from "react";
import { useWorkspace } from "../../hooks/useWorkspace";
import { getReviews } from "../../lib/commands";

export function ReviewsList() {
  const { state, dispatch } = useWorkspace();

  useEffect(() => {
    getReviews()
      .then((reviews) => dispatch({ type: "SET_REVIEWS", reviews }))
      .catch(console.error);
  }, [dispatch]);

  const statusConfig: Record<string, { color: string; label: string; border: string }> = {
    pending: { color: "#e0a23a", label: "AWAITING REVIEW", border: "#e0a23a" },
    "changes-requested": { color: "#e0584a", label: "CHANGES REQUESTED", border: "#e0584a" },
    accepted: { color: "#46b07d", label: "ACCEPTED", border: "#46b07d" },
    blocked: { color: "#e0584a", label: "BLOCKED", border: "#e0584a" },
    superseded: { color: "#6c6671", label: "SUPERSEDED", border: "#6c6671" },
  };

  return (
    <div style={{ overflowY: "auto", height: "100%" }}>
      <div style={{ maxWidth: 920, margin: "0 auto", padding: "24px 36px 70px" }}>
        <h1
          style={{
            fontSize: 24,
            fontWeight: 800,
            letterSpacing: "-.025em",
            margin: "0 0 5px",
          }}
        >
          Reviews
        </h1>
        <p
          style={{
            fontSize: 13.5,
            color: "var(--text-tertiary)",
            margin: "0 0 22px",
          }}
        >
          Work returned by the Project Lead. Accept to close, or send findings
          back to a specialist.
        </p>

        <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
          {state.reviews.length === 0 && (
            <div
              style={{
                textAlign: "center",
                padding: 40,
                color: "var(--text-tertiary)",
              }}
            >
              No reviews yet.
            </div>
          )}
          {state.reviews.map((review) => {
            const cfg = statusConfig[review.status] || statusConfig.pending;
            return (
              <div
                key={review.id}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 14,
                  background: "var(--bg-tertiary)",
                  border: "1px solid #2a2731",
                  borderRadius: 12,
                  padding: "15px 16px",
                  cursor: "pointer",
                  borderLeft: `3px solid ${cfg.border}`,
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.borderColor = "#3a3446";
                  e.currentTarget.style.background = "#171420";
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.borderColor = "#2a2731";
                  e.currentTarget.style.background = "var(--bg-tertiary)";
                }}
              >
                <div style={{ flex: 1 }}>
                  <div
                    style={{
                      display: "flex",
                      alignItems: "center",
                      gap: 9,
                      marginBottom: 4,
                    }}
                  >
                    <span
                      style={{
                        fontFamily: "var(--font-mono)",
                        fontSize: 12,
                        color: "#bcaef6",
                      }}
                    >
                      {review.id}
                    </span>
                    <span
                      style={{
                        fontSize: 14,
                        fontWeight: 700,
                        color: "var(--text-primary)",
                      }}
                    >
                      {review.title}
                    </span>
                  </div>
                  <div
                    style={{
                      fontSize: 12,
                      color: "var(--text-tertiary)",
                    }}
                  >
                    {review.reviewer
                      ? `Reviewed by ${review.reviewer}`
                      : "No reviewer assigned"}
                  </div>
                </div>
                <span
                  style={{
                    display: "inline-flex",
                    alignItems: "center",
                    gap: 5,
                    fontSize: 11,
                    fontWeight: 600,
                    color: cfg.color,
                    background: `${cfg.color}1a`,
                    border: `1px solid ${cfg.color}40`,
                    borderRadius: 6,
                    padding: "4px 9px",
                  }}
                >
                  {cfg.label}
                </span>
                <i
                  className="material-symbols-outlined"
                  style={{ fontSize: 18, color: "#6c6671" }}
                >
                  chevron_right
                </i>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
