import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { SettingsView } from "../components/Settings/SettingsView";

vi.mock("../hooks/useWorkspace", () => ({ useWorkspace: () => ({ state: { sessions: [], currentWorkspace: { project_kit_version: "2.7.3", bundled_kit_version: "2.8.0" } } }) }));
vi.mock("@tauri-apps/api/app", () => ({ getVersion: vi.fn().mockResolvedValue("9.9.9-test") }));
vi.mock("../lib/commands", () => ({
  probeHarnesses: vi.fn().mockResolvedValue([]), updateHarness: vi.fn(),
}));

describe("SettingsView", () => {
  beforeEach(() => { window.location.hash = ""; vi.clearAllMocks(); });

  it("renders accessible functional settings tabs without placeholder controls", () => {
    render(<SettingsView />);
    expect(screen.getByRole("heading", { name: "Settings" })).toBeDefined();
    expect(screen.getAllByRole("tab")).toHaveLength(3);
    expect(screen.getByRole("tabpanel")).toBeDefined();
    expect(screen.queryByText("Theme")).toBeNull();
    expect(screen.queryByText("Auto-start agents")).toBeNull();
  });

  it("no longer hosts the governed environment tabs (moved to the Environment page)", () => {
    render(<SettingsView />);
    expect(screen.queryByRole("tab", { name: "Project environment" })).toBeNull();
    expect(screen.queryByRole("tab", { name: "Verification" })).toBeNull();
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

  it("supports the legacy harness deep-link as a Settings tab", async () => {
    render(<SettingsView initialTab="harnesses" />);
    await waitFor(() => expect(screen.getByRole("tab", { name: "Harnesses" }).getAttribute("aria-selected")).toBe("true"));
    expect(screen.getByLabelText("Codex executable override")).toBeDefined();
  });
});
