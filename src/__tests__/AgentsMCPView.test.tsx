import { describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { AgentsMCPView } from "../components/Agents/AgentsMCPView";
import type { AgentProfile, AgentProposal } from "../types";

const dispatch = vi.fn();

const activeAgent: AgentProfile = {
  id: "AGENT-001",
  title: "Active Specialist",
  status: "active",
  role: "specialist",
  activation: "manual",
  can_implement: true,
  can_review: false,
  domains: null,
  primary_files: null,
  review_focus: null,
  context_pack: null,
  constraints: null,
  body: "",
  path: ".lmbrain/agents/profiles/AGENT-001.md",
  created: "2026-06-28",
  updated: "2026-06-28",
  tags: [],
  links: ["AGENT-PROP-001"],
};

// Agent with v3 specialization metadata
const specializedAgent: AgentProfile = {
  id: "AGENT-FRONTEND-UI",
  title: "Frontend UI Specialist",
  status: "proposed",
  role: "frontend-ui-specialist",
  activation: "manual",
  can_implement: true,
  can_review: false,
  domains: ["frontend", "ui", "react"],
  primary_files: ["src/components", "src/lib"],
  review_focus: ["accessibility", "state-management"],
  context_pack: "spec",
  constraints: [],
  body: "",
  path: ".lmbrain/agents/profiles/AGENT-FRONTEND-UI.md",
  created: "2026-07-02",
  updated: "2026-07-02",
  tags: ["v3", "frontend"],
  links: [],
};

const approvedProposal: AgentProposal = {
  id: "AGENT-PROP-001",
  title: "Approved Specialist Proposal",
  status: "approved",
  proposal_type: null,
  target_profile: null,
  body: "",
  path: ".lmbrain/agents/proposals/AGENT-PROP-001.md",
  created: "2026-06-28",
  updated: "2026-06-28",
  tags: [],
  links: [],
};

const pendingProposal: AgentProposal = {
  id: "AGENT-PROP-002",
  title: "Pending Specialist Proposal",
  status: "proposed",
  proposal_type: null,
  target_profile: null,
  body: "",
  path: ".lmbrain/agents/proposals/AGENT-PROP-002.md",
  created: "2026-06-29",
  updated: "2026-06-29",
  tags: [],
  links: [],
};

const approvedOrphanProposal: AgentProposal = {
  id: "AGENT-PROP-003",
  title: "Approved Without Profile",
  status: "approved",
  proposal_type: null,
  target_profile: null,
  body: "",
  path: ".lmbrain/agents/proposals/AGENT-PROP-003.md",
  created: "2026-06-29",
  updated: "2026-06-29",
  tags: [],
  links: [],
};

// Improvement proposal with v3 fields
const improvementProposal: AgentProposal = {
  id: "AGENT-PROP-IMPROVE-001",
  title: "Improve frontend specialist",
  status: "proposed",
  proposal_type: "improvement",
  target_profile: "AGENT-FRONTEND-UI",
  body: "",
  path: ".lmbrain/agents/proposals/AGENT-PROP-IMPROVE-001.md",
  created: "2026-07-02",
  updated: "2026-07-02",
  tags: ["proposal", "improvement"],
  links: [],
};

vi.mock("../hooks/useWorkspace", () => ({
  useWorkspace: () => ({
    state: {
      agents: [activeAgent, specializedAgent],
      agentProposals: [approvedProposal, pendingProposal, approvedOrphanProposal, improvementProposal],
      mcpRecords: [],
      mcpProposals: [],
    },
    dispatch,
    openDetailArtifact: vi.fn(),
  }),
}));

vi.mock("../lib/commands", () => ({
  getAgents: vi.fn(async () => [activeAgent, specializedAgent]),
  getAgentProposals: vi.fn(async () => [
    approvedProposal,
    pendingProposal,
    approvedOrphanProposal,
    improvementProposal,
  ]),
  getMcpRecords: vi.fn(async () => []),
  getMcpProposals: vi.fn(async () => []),
}));

describe("AgentsMCPView", () => {
  it("hides materialized approvals while keeping unresolved agent proposals visible", async () => {
    render(<AgentsMCPView />);

    await waitFor(() => expect(screen.getByText("Active Specialist")).toBeDefined());

    expect(screen.getByText("Pending Specialist Proposal")).toBeDefined();
    expect(screen.getByText("Approved Without Profile")).toBeDefined();
    expect(screen.queryByText("Approved Specialist Proposal")).toBeNull();
  });

  it("renders domain chips for agents with specialization metadata", async () => {
    render(<AgentsMCPView />);

    await waitFor(() => {
      expect(screen.getByText("Frontend UI Specialist")).toBeDefined();
    });

    // Domain chips should be visible
    expect(screen.getByText("frontend")).toBeDefined();
    expect(screen.getByText("ui")).toBeDefined();
    expect(screen.getByText("react")).toBeDefined();
  });

  it("renders review focus chips for agents with specialization metadata", async () => {
    render(<AgentsMCPView />);

    await waitFor(() => {
      expect(screen.getByText("Frontend UI Specialist")).toBeDefined();
    });

    // Review focus chips should be visible
    expect(screen.getByText("accessibility")).toBeDefined();
    expect(screen.getByText("state-management")).toBeDefined();
  });

  it("renders improvement proposal with target profile label", async () => {
    render(<AgentsMCPView />);

    await waitFor(() => {
      expect(screen.getByText("Improve frontend specialist")).toBeDefined();
    });

    // Improvement proposal should show the target profile
    const targetMatches = screen.getAllByText(/AGENT-FRONTEND-UI/);
    expect(targetMatches.length).toBeGreaterThanOrEqual(1);
    // Should show "Improvement proposal" label
    const labelMatches = screen.getAllByText(/Improvement proposal/);
    expect(labelMatches.length).toBeGreaterThanOrEqual(1);
  });
});
