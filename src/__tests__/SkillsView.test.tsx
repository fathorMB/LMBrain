import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { SkillsView } from "../components/Skills/SkillsView";
import type { Skill } from "../types";

const openDetailArtifact = vi.fn();

const activeSkill: Skill = {
  id: "SKILL-001",
  title: "Build and test",
  status: "active",
  scope: "project",
  kind: "verification",
  risk: "medium",
  applies_to: ["AGENT-FULLSTACK-DESKTOP"],
  domains: ["build", "test"],
  commands: ["pnpm test", "cargo test --workspace"],
  requires_operator_approval: false,
  body: "",
  path: ".lmbrain/skills/active/SKILL-001-build-and-test.md",
  created: "2026-07-07",
  updated: "2026-07-07",
  tags: [],
  links: [],
};

const proposedSkill: Skill = {
  id: "SKILL-002",
  title: "Release checklist",
  status: "proposed",
  scope: "project",
  kind: "release",
  risk: "high",
  applies_to: ["all"],
  domains: ["release"],
  commands: [],
  requires_operator_approval: true,
  body: "",
  path: ".lmbrain/skills/proposed/SKILL-002-release-checklist.md",
  created: "2026-07-07",
  updated: "2026-07-07",
  tags: [],
  links: [],
};

vi.mock("../hooks/useWorkspace", () => ({
  useWorkspace: () => ({
    state: {
      skills: [activeSkill, proposedSkill],
      diagnostics: [
        {
          message: "Missing reference: skill SKILL-002 applies to 'AGENT-MISSING'",
          severity: "warning",
          path: "skills/proposed/SKILL-002-release-checklist.md",
        },
      ],
    },
    openDetailArtifact,
  }),
}));

describe("SkillsView", () => {
  it("renders skill cards, diagnostics, and opens artifact details", () => {
    render(<SkillsView />);

    expect(screen.getByText("Build and test")).toBeDefined();
    expect(screen.getByText("Release checklist")).toBeDefined();
    expect(screen.getByText("SKILL-001")).toBeDefined();
    expect(screen.getByText("pnpm test - cargo test --workspace")).toBeDefined();
    expect(screen.getByText(/Missing reference/)).toBeDefined();

    fireEvent.click(screen.getByText("Build and test"));

    expect(openDetailArtifact).toHaveBeenCalledWith({
      title: "Build and test",
      path: ".lmbrain/skills/active/SKILL-001-build-and-test.md",
    });
  });

  it("filters by skill status", () => {
    render(<SkillsView />);

    fireEvent.click(screen.getByRole("button", { name: "proposed" }));

    expect(screen.queryByText("Build and test")).toBeNull();
    expect(screen.getByText("Release checklist")).toBeDefined();
  });
});
