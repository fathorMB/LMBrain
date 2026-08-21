import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { OperationsView } from "../components/Operations/OperationsView";
import type { OperatorGate, Spec } from "../types";

const attestOperatorVerification = vi.fn();
const refreshWorkspaceData = vi.fn();
const openSpec = vi.fn();

vi.mock("../lib/commands", () => ({
  attestOperatorVerification: (...args: unknown[]) => attestOperatorVerification(...args),
}));

const mockGates: OperatorGate[] = [
  {
    spec_id: "SPEC-053",
    spec_title: "Operator attestation",
    spec_status: "review",
    spec_path: "specs/review/SPEC-053.md",
    requirement_id: "HUMAN-PLAYTEST",
    text: "Exercise the completed desktop flow",
    kind: "operator",
    evidence_kind: "observation",
    checked: true,
    attested: null,
    blocker: null,
    milestone: "M-07",
    updated: "2026-08-20",
  },
  {
    spec_id: "SPEC-054",
    spec_title: "Release notes verification",
    spec_status: "done",
    spec_path: "specs/done/SPEC-054.md",
    requirement_id: "DOCS-CHECK",
    text: "Verify changelog accuracy",
    kind: "operator",
    evidence_kind: "artifact",
    checked: true,
    attested: {
      schema_version: "1",
      id: "SPEC-054-ATTEST-001",
      requirement_id: "DOCS-CHECK",
      requirement_digest: "digest-123",
      actor_role: "operator",
      actor: "OperatorOne",
      timestamp: "2026-08-21T10:00:00Z",
      result: "passed",
      evidence_ref: "docs:verified",
      evidence_digest: null,
      delegated_by: null,
      delegation_channel: null,
      delegation_authorization: null,
    },
    blocker: null,
    milestone: "M-08",
    updated: "2026-08-21",
  },
  {
    spec_id: "SPEC-055",
    spec_title: "Blocked security verification",
    spec_status: "review",
    spec_path: "specs/review/SPEC-055.md",
    requirement_id: "SEC-AUDIT",
    text: "Perform security smoke test",
    kind: "operator",
    evidence_kind: "test-run",
    checked: false,
    attested: null,
    blocker: "checklist item is unchecked",
    milestone: "M-07",
    updated: "2026-08-19",
  },
];

const mockSpecs: Spec[] = [
  {
    id: "SPEC-053",
    title: "Operator attestation",
    status: "review",
    priority: "critical",
    area: "verification",
    milestone: "M-07",
    recommended_agent: "AGENT-LEAD",
    skills: [],
    body: "",
    path: "specs/review/SPEC-053.md",
    created: "2026-08-01",
    updated: "2026-08-20",
    tags: [],
    links: [],
    related_tasks: [],
    related_decisions: [],
  },
];

vi.mock("../hooks/useWorkspace", () => ({
  useWorkspace: () => ({
    state: {
      operatorGates: mockGates,
      specs: mockSpecs,
      dataRefreshing: false,
    },
    refreshWorkspaceData,
    openSpec,
  }),
}));

describe("OperationsView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
    attestOperatorVerification.mockResolvedValue({
      path: "specs/review/SPEC-053.md",
      created: true,
      attestation: {
        schema_version: "1",
        id: "SPEC-053-ATTEST-002",
        requirement_id: "HUMAN-PLAYTEST",
        requirement_digest: "digest",
        actor_role: "operator",
        actor: "Jane",
        timestamp: "2026-08-21T12:00:00Z",
        result: "passed",
        evidence_ref: "playtest-evidence",
      },
    });
  });

  it("renders operator verification gates list with status pills", () => {
    render(<OperationsView />);

    expect(screen.getByText("Operations")).toBeTruthy();
    expect(screen.getByText("2 pending")).toBeTruthy();
    expect(screen.getByText("SPEC-053")).toBeTruthy();
    expect(screen.getByText("SPEC-054")).toBeTruthy();
    expect(screen.getByText("SPEC-055")).toBeTruthy();
    expect(screen.getByText("ATTESTED")).toBeTruthy();
    expect(screen.getByText("BLOCKED")).toBeTruthy();
    expect(screen.getByText("PENDING")).toBeTruthy();
  });

  it("filters operator gates by status and search text", () => {
    render(<OperationsView />);

    const searchInput = screen.getByPlaceholderText(/filter by spec/i);
    fireEvent.change(searchInput, { target: { value: "changelog" } });

    expect(screen.queryByText("SPEC-053")).toBeNull();
    expect(screen.getByText("SPEC-054")).toBeTruthy();
    expect(screen.queryByText("SPEC-055")).toBeNull();
  });

  it("opens the attest modal, records evidence, and refreshes workspace data", async () => {
    render(<OperationsView />);

    const attestBtns = screen.getAllByRole("button", { name: /attest gate/i });
    fireEvent.click(attestBtns[0]!);

    expect(await screen.findByText("Attest Operator Verification")).toBeTruthy();
    expect(screen.getByText("HUMAN-PLAYTEST:")).toBeTruthy();

    const identityInput = screen.getByLabelText(/operator identity/i);
    const evidenceInput = screen.getByLabelText(/evidence reference/i);

    fireEvent.change(identityInput, { target: { value: "Jane" } });
    fireEvent.change(evidenceInput, { target: { value: "playtest notes: verified locally" } });

    const submitBtn = screen.getByRole("button", { name: /record attestation/i });
    fireEvent.click(submitBtn);

    await waitFor(() => {
      expect(attestOperatorVerification).toHaveBeenCalledWith(
        "specs/review/SPEC-053.md",
        "HUMAN-PLAYTEST",
        "Jane",
        "playtest notes: verified locally",
      );
    });

    expect(refreshWorkspaceData).toHaveBeenCalled();
    expect(localStorage.getItem("lmbrain.operator.identity")).toBe("Jane");
  });

  it("allows navigating to spec from spec ID link", () => {
    render(<OperationsView />);

    const specLink = screen.getByRole("button", { name: "SPEC-053" });
    fireEvent.click(specLink);

    expect(openSpec).toHaveBeenCalledWith(mockSpecs[0]);
  });
});
