import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { SettingsView } from "../components/Settings/SettingsView";
import {
  getHarnessApprovalStatus,
  getHarnessDrift,
  getVerificationManifestStatus,
  planHarnessConfiguration,
  previewVerificationManifest,
  setVerificationManifest,
} from "../lib/commands";

vi.mock("../hooks/useWorkspace", () => ({ useWorkspace: () => ({ state: { sessions: [], currentWorkspace: { project_kit_version: "2.7.3", bundled_kit_version: "2.8.0" } } }) }));
vi.mock("@tauri-apps/api/app", () => ({ getVersion: vi.fn().mockResolvedValue("9.9.9-test") }));
vi.mock("../lib/commands", () => ({
  getHarnessApprovalStatus: vi.fn(), getHarnessDrift: vi.fn(), planHarnessConfiguration: vi.fn(),
  approveHarnessManifest: vi.fn(), revokeHarnessManifestApproval: vi.fn(), applyHarnessConfiguration: vi.fn(),
  probeHarnesses: vi.fn().mockResolvedValue([]), updateHarness: vi.fn(),
  getVerificationManifestStatus: vi.fn(), previewVerificationManifest: vi.fn(),
  setVerificationManifest: vi.fn(), rollbackVerificationManifest: vi.fn(),
}));

describe("SettingsView", () => {
  const writeText = vi.fn().mockResolvedValue(undefined);
  beforeEach(() => { window.location.hash = ""; vi.clearAllMocks(); Object.assign(navigator, { clipboard: { writeText } }); });

  it("renders accessible functional settings tabs without placeholder controls", () => {
    render(<SettingsView />);
    expect(screen.getByRole("heading", { name: "Settings" })).toBeDefined();
    expect(screen.getAllByRole("tab")).toHaveLength(5);
    expect(screen.getByRole("tabpanel")).toBeDefined();
    expect(screen.queryByText("Theme")).toBeNull();
    expect(screen.queryByText("Auto-start agents")).toBeNull();
  });

  it("routes tabs through the settings hash and exposes About versions", async () => {
    render(<SettingsView />);
    fireEvent.click(screen.getByRole("tab", { name: "About" }));
    expect(window.location.hash).toBe("#settings/about");
    expect(screen.getByText("2.7.3")).toBeDefined();
    // The product version follows package/build metadata, never a hardcode.
    await screen.findByText("LMBrain 9.9.9-test");
    expect(screen.queryByText(/2\.8\.0 \(development\)/)).toBeNull();
  });

  it("shows optional setup guidance for an unconfigured project", async () => {
    vi.mocked(getHarnessApprovalStatus).mockResolvedValue({ state: "unconfigured", manifest_digest: null, approved_digest: null, approved_at: null, workspace_fingerprint: "abc" });
    vi.mocked(planHarnessConfiguration).mockRejectedValue(new Error("must not be called"));
    vi.mocked(getHarnessDrift).mockRejectedValue(new Error("must not be called"));
    render(<SettingsView />);
    fireEvent.click(screen.getByRole("tab", { name: "Project environment" }));
    await screen.findByText("No harness manifest");
    expect(screen.getByRole("button", { name: "Refresh" }).style.alignSelf).toBe("flex-start");
    expect(planHarnessConfiguration).not.toHaveBeenCalled();
    expect(getHarnessDrift).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Copy Project Lead prompt" }));
    await screen.findByRole("status");
    expect(screen.getByRole("button", { name: "Copied" })).toBeDefined();
    expect(writeText).toHaveBeenCalledOnce();
  });

  it("supports the legacy harness deep-link as a Settings tab", async () => {
    render(<SettingsView initialTab="harnesses" />);
    await waitFor(() => expect(screen.getByRole("tab", { name: "Harnesses" }).getAttribute("aria-selected")).toBe("true"));
    expect(screen.getByLabelText("Codex executable override")).toBeDefined();
  });

  it("previews and writes verification configuration without approval or status actions", async () => {
    vi.mocked(getVerificationManifestStatus).mockResolvedValue({
      schema_version: "1", state: "absent", manifest_digest: null,
      approved_digest: null, approved_at: null, workspace_fingerprint: "workspace",
      gate_count: 0, issues: [], next_action: "Inspect a preview.", can_rollback: false,
    });
    vi.mocked(previewVerificationManifest).mockResolvedValue({
      schema_version: "1",
      status: {
        schema_version: "1", state: "absent", manifest_digest: null,
        approved_digest: null, approved_at: null, workspace_fingerprint: "workspace",
        gate_count: 0, issues: [], next_action: "Inspect a preview.", can_rollback: false,
      },
      candidates: [{
        gate: {
          id: "test", program: "cargo", args: ["test"], cwd: ".",
          timeout_seconds: 900, output_limit_bytes: 131072, expected_exit_code: 0,
          result_matcher: null, environment: {}, fingerprint_exclude: ["target"],
        },
        provenance: "Cargo.toml", confidence: "high", selected: true,
        environment_policy: "empty environment", mutation_policy: "target excluded",
        security_notes: ["No shell is used."],
      }],
      conflicts: [], guidance: [], proposed_manifest: {
        schema_version: 1,
        gates: [{
          id: "test", program: "cargo", args: ["test"], cwd: ".",
          timeout_seconds: 900, output_limit_bytes: 131072, expected_exit_code: 0,
          result_matcher: null, environment: {}, fingerprint_exclude: ["target"],
        }],
      },
      proposed_toml: "schema_version = 1", proposed_digest: "new-digest",
      current_toml: null, diff: "+ schema_version = 1", discovered_only: true,
    });
    vi.mocked(setVerificationManifest).mockResolvedValue({
      path: ".lmbrain/verification.toml", digest: "new-digest", previous_digest: null,
      approval_required: true, rollback_available: false,
    });

    render(<SettingsView />);
    fireEvent.click(screen.getByRole("tab", { name: "Verification" }));
    await screen.findByText("Manifest: absent");
    expect(screen.queryByRole("button", { name: /approve/i })).toBeNull();

    fireEvent.click(screen.getByRole("button", { name: "Discover / refresh preview" }));
    await screen.findByText("cargo test");
    expect(screen.getByText(/does not approve it/i)).toBeDefined();
    fireEvent.click(screen.getByRole("button", { name: "Create manifest" }));
    await waitFor(() => expect(setVerificationManifest).toHaveBeenCalledWith(
      expect.objectContaining({ schema_version: 1 }),
      null,
    ));
    expect(screen.queryByRole("button", { name: /approve/i })).toBeNull();
  });
});
