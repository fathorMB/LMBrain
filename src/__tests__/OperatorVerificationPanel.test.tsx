import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { OperatorVerificationPanel } from "../components/Spec/OperatorVerificationPanel";
import type { SpecVerificationState, Spec } from "../types";

const getSpecVerification = vi.fn();
const navigateToMock = vi.fn();

vi.mock("../lib/commands", () => ({
  getSpecVerification: (...args: unknown[]) => getSpecVerification(...args),
}));

vi.mock("../hooks/useWorkspace", () => ({
  useWorkspace: () => ({
    navigateTo: navigateToMock,
  }),
}));

const spec: Spec = {
  id: "SPEC-053",
  title: "Operator attestation",
  status: "review",
  priority: "critical",
  area: "verification",
  milestone: "M-07",
  recommended_agent: "AGENT-FULLSTACK-DESKTOP",
  skills: [],
  body: "",
  path: "C:/workspace/.lmbrain/specs/review/SPEC-053.md",
  created: "2026-07-29",
  updated: "2026-07-29",
  tags: [],
  links: [],
  related_tasks: [],
  related_decisions: [],
};

const verification: SpecVerificationState = {
  requirements: [
    {
      id: "HUMAN-PLAYTEST",
      text: "Exercise the completed desktop flow",
      checked: true,
      kind: "operator",
      owner: "operator",
      phase: "before-done",
      evidence: "observation",
      source: "required-verification",
    },
    {
      id: "LEAD-REVIEW",
      text: "Perform independent review",
      checked: false,
      kind: "manual",
      owner: "lead",
      phase: "before-done",
      evidence: "artifact",
      source: "required-verification",
    },
  ],
  attestations: [],
  blockers: [
    {
      requirement_id: "HUMAN-PLAYTEST",
      owner: "operator",
      cause: "checklist item is unchecked",
    },
  ],
};

describe("OperatorVerificationPanel", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    getSpecVerification.mockResolvedValue(verification);
  });

  it("renders read-only verification gates with status badges and Go to Operations action", async () => {
    render(<OperatorVerificationPanel spec={spec} />);

    expect(await screen.findByText("Verification gates")).toBeTruthy();
    expect(screen.getByText("HUMAN-PLAYTEST")).toBeTruthy();
    expect(screen.getByText("LEAD-REVIEW")).toBeTruthy();
    expect(screen.getByText("BLOCKED")).toBeTruthy();
    expect(screen.getByText("PENDING")).toBeTruthy();

    const operationsBtn = screen.getByRole("button", { name: /go to operations/i });
    expect(operationsBtn).toBeTruthy();

    fireEvent.click(operationsBtn);
    expect(navigateToMock).toHaveBeenCalledWith("operations");
  });
});
