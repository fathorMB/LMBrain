import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { SettingsView } from "../components/Settings/SettingsView";

// Mock the workspace context
vi.mock("../context/WorkspaceContext", () => ({
  useWorkspace: () => ({
    state: {
      screen: "app",
      view: "settings",
      currentWorkspace: null,
      recentWorkspaces: [],
      gitInfo: null,
      pulseData: null,
      tasks: [],
      specs: [],
      reviews: [],
      adrs: [],
      agents: [],
      mcpRecords: [],
      mcpProposals: [],
      handoffs: [],
      wikiTree: null,
      wikiPage: null,
      selectedSpec: null,
      drawerTask: null,
      cmdkOpen: false,
      watcherActive: false,
      loading: false,
      error: null,
    },
    dispatch: vi.fn(),
    openWorkspace: vi.fn(),
    loadAllData: vi.fn(),
    navigateTo: vi.fn(),
    openSpec: vi.fn(),
    openTaskDrawer: vi.fn(),
    closeTaskDrawer: vi.fn(),
    toggleCmdk: vi.fn(),
    closeCmdk: vi.fn(),
    goToPicker: vi.fn(),
  }),
}));

describe("SettingsView", () => {
  it("renders the settings title", () => {
    render(<SettingsView />);
    expect(screen.getByText("Settings")).toBeDefined();
  });

  it("renders appearance section", () => {
    render(<SettingsView />);
    expect(screen.getByText("Appearance")).toBeDefined();
  });

  it("renders theme option", () => {
    render(<SettingsView />);
    expect(screen.getByText("Theme")).toBeDefined();
  });

  it("renders density option", () => {
    render(<SettingsView />);
    expect(screen.getByText("Density")).toBeDefined();
  });

  it("renders agents section", () => {
    render(<SettingsView />);
    expect(screen.getByText("Agents")).toBeDefined();
  });

  it("renders auto-start agents setting", () => {
    render(<SettingsView />);
    expect(screen.getByText("Auto-start agents")).toBeDefined();
  });
});
