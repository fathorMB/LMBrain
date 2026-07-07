import { render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { InsightsView } from "../components/Insights/InsightsView";
import { getProjectStatistics } from "../lib/commands";
import type { ProjectStatistics } from "../types";

const stats: ProjectStatistics = {
  artifact_families: [
    {
      family: "specs",
      label: "Specs",
      total: 4,
      statuses: [
        { status: "ready", count: 1 },
        { status: "done", count: 3 },
      ],
    },
    {
      family: "reviews",
      label: "Reviews",
      total: 3,
      statuses: [
        { status: "accepted", count: 2 },
        { status: "changes-requested", count: 1 },
      ],
    },
  ],
  spec_flow: {
    total_specs: 4,
    done_specs: 3,
    open_specs: 1,
    done_ratio: 0.75,
    by_status: [
      { status: "ready", count: 1 },
      { status: "done", count: 3 },
    ],
    by_priority: [{ status: "high", count: 4 }],
    by_area: [{ status: "backend", count: 4 }],
  },
  review_quality: {
    total_reviews: 3,
    reviewed_specs: 2,
    accepted_reviews: 2,
    changes_requested_reviews: 1,
    blocked_reviews: 0,
    superseded_reviews: 0,
    reviews_without_spec: 0,
    reviews_without_created: 0,
    specs_with_changes_requested: 1,
    specs_with_multiple_changes_requested: 0,
    change_request_rate: 0.5,
    first_pass_eligible_specs: 2,
    first_pass_accepted_specs: 1,
    first_pass_acceptance_rate: 0.5,
    average_reviews_per_reviewed_spec: 1.5,
    by_area: [
      {
        value: "backend",
        reviewed_specs: 2,
        specs_with_changes_requested: 1,
        change_request_rate: 0.5,
      },
    ],
    by_agent: [
      {
        value: "AGENT-BACKEND",
        reviewed_specs: 2,
        specs_with_changes_requested: 1,
        change_request_rate: 0.5,
      },
    ],
    trend: [
      {
        period: "2026-07",
        total_reviews: 3,
        accepted_reviews: 2,
        changes_requested_reviews: 1,
        reviewed_specs: 2,
        specs_with_changes_requested: 1,
      },
    ],
  },
  diagnostics: {
    total: 1,
    warnings: 1,
    errors: 0,
    by_family: [{ status: "specs", count: 1 }],
  },
};

vi.mock("../lib/commands", () => ({
  getProjectStatistics: vi.fn(),
}));

describe("InsightsView", () => {
  it("renders project statistics and review quality KPIs", async () => {
    vi.mocked(getProjectStatistics).mockResolvedValue(stats);

    render(<InsightsView />);

    await waitFor(() => expect(screen.getByText("Insights")).toBeDefined());

    expect(screen.getByText("Change-request rate")).toBeDefined();
    expect(screen.getByText("1/2 reviewed specs")).toBeDefined();
    expect(screen.getByText("First-pass accepted")).toBeDefined();
    expect(screen.getByText("1/2 date-ordered specs")).toBeDefined();
    expect(screen.getByText("Review Quality")).toBeDefined();
    expect(screen.getByText("Artifact Inventory")).toBeDefined();
    expect(screen.getByText("AGENT-BACKEND")).toBeDefined();
  });
});
