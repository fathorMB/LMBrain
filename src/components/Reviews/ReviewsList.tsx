import { useEffect, useMemo, useState } from "react";
import { useWorkspace } from "../../hooks/useWorkspace";
import { getReviews } from "../../lib/commands";
import { CardGrid, EmptyState, PageHeader, PageShell } from "../Shared/PageLayout";
import type { Review } from "../../types";

const ACTIONABLE_STATUSES = ["changes-requested", "pending", "blocked"] as const;
const HISTORY_STATUSES = ["accepted", "superseded"] as const;

type ReviewGroup = { key: string; label: string; reviews: Review[]; actionable: boolean };

const statusConfig: Record<string, { color: string; label: string; border: string }> = {
  pending: { color: "#e0a23a", label: "AWAITING REVIEW", border: "#e0a23a" },
  "changes-requested": { color: "#e0584a", label: "CHANGES REQUESTED", border: "#e0584a" },
  accepted: { color: "#46b07d", label: "ACCEPTED", border: "#46b07d" },
  blocked: { color: "#e0584a", label: "BLOCKED", border: "#e0584a" },
  superseded: { color: "var(--text-tertiary)", label: "SUPERSEDED", border: "var(--text-tertiary)" },
};

function reviewDate(review: Review): number | null {
  for (const value of [review.updated, review.created]) {
    const timestamp = Date.parse(value);
    if (Number.isFinite(timestamp)) return timestamp;
  }
  return null;
}

function compareReviews(left: Review, right: Review): number {
  const leftDate = reviewDate(left);
  const rightDate = reviewDate(right);
  if (leftDate !== null && rightDate !== null && leftDate !== rightDate) return rightDate - leftDate;
  if (leftDate !== null && rightDate === null) return -1;
  if (leftDate === null && rightDate !== null) return 1;
  return left.id.localeCompare(right.id);
}

function groupReviews(reviews: Review[]): ReviewGroup[] {
  const buckets = new Map<string, Review[]>();
  for (const review of reviews) {
    const key = statusConfig[review.status] ? review.status : "unknown";
    const bucket = buckets.get(key) ?? [];
    bucket.push(review);
    buckets.set(key, bucket);
  }

  const rank = (key: string) => {
    const actionableIndex = ACTIONABLE_STATUSES.indexOf(key as (typeof ACTIONABLE_STATUSES)[number]);
    if (actionableIndex >= 0) return actionableIndex;
    const historyIndex = HISTORY_STATUSES.indexOf(key as (typeof HISTORY_STATUSES)[number]);
    if (historyIndex >= 0) return ACTIONABLE_STATUSES.length + historyIndex;
    return ACTIONABLE_STATUSES.length + HISTORY_STATUSES.length;
  };

  return Array.from(buckets.entries())
    .sort(([left], [right]) => rank(left) - rank(right) || left.localeCompare(right))
    .map(([key, group]) => ({
      key,
      label: key === "unknown" ? "UNKNOWN STATUS" : statusConfig[key].label,
      reviews: [...group].sort(compareReviews),
      actionable: ACTIONABLE_STATUSES.includes(key as (typeof ACTIONABLE_STATUSES)[number]),
    }));
}

