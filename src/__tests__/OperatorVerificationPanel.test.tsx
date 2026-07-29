import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { OperatorVerificationPanel } from "../components/Spec/OperatorVerificationPanel";
import type { SpecVerificationState, Spec } from "../types";

const getSpecVerification = vi.fn();
const attestOperatorVerification = vi.fn();
const setArtifactStatus = vi.fn();

vi.mock("../lib/commands", () => ({
  getSpecVerification: (...arguments_: unknown[]) =>
    getSpecVerification(...arguments_),
  attestOperatorVerification: (...arguments_: unknown[]) =>
    attestOperatorVerification(...arguments_),
  setArtifactStatus: (...arguments_: unknown[]) =>
    setArtifactStatus(...arguments_),
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
    {
      requirement_id: "LEAD-REVIEW",
      owner: "lead",
      cause: "checklist item is unchecked",
    },
  ],
};

describe("OperatorVerificationPanel", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    getSpecVerification.mockResolvedValue(verification);
    attestOperatorVerification.mockResolvedValue({
      path: spec.path,
      created: true,
      attestation: {
        schema_version: "1",
        id: "SPEC-053-ATTEST-001",
        requirement_id: "HUMAN-PLAYTEST",
        requirement_digest: "requirement-digest",
        actor_role: "operator",
        actor: "Moren",
        timestamp: "2026-07-29T14:00:00+02:00",
        result: "passed",
        evidence_ref: "playtest:2026-07-29",
        evidence_digest: "evidence-digest",
      },
    });
  });

  it("records operator evidence without approving or changing spec status", async () => {
    const onAttested = vi.fn().mockResolvedValue(undefined);
    render(
      <OperatorVerificationPanel spec={spec} onAttested={onAttested} />,
    );

    expect(
      await screen.findByText(
        /does not approve the spec or change its status/i,
      ),
    ).toBeTruthy();
    expect(screen.getByText("owner=lead")).toBeTruthy();

    fireEvent.change(screen.getByLabelText("Attestor identity"), {
      target: { value: "Moren" },
    });
    fireEvent.change(screen.getByLabelText("Evidence reference"), {
      target: { value: "playtest:2026-07-29" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Attest evidence" }));

    await waitFor(() => {
      expect(attestOperatorVerification).toHaveBeenCalledWith(
        spec.path,
        "HUMAN-PLAYTEST",
        "Moren",
        "playtest:2026-07-29",
      );
    });
    expect(onAttested).toHaveBeenCalledTimes(1);
    expect(setArtifactStatus).not.toHaveBeenCalled();
    expect(spec.status).toBe("review");
  });

  it("requires an explicit attestor and evidence reference", async () => {
    render(
      <OperatorVerificationPanel
        spec={spec}
        onAttested={vi.fn().mockResolvedValue(undefined)}
      />,
    );

    const button = await screen.findByRole("button", {
      name: "Attest evidence",
    });
    expect(button.hasAttribute("disabled")).toBe(true);

    fireEvent.change(screen.getByLabelText("Attestor identity"), {
      target: { value: "Moren" },
    });
    expect(button.hasAttribute("disabled")).toBe(true);

    fireEvent.change(screen.getByLabelText("Evidence reference"), {
      target: { value: "playtest:2026-07-29" },
    });
    expect(button.hasAttribute("disabled")).toBe(false);
  });
});
