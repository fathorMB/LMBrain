import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { DebtsView } from "../components/Debts/DebtsView";
import { getDebtContext } from "../lib/commands";

const refreshWorkspaceData = vi.fn().mockResolvedValue(undefined);
const openDetailArtifact = vi.fn();
const debts = [
  {
    id: "DEBT-001", title: "Routed debt", status: "planned", category: "correctness",
    severity: "high", origin_severity: "blocking", area: "engine", milestone: "M-04",
    owner: "AGENT-ENGINE", origin_artifact: "REVIEW-054", origin_ref: "RF-007",
    related_specs: ["SPEC-048"], related_reviews: ["REVIEW-054"], related_decisions: [],
    target_specs: ["SPEC-059"], blocked_by: [], resolution_refs: [], superseded_by: null,
    created: "2026-07-28", updated: "2026-07-29", tags: ["debt"], body: "body",
    path: ".lmbrain/debts/planned/DEBT-001-routed-debt.md", malformed: false,
  },
  {
    id: "DEBT-002", title: "Design observation", status: "open", category: "design",
    severity: "medium", origin_severity: null, area: "ux", milestone: "M-04",
    owner: null, origin_artifact: null, origin_ref: null, related_specs: [],
    related_reviews: [], related_decisions: [], target_specs: [], blocked_by: [],
    resolution_refs: [], superseded_by: null, created: "2026-07-29", updated: "2026-07-29",
    tags: [], body: "body", path: ".lmbrain/debts/open/DEBT-002-design.md", malformed: false,
  },
];

vi.mock("../hooks/useWorkspace", () => ({
  useWorkspace: () => ({
    state: { debts },
    refreshWorkspaceData,
    openDetailArtifact,
  }),
}));
vi.mock("../lib/commands", () => ({
  getDebts: vi.fn(),
  getDebtContext: vi.fn(),
}));

describe("DebtsView", () => {
  beforeEach(() => vi.clearAllMocks());

  it("filters active debts with accessible controls and explicit states", () => {
    render(<DebtsView />);
    expect(screen.getByRole("heading", { name: "Debts" })).toBeDefined();
    expect(screen.getByRole("button", { name: /Open DEBT-001/ })).toBeDefined();
    expect(screen.getByText("Needs triage")).toBeDefined();
    fireEvent.change(screen.getByLabelText("Debt severity"), { target: { value: "high" } });
    expect(screen.getByRole("button", { name: /Open DEBT-001/ })).toBeDefined();
    expect(screen.queryByRole("button", { name: /Open DEBT-002/ })).toBeNull();
  });

  it("shows canonical relationships and no lifecycle mutation actions", async () => {
    vi.mocked(getDebtContext).mockResolvedValue({
      schema_version: "1", debt: debts[0],
      origin: { id: "REVIEW-054", title: "Review", status: "accepted", path: ".lmbrain/reviews/accepted/REVIEW-054.md" },
      related_specs: [], related_reviews: [], related_decisions: [],
      target_specs: [{ id: "SPEC-059", title: "Fix", status: "backlog", path: ".lmbrain/specs/backlog/SPEC-059.md" }],
      blockers: [], resolution_refs: [], superseded_by: null,
      events: [{ id: "DEBT-001-EVENT-001", action: "created", rationale: "routed", timestamp: "2026-07-29" }],
      warnings: [], omitted_relations: 0,
    });
    render(<DebtsView />);
    expect(screen.getAllByText("superseded").length).toBeGreaterThan(0);
    fireEvent.click(screen.getByRole("button", { name: /Open DEBT-001/ }));
    await screen.findByRole("dialog");
    expect(screen.getByText(/This debt is planned and routed to target spec\(s\)/)).toBeDefined();
    expect(screen.getByRole("button", { name: /REVIEW-054 · Review \(accepted\)/ })).toBeDefined();
    expect(screen.getByRole("button", { name: /SPEC-059 · Fix \(backlog\)/ })).toBeDefined();
    expect(screen.getByText(/Lifecycle actions are intentionally not available/)).toBeDefined();
    expect(screen.queryByRole("button", { name: /resolve|accept risk|reopen/i })).toBeNull();
  });

  it("closes from the compact control, Escape, and the backdrop", async () => {
    vi.mocked(getDebtContext).mockResolvedValue({
      schema_version: "1", debt: debts[0],
      origin: null, related_specs: [], related_reviews: [], related_decisions: [],
      target_specs: [], blockers: [], resolution_refs: [], superseded_by: null,
      events: [], warnings: [], omitted_relations: 0,
    });
    render(<DebtsView />);
    fireEvent.click(screen.getByRole("button", { name: /Open DEBT-001/ }));
    await screen.findByRole("dialog");

    const close = screen.getByRole("button", { name: "Close debt detail" });
    expect(close.className).toContain("modal-close-button");
    fireEvent.click(close);
    expect(screen.queryByRole("dialog")).toBeNull();

    fireEvent.click(screen.getByRole("button", { name: /Open DEBT-001/ }));
    await screen.findByRole("dialog");
    fireEvent.keyDown(window, { key: "Escape" });
    expect(screen.queryByRole("dialog")).toBeNull();

    fireEvent.click(screen.getByRole("button", { name: /Open DEBT-001/ }));
    const dialog = await screen.findByRole("dialog");
    fireEvent.mouseDown(dialog.parentElement as HTMLElement);
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("restores the original opener focus after a parent rerender", async () => {
    vi.mocked(getDebtContext).mockResolvedValue({
      schema_version: "1", debt: debts[0],
      origin: null, related_specs: [], related_reviews: [], related_decisions: [],
      target_specs: [], blockers: [], resolution_refs: [], superseded_by: null,
      events: [], warnings: [], omitted_relations: 0,
    });
    const { rerender } = render(<DebtsView />);
    const opener = screen.getByRole("button", { name: /Open DEBT-001/ });
    opener.focus();
    fireEvent.click(opener);
    await screen.findByRole("dialog");

    rerender(<DebtsView />);
    fireEvent.click(screen.getByRole("button", { name: "Close debt detail" }));
    expect(document.activeElement).toBe(opener);
  });

  it("keeps keyboard focus inside the detail modal", async () => {
    vi.mocked(getDebtContext).mockResolvedValue({
      schema_version: "1", debt: debts[0],
      origin: null, related_specs: [], related_reviews: [], related_decisions: [],
      target_specs: [], blockers: [], resolution_refs: [], superseded_by: null,
      events: [], warnings: [], omitted_relations: 0,
    });
    render(<DebtsView />);
    fireEvent.click(screen.getByRole("button", { name: /Open DEBT-001/ }));
    const dialog = await screen.findByRole("dialog");
    const close = screen.getByRole("button", { name: "Close debt detail" });
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
    render(<DebtsView />);
    fireEvent.click(screen.getByRole("button", { name: "Refresh" }));
    await waitFor(() => expect(refreshWorkspaceData).toHaveBeenCalled());
  });
});
