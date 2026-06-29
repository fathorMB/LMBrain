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
  body: "",
  path: ".lmbrain/agents/profiles/AGENT-001.md",
  created: "2026-06-28",
  updated: "2026-06-28",
  tags: [],
  links: ["AGENT-PROP-001"],
};

const approvedProposal: AgentProposal = {
  id: "AGENT-PROP-001",
  title: "Approved Specialist Proposal",
  status: "approved",
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
  body: "",
  path: ".lmbrain/agents/proposals/AGENT-PROP-003.md",
  created: "2026-06-29",
  updated: "2026-06-29",
  tags: [],
  links: [],
};

vi.mock("../hooks/useWorkspace", () => ({
  useWorkspace: () => ({
    state: {
      agents: [activeAgent],
      agentProposals: [approvedProposal, pendingProposal, approvedOrphanProposal],
      mcpRecords: [],
      mcpProposals: [],
    },
    dispatch,
    openDetailArtifact: vi.fn(),
  }),
}));

vi.mock("../lib/commands", () => ({
  getAgents: vi.fn(async () => [activeAgent]),
  getAgentProposals: vi.fn(async () => [
    approvedProposal,
    pendingProposal,
    approvedOrphanProposal,
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
});
