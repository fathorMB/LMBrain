import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { EnvironmentView } from "../components/Environment/EnvironmentView";
import {
  getHarnessApprovalStatus,
  getHarnessDrift,
  getVerificationManifestStatus,
  planHarnessConfiguration,
} from "../lib/commands";

vi.mock("../lib/commands", () => ({
  getHarnessApprovalStatus: vi.fn(),
  getHarnessDrift: vi.fn(),
  getVerificationManifestStatus: vi.fn(),
  planHarnessConfiguration: vi.fn(),
}));

describe("EnvironmentView", () => {
  beforeEach(() => vi.clearAllMocks());

  it("shows both governed statuses read-only with no mutating controls", async () => {
    vi.mocked(getHarnessApprovalStatus).mockResolvedValue({
      state: "approved", manifest_digest: "digest-1", approved_digest: "digest-1",
      approved_at: "2026-08-07T00:00:00Z", approved_by: "project-lead", workspace_fingerprint: "abc",
    });
    vi.mocked(planHarnessConfiguration).mockResolvedValue({
      manifest_digest: "digest-1",
      hosts: [{
        host: "claude-code",
        effective: { enabled: true, required_tools: [], environment: {} },
        supported_capabilities: ["enabled", "required-tools", "environment", "lsp", "browser-mcp"],
        tools: [], lsp: null,
        browser_mcp: {
          provider: "playwright", package_available: true, package_version: "0.0.41",
          browser_runtime_found: true, state: "prerequisite-ready", detail: "provisioned",
        },
        native_files: [{ path: ".mcp.json", owned_paths: ["mcpServers.lmbrain", "mcpServers.lmbrain-browser"], action: "preserved", detail: "already matches effective configuration" }],
        ready: true,
      }],
      has_conflicts: false,
    });
    vi.mocked(getHarnessDrift).mockResolvedValue([]);
    vi.mocked(getVerificationManifestStatus).mockResolvedValue({
      schema_version: "1", state: "approved", manifest_digest: "ver-digest",
      approved_digest: "ver-digest", approved_at: "2026-08-06T10:11:43Z", workspace_fingerprint: "abc",
      gate_count: 10, issues: [], next_action: "spec_verify may execute referenced gates.", can_rollback: true,
    });

    render(<EnvironmentView />);
    await screen.findByText("Approval: approved");
    await screen.findByText("Manifest: approved");
    expect(screen.getByText(/project-lead/)).toBeDefined();
    expect(screen.getByText(/Browser MCP \(playwright\)/)).toBeDefined();

    // Read-only: only Refresh is interactive; no approve/revoke/apply/discover.
    const buttons = screen.getAllByRole("button");
    expect(buttons).toHaveLength(1);
    expect(buttons[0].textContent).toBe("Refresh");
    expect(screen.queryByRole("button", { name: /approve/i })).toBeNull();
    expect(screen.queryByRole("button", { name: /apply/i })).toBeNull();
    expect(screen.queryByRole("button", { name: /revoke/i })).toBeNull();
    expect(screen.queryByRole("button", { name: /discover/i })).toBeNull();
  });

  it("explains the unconfigured state without offering mutations", async () => {
    vi.mocked(getHarnessApprovalStatus).mockResolvedValue({
      state: "unconfigured", manifest_digest: null, approved_digest: null,
      approved_at: null, approved_by: null, workspace_fingerprint: "abc",
    });
    vi.mocked(getVerificationManifestStatus).mockResolvedValue({
      schema_version: "1", state: "absent", manifest_digest: null,
      approved_digest: null, approved_at: null, workspace_fingerprint: "abc",
      gate_count: 0, issues: [], next_action: "Create a manifest via MCP.", can_rollback: false,
    });

    render(<EnvironmentView />);
    await screen.findByText("No harness manifest");
    expect(planHarnessConfiguration).not.toHaveBeenCalled();
    expect(getHarnessDrift).not.toHaveBeenCalled();
    expect(screen.getAllByRole("button")).toHaveLength(1);
  });
});
