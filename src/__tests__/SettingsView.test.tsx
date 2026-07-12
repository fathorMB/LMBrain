import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { SettingsView } from "../components/Settings/SettingsView";
import { getHarnessApprovalStatus, getHarnessDrift, planHarnessConfiguration } from "../lib/commands";

vi.mock("../hooks/useWorkspace", () => ({ useWorkspace: () => ({ state: { sessions: [], currentWorkspace: { project_kit_version: "2.7.3", bundled_kit_version: "2.8.0" } } }) }));
vi.mock("../lib/commands", () => ({
  getHarnessApprovalStatus: vi.fn(), getHarnessDrift: vi.fn(), planHarnessConfiguration: vi.fn(),
  approveHarnessManifest: vi.fn(), revokeHarnessManifestApproval: vi.fn(), applyHarnessConfiguration: vi.fn(),
  probeHarnesses: vi.fn().mockResolvedValue([]), updateHarness: vi.fn(),
}));

describe("SettingsView", () => {
  const writeText = vi.fn().mockResolvedValue(undefined);
  beforeEach(() => { window.location.hash = ""; vi.clearAllMocks(); Object.assign(navigator, { clipboard: { writeText } }); });

  it("renders accessible functional settings tabs without placeholder controls", () => {
    render(<SettingsView />);
    expect(screen.getByRole("heading", { name: "Settings" })).toBeDefined();
    expect(screen.getAllByRole("tab")).toHaveLength(4);
    expect(screen.getByRole("tabpanel")).toBeDefined();
    expect(screen.queryByText("Theme")).toBeNull();
    expect(screen.queryByText("Auto-start agents")).toBeNull();
  });

  it("routes tabs through the settings hash and exposes About versions", () => {
    render(<SettingsView />);
    fireEvent.click(screen.getByRole("tab", { name: "About" }));
    expect(window.location.hash).toBe("#settings/about");
    expect(screen.getByText("2.7.3")).toBeDefined();
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
});
