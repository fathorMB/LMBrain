import { render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { McpView } from "../components/Agents/McpView";

const workspace = vi.hoisted(() => ({
  state: { mcpRecords: [], mcpProposals: [] },
  dispatch: vi.fn(),
}));

const commands = vi.hoisted(() => ({
  getMcpRecords: vi.fn(),
  getMcpProposals: vi.fn(),
}));

vi.mock("../hooks/useWorkspace", () => ({
  useWorkspace: () => workspace,
}));

vi.mock("../lib/commands", () => commands);

describe("McpView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    workspace.state.mcpRecords = [];
    workspace.state.mcpProposals = [];
    commands.getMcpRecords.mockResolvedValue([]);
    commands.getMcpProposals.mockResolvedValue([]);
  });

  it("explains a valid empty project-specific MCP state", async () => {
    render(<McpView />);

    await waitFor(() => expect(screen.getByText("No project MCP specifications found.")).toBeDefined());
    expect(screen.getByText(/does not declare project-specific MCP integrations/)).toBeDefined();
    expect(screen.getByText(/built-in section below lists the tools/)).toBeDefined();
  });

  it("shows project specifications separately from built-in tools", async () => {
    workspace.state.mcpRecords = [{
      id: "MCP-001",
      title: "Issue tracker integration",
      status: "specified",
      body: "",
      path: ".lmbrain/mcp/specs/MCP-001.md",
      created: "2026-07-31",
      updated: "2026-07-31",
      tags: [],
      links: [],
    }];
    commands.getMcpRecords.mockResolvedValue(workspace.state.mcpRecords);

    render(<McpView />);

    await waitFor(() => expect(screen.getByText("Issue tracker integration")).toBeDefined());
    expect(screen.getByText("Project MCP specifications")).toBeDefined();
    expect(screen.getByText("spec_ready")).toBeDefined();
  });

  it("reports load failures and offers a retry", async () => {
    commands.getMcpRecords.mockRejectedValue(new Error("backend unavailable"));

    render(<McpView />);

    await waitFor(() => expect(screen.getByRole("alert").textContent).toBe("Unable to load project MCP specifications."));
    expect(screen.getByRole("button", { name: "Retry" })).toBeDefined();
  });
});
