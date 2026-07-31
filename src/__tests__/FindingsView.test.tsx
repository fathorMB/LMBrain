import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { FindingsView } from "../components/Findings/FindingsView";
import { getFindingContext, getFindings } from "../lib/commands";

const dispatch = vi.fn();
const openDetailArtifact = vi.fn();
const findings = [
  {
    id: "FINDING-001", title: "Routed debt", status: "planned", category: "correctness",
    severity: "high", origin_severity: "blocking", area: "engine", milestone: "M-04",
    owner: "AGENT-ENGINE", origin_artifact: "REVIEW-054", origin_ref: "FINDING-07",
    related_specs: ["SPEC-048"], related_reviews: ["REVIEW-054"], related_decisions: [],
    target_specs: ["SPEC-059"], blocked_by: [], resolution_refs: [], superseded_by: null,
    created: "2026-07-28", updated: "2026-07-29", tags: ["debt"], body: "body",
    path: ".lmbrain/findings/planned/FINDING-001-routed-debt.md", malformed: false,
  },
  {
    id: "FINDING-002", title: "Design observation", status: "open", category: "design",
    severity: "medium", origin_severity: null, area: "ux", milestone: "M-04",
    owner: null, origin_artifact: null, origin_ref: null, related_specs: [],
    related_reviews: [], related_decisions: [], target_specs: [], blocked_by: [],
    resolution_refs: [], superseded_by: null, created: "2026-07-29", updated: "2026-07-29",
    tags: [], body: "body", path: ".lmbrain/findings/open/FINDING-002-design.md", malformed: false,
  },
];

vi.mock("../hooks/useWorkspace", () => ({
  useWorkspace: () => ({
    state: { findings },
    dispatch,
    openDetailArtifact,
  }),
}));
vi.mock("../lib/commands", () => ({
  getFindings: vi.fn(),
  getFindingContext: vi.fn(),
}));

describe("FindingsView", () => {
  beforeEach(() => vi.clearAllMocks());

  it("filters active findings with accessible controls and explicit states", () => {
    render(<FindingsView />);
    expect(screen.getByRole("heading", { name: "Findings" })).toBeDefined();
    expect(screen.getByRole("button", { name: /Open FINDING-001/ })).toBeDefined();
    expect(screen.getByText("Needs triage")).toBeDefined();
    fireEvent.change(screen.getByLabelText("Finding severity"), { target: { value: "high" } });
    expect(screen.getByRole("button", { name: /Open FINDING-001/ })).toBeDefined();
    expect(screen.queryByRole("button", { name: /Open FINDING-002/ })).toBeNull();
  });

  it("shows canonical relationships and no lifecycle mutation actions", async () => {
    vi.mocked(getFindingContext).mockResolvedValue({
      schema_version: "1", finding: findings[0],
      origin: { id: "REVIEW-054", title: "Review", status: "accepted", path: ".lmbrain/reviews/accepted/REVIEW-054.md" },
      related_specs: [], related_reviews: [], related_decisions: [],
      target_specs: [{ id: "SPEC-059", title: "Fix", status: "backlog", path: ".lmbrain/specs/backlog/SPEC-059.md" }],
      blockers: [], resolution_refs: [], superseded_by: null,
      events: [{ id: "FINDING-001-EVENT-001", action: "created", rationale: "routed", timestamp: "2026-07-29" }],
      warnings: [], omitted_relations: 0,
    });
    render(<FindingsView />);
    expect(screen.getAllByText("superseded").length).toBeGreaterThan(0);
    fireEvent.click(screen.getByRole("button", { name: /Open FINDING-001/ }));
    await screen.findByRole("dialog");
    expect(screen.getByText(/This finding is planned and routed to target spec\(s\)/)).toBeDefined();
    expect(screen.getByRole("button", { name: /REVIEW-054 · Review \(accepted\)/ })).toBeDefined();
    expect(screen.getByRole("button", { name: /SPEC-059 · Fix \(backlog\)/ })).toBeDefined();
    expect(screen.getByText(/Lifecycle actions are intentionally not available/)).toBeDefined();
    expect(screen.queryByRole("button", { name: /resolve|accept risk|reopen/i })).toBeNull();
  });

  it("closes from the compact control, Escape, and the backdrop", async () => {
    vi.mocked(getFindingContext).mockResolvedValue({
      schema_version: "1", finding: findings[0],
      origin: null, related_specs: [], related_reviews: [], related_decisions: [],
      target_specs: [], blockers: [], resolution_refs: [], superseded_by: null,
      events: [], warnings: [], omitted_relations: 0,
    });
    render(<FindingsView />);
    fireEvent.click(screen.getByRole("button", { name: /Open FINDING-001/ }));
    await screen.findByRole("dialog");

    const close = screen.getByRole("button", { name: "Close finding detail" });
    expect(close.className).toContain("modal-close-button");
    fireEvent.click(close);
    expect(screen.queryByRole("dialog")).toBeNull();

    fireEvent.click(screen.getByRole("button", { name: /Open FINDING-001/ }));
    await screen.findByRole("dialog");
    fireEvent.keyDown(window, { key: "Escape" });
    expect(screen.queryByRole("dialog")).toBeNull();

    fireEvent.click(screen.getByRole("button", { name: /Open FINDING-001/ }));
    const dialog = await screen.findByRole("dialog");
    fireEvent.mouseDown(dialog.parentElement as HTMLElement);
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("restores the original opener focus after a parent rerender", async () => {
    vi.mocked(getFindingContext).mockResolvedValue({
      schema_version: "1", finding: findings[0],
      origin: null, related_specs: [], related_reviews: [], related_decisions: [],
      target_specs: [], blockers: [], resolution_refs: [], superseded_by: null,
      events: [], warnings: [], omitted_relations: 0,
    });
    const { rerender } = render(<FindingsView />);
    const opener = screen.getByRole("button", { name: /Open FINDING-001/ });
    opener.focus();
    fireEvent.click(opener);
    await screen.findByRole("dialog");

    rerender(<FindingsView />);
    fireEvent.click(screen.getByRole("button", { name: "Close finding detail" }));
    expect(document.activeElement).toBe(opener);
  });

  it("keeps keyboard focus inside the detail modal", async () => {
    vi.mocked(getFindingContext).mockResolvedValue({
      schema_version: "1", finding: findings[0],
      origin: null, related_specs: [], related_reviews: [], related_decisions: [],
      target_specs: [], blockers: [], resolution_refs: [], superseded_by: null,
      events: [], warnings: [], omitted_relations: 0,
    });
    render(<FindingsView />);
    fireEvent.click(screen.getByRole("button", { name: /Open FINDING-001/ }));
    const dialog = await screen.findByRole("dialog");
    const close = screen.getByRole("button", { name: "Close finding detail" });
    const focusable = Array.from(
      dialog.querySelectorAll<HTMLElement>(
        "button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex='-1'])",
      ),
    );
    const last = focusable[focusable.length - 1];
    last.focus();
    fireEvent.keyDown(last, { key: "Tab" });
    expect(document.activeElement).toBe(close);
    close.focus();
    fireEvent.keyDown(close, { key: "Tab", shiftKey: true });
    expect(document.activeElement).toBe(last);
    expect(dialog).toBeDefined();
  });

  it("refreshes read-only data through the loader", async () => {
    vi.mocked(getFindings).mockResolvedValue(findings);
    render(<FindingsView />);
    fireEvent.click(screen.getByRole("button", { name: "Refresh" }));
    await waitFor(() => expect(dispatch).toHaveBeenCalledWith({ type: "SET_FINDINGS", findings }));
  });
});
