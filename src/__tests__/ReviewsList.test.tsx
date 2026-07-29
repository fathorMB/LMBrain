import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
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

vi.mock("../hooks/useWorkspace", () => ({
  useWorkspace: () => ({
    state: {
      reviews: [
        review,
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
  it("surfaces typed lifecycle history and legacy uncertainty", () => {
    render(<ReviewsList />);

    expect(
      screen.getByText(
        "1 review pass · 1 remediation cycles · latest pending → changes-requested by project-lead",
      ),
    ).toBeDefined();
    expect(
      screen.getByText(
        "Review lifecycle history is absent; prior review cycles are unknown.",
      ),
    ).toBeDefined();
  });
});