export function ReviewsList() {
  const { state, dispatch, navigateTo } = useWorkspace();
  const [collapsedGroups, setCollapsedGroups] = useState<Set<string>>(() => new Set(HISTORY_STATUSES));

  useEffect(() => {
    getReviews()
      .then((reviews) => dispatch({ type: "SET_REVIEWS", reviews }))
      .catch(console.error);
  }, [dispatch]);

  const groups = useMemo(() => groupReviews(state.reviews), [state.reviews]);

  const toggleGroup = (key: string) => {
    setCollapsedGroups((current) => {
      const next = new Set(current);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
  };

  return (
    <PageShell archetype="dense">
      <PageHeader
        title="Reviews"
        description="Work returned by the Project Lead. Accept to close, or send findings back to a specialist."
      />

      <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-4)" }}>
          {groups.length === 0 && <EmptyState>No reviews yet.</EmptyState>}
          {groups.map((group) => {
            const expanded = !collapsedGroups.has(group.key);
            const panelId = `reviews-group-${group.key}`;
            const groupConfig = statusConfig[group.key] ?? { color: "#9a91a8", label: group.label, border: "#9a91a8" };

            return (
              <section key={group.key} aria-labelledby={`${panelId}-label`}>
                <button
                  type="button"
                  aria-expanded={expanded}
                  aria-controls={panelId}
                  onClick={() => toggleGroup(group.key)}
                  style={{ width: "100%", display: "flex", alignItems: "center", gap: 10, padding: "10px 12px", border: "1px solid #2a2731", borderRadius: expanded ? "10px 10px 0 0" : 10, background: "var(--bg-secondary)", color: "var(--text-primary)", cursor: "pointer", textAlign: "left" }}
                >
                  <i className="material-symbols-outlined" aria-hidden="true" style={{ fontSize: 18, color: groupConfig.color }}>{expanded ? "expand_more" : "chevron_right"}</i>
                  <span id={`${panelId}-label`} style={{ flex: 1, fontSize: "var(--text-sm)", fontWeight: 800, letterSpacing: ".04em" }}>{group.label}</span>
                  <span style={{ fontSize: "var(--text-xs)", color: "var(--text-tertiary)" }}>{group.reviews.length} {group.reviews.length === 1 ? "review" : "reviews"}</span>
                </button>

                {expanded && (
                  <div id={panelId} role="region" aria-labelledby={`${panelId}-label`} style={{ paddingTop: "var(--space-3)" }}>
                    <CardGrid>
                    {group.reviews.map((review) => {
                      const reviewConfig = statusConfig[review.status] ?? groupConfig;
                      const isMalformed = !!review.malformed;
                      const latestEvent = review.events.at(-1);
                      const promoted = (state.debts ?? []).filter((debt) => debt.origin_artifact === review.id || debt.related_reviews.includes(review.id));
                      const openReview = () => dispatch({ type: "SET_DETAIL_ARTIFACT", artifact: { title: review.title, path: review.path } });

                      return (
                        <div
                          key={review.id}
                          role="group"
                          aria-label={`Review ${review.id}: ${review.title}`}
                          style={{ display: "block", background: "var(--bg-tertiary)", border: isMalformed ? "1px solid #e0584a" : "1px solid #2a2731", borderRadius: 12, padding: "15px 16px", borderLeft: isMalformed ? "3px solid #e0584a" : `3px solid ${reviewConfig.border}` }}
                          onMouseEnter={(event) => { event.currentTarget.style.borderColor = isMalformed ? "#f06f60" : "#3a3446"; event.currentTarget.style.background = "#171420"; }}
                          onMouseLeave={(event) => { event.currentTarget.style.borderColor = isMalformed ? "#e0584a" : "#2a2731"; event.currentTarget.style.background = "var(--bg-tertiary)"; }}
                        >
                          <button
                            type="button"
                            aria-label={`Open review ${review.id}: ${review.title}`}
                            onClick={openReview}
                            style={{ width: "100%", display: "flex", alignItems: "center", gap: 14, border: 0, padding: 0, background: "transparent", color: "inherit", cursor: "pointer", textAlign: "left" }}
                            onFocus={(event) => { event.currentTarget.style.outline = "2px solid #bcaef6"; event.currentTarget.style.outlineOffset = "2px"; }}
                            onBlur={(event) => { event.currentTarget.style.outline = "none"; }}
                          >
                            <div style={{ flex: 1 }}>
                              <div style={{ display: "flex", alignItems: "center", gap: 9, marginBottom: 4, flexWrap: "wrap" }}>
                                <span style={{ fontFamily: "var(--font-mono)", fontSize: "var(--text-sm)", color: "#bcaef6" }}>{review.id}</span>
                                <span style={{ fontSize: "var(--text-md)", fontWeight: 700, color: "var(--text-primary)" }}>{review.title}</span>
                                {isMalformed && <span style={{ display: "inline-flex", alignItems: "center", gap: 4, fontSize: 10, fontWeight: 700, color: "#e0584a", background: "rgba(224,88,74,0.13)", borderRadius: 5, padding: "2px 6px" }}><i className="material-symbols-outlined" aria-hidden="true" style={{ fontSize: 11 }}>warning</i>MALFORMED</span>}
                              </div>
                              <div style={{ fontSize: "var(--text-sm)", color: "var(--text-tertiary)" }}>{review.reviewer ? `Reviewed by ${review.reviewer}` : "No reviewer assigned"}</div>
                              {(latestEvent || review.lifecycle.source !== "status-only") && <div style={{ fontSize: "var(--text-xs)", color: "var(--text-tertiary)", marginTop: 4 }}>{review.lifecycle.review_passes} review {review.lifecycle.review_passes === 1 ? "pass" : "passes"}{" · "}{review.lifecycle.remediation_cycles} remediation cycles{latestEvent && <>{" · "}latest {latestEvent.from_status} → {latestEvent.to_status} by {latestEvent.actor_role}</>}</div>}
                              {review.lifecycle_warnings.length > 0 && <div role="status" style={{ fontSize: 11, color: "#e0a23a", marginTop: 4 }}><i className="material-symbols-outlined" aria-hidden="true" style={{ fontSize: 12, verticalAlign: -2 }}>history</i>{" "}{review.lifecycle_warnings[0]}</div>}
                            </div>
                            <span style={{ display: "inline-flex", alignItems: "center", gap: 5, fontSize: "var(--text-xs)", fontWeight: 600, color: reviewConfig.color, background: `${reviewConfig.color}1a`, border: `1px solid ${reviewConfig.color}40`, borderRadius: 6, padding: "4px 9px" }}>{reviewConfig.label}</span>
                            <i className="material-symbols-outlined" aria-hidden="true" style={{ fontSize: 18, color: "var(--text-tertiary)" }}>chevron_right</i>
                          </button>
                          {promoted.length > 0 && <button type="button" onClick={() => navigateTo("debts")} style={{ marginTop: 6, border: 0, padding: 0, background: "transparent", color: "#bcaef6", cursor: "pointer", fontSize: "var(--text-xs)" }}>{promoted.length} promoted {promoted.length === 1 ? "debt" : "debts"} · view current disposition</button>}
                        </div>
                      );
                    })}
                    </CardGrid>
                  </div>
                )}
              </section>
            );
          })}
      </div>
    </PageShell>
  );
}
