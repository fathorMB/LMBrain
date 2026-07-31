import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ReviewsList } from "../components/Reviews/ReviewsList";
import type { Review } from "../types";

const dispatch = vi.fn();

const review: Review = {
  id: "REVIEW-001",
  title: "Review of SPEC-001",
  status: "changes-requested",
  spec_id: "SPEC-001",
  reviewer: "AGENT-LEAD",
  implementation_agent: "AGENT-002",
  finding_categories: [],
  findings: [],
  events: [
    {
      schema_version: "1",
      id: "REVIEW-001-EVENT-001",
      timestamp: "2026-07-29T12:00:00+02:00",
      action: "verdict",
      from_status: "pending",
      to_status: "changes-requested",
      actor_role: "project-lead",
      reason: "Regression coverage is missing",
      evidence_refs: ["SPEC-001"],
      implementation_agent: "AGENT-002",
      remediation_agent: null,
    },
  ],
  lifecycle: {
    source: "structured-events",
    confidence: "high",
    review_passes: 1,
    remediation_cycles: 1,
    initial_verdict: "changes-requested",
    final_verdict: "changes-requested",
    escalation_count: 0,
    takeover_count: 0,
    remediation_agents: [],
    escalation_owners: [],
    takeover_owners: [],
    warnings: [],
  },
  lifecycle_warnings: [],
  body: "",
  path: ".lmbrain/reviews/changes-requested/REVIEW-001.md",
  created: "2026-07-29",
  updated: "2026-07-29",
  tags: [],
  links: [],
};

const pendingReview: Review = {
  ...review,
  id: "REVIEW-002",
  title: "New pending review",
  status: "pending",
  path: ".lmbrain/reviews/pending/REVIEW-002.md",
  created: "2026-07-30",
  updated: "2026-07-30",
};

const acceptedReview: Review = {
  ...review,
  id: "REVIEW-003",
  title: "Historical accepted review",
  status: "accepted",
  path: ".lmbrain/reviews/accepted/REVIEW-003.md",
  created: "2026-07-28",
  updated: "2026-07-28",
};

const unknownReview: Review = {
  ...review,
  id: "REVIEW-004",
  title: "Review with unknown status",
  status: "future-status" as Review["status"],
  path: ".lmbrain/reviews/unknown/REVIEW-004.md",
  created: "invalid-date",
  updated: "invalid-date",
};

vi.mock("../hooks/useWorkspace", () => ({
  useWorkspace: () => ({
    state: {
      reviews: [
        review,
        pendingReview,
        acceptedReview,
        unknownReview,
        {
          ...review,
          id: "REVIEW-LEGACY",
          title: "Legacy review",
          events: [],
          lifecycle: {
            ...review.lifecycle,
            source: "status-only",
            confidence: "low",
            review_passes: 1,
            remediation_cycles: 0,
            initial_verdict: null,
            warnings: [
              "Lifecycle has status only; first-pass outcome and prior remediation are unknown.",
            ],
          },
          lifecycle_warnings: [
            "Review lifecycle history is absent; prior review cycles are unknown.",
          ],
        },
      ],
    },
    dispatch,
  }),
}));

vi.mock("../lib/commands", () => ({
  getReviews: vi.fn().mockResolvedValue([]),
}));

describe("ReviewsList", () => {
  afterEach(() => {
    cleanup();
    dispatch.mockClear();
  });

  it("surfaces typed lifecycle history and legacy uncertainty", () => {
    render(<ReviewsList />);

    expect(
      screen.getAllByText(
        "1 review pass · 1 remediation cycles · latest pending → changes-requested by project-lead",
      )[0],
    ).toBeDefined();
    expect(
      screen.getByText(
        "Review lifecycle history is absent; prior review cycles are unknown.",
      ),
    ).toBeDefined();
  });

  it("groups actionable reviews first and collapses accepted history by default", () => {
    render(<ReviewsList />);

    const changesRequestedGroup = screen.getByRole("button", { name: /CHANGES REQUESTED/i });
    const pendingGroup = screen.getByRole("button", { name: /AWAITING REVIEW/i });
    const acceptedGroup = screen.getByRole("button", { name: /ACCEPTED/i });
    const unknownGroup = screen.getAllByRole("button", { name: /UNKNOWN STATUS/i })[0];
    expect(changesRequestedGroup.textContent).toContain("CHANGES REQUESTED");
    expect(pendingGroup.textContent).toContain("AWAITING REVIEW");
    expect(acceptedGroup.textContent).toContain("ACCEPTED");
    expect(unknownGroup.textContent).toContain("UNKNOWN STATUS");
    expect(changesRequestedGroup.getAttribute("aria-expanded")).toBe("true");
    expect(acceptedGroup.getAttribute("aria-expanded")).toBe("false");
    expect(screen.getByText("New pending review")).toBeDefined();
    expect(screen.queryByText("Historical accepted review")).toBeNull();
  });

  it("supports accordion toggling and keyboard review opening", () => {
    render(<ReviewsList />);

    const acceptedGroup = screen.getByRole("button", { name: /accepted/i });
    fireEvent.click(acceptedGroup);
    expect(acceptedGroup.getAttribute("aria-expanded")).toBe("true");
    expect(screen.getByText("Historical accepted review")).toBeDefined();

    const reviewCard = screen.getByRole("button", { name: /Open review REVIEW-003/i });
    expect(reviewCard.getAttribute("type")).toBe("button");
    fireEvent.click(reviewCard);
    expect(dispatch).toHaveBeenCalledWith({
      type: "SET_DETAIL_ARTIFACT",
      artifact: { title: "Historical accepted review", path: ".lmbrain/reviews/accepted/REVIEW-003.md" },
    });
  });

  it("sorts reviews newest first within a status group", () => {
    render(<ReviewsList />);

    const changesRequestedGroup = document.getElementById("reviews-group-changes-requested");
    expect(changesRequestedGroup).not.toBeNull();
    if (!changesRequestedGroup) throw new Error("Changes requested group is not rendered");
    const cards = within(changesRequestedGroup).getAllByRole("button", { name: /Open review/i });
    expect(cards[0].getAttribute("aria-label")).toBe("Open review REVIEW-001: Review of SPEC-001");
    expect(cards[1].getAttribute("aria-label")).toBe("Open review REVIEW-LEGACY: Legacy review");
  });
});
