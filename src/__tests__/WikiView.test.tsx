import { beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import {
  WorkspaceContext,
  type WorkspaceContextValue,
  type WorkspaceState,
} from "../context/WorkspaceContext";
import type { WikiPage, WikiTree } from "../types";
import { WikiView } from "../components/Wiki/WikiView";

const mocks = vi.hoisted(() => ({
  getWikiTree: vi.fn(),
  getWikilinkIndex: vi.fn(),
  getWikiPage: vi.fn(),
}));

vi.mock("../lib/commands", () => ({
  getWikiTree: mocks.getWikiTree,
  getWikilinkIndex: mocks.getWikilinkIndex,
  getWikiPage: mocks.getWikiPage,
}));

const sourcePage: WikiPage = {
  path: ".lmbrain/knowledge/source.md",
  name: "source",
  content_html: "See [[EXISTING]] and [[MISSING]].",
  frontmatter: {},
  wikilinks: ["EXISTING", "MISSING"],
  backlinks: [],
  updated: null,
  word_count: 4,
};

const existingPage: WikiPage = {
  path: ".lmbrain/knowledge/EXISTING.md",
  name: "EXISTING",
  content_html: "Existing target.",
  frontmatter: {},
  wikilinks: [],
  backlinks: [],
  updated: null,
  word_count: 2,
};

const wikiTree: WikiTree = {
  root: {
    name: ".lmbrain",
    path: ".lmbrain",
    kind: "folder",
    count: 2,
    children: [
      {
        name: "knowledge",
        path: ".lmbrain/knowledge",
        kind: "knowledge",
        count: 2,
        children: [
          {
            name: "source",
            path: ".lmbrain/knowledge/source.md",
            kind: "file",
            count: null,
            children: [],
          },
          {
            name: "EXISTING",
            path: ".lmbrain/knowledge/EXISTING.md",
            kind: "file",
            count: null,
            children: [],
          },
        ],
      },
    ],
  },
};

function createWorkspaceState(): WorkspaceState {
  return {
    screen: "app",
    view: "wiki",
    currentWorkspace: {
      path: "C:/workspace",
      name: "workspace",
      kit_version: "1.0.0",
      health: "ok",
      diagnostics: [],
      branch: null,
      is_clean: null,
      spec_count: 0,
      task_count: 0,
      decision_count: 0,
      agent_count: 0,
    },
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
    wikiTree,
    wikiPage: null,
    selectedSpec: null,
    drawerTask: null,
    cmdkOpen: false,
    watcherActive: false,
    loading: false,
    error: null,
  };
}

function renderWikiView() {
  const context: WorkspaceContextValue = {
    state: createWorkspaceState(),
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
  };

  return render(
    <WorkspaceContext.Provider value={context}>
      <WikiView />
    </WorkspaceContext.Provider>
  );
}

describe("WikiView link-resolution integration", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.getWikiTree.mockResolvedValue(wikiTree);
    // Both keys are present: this recreates the former defect exactly.
    mocks.getWikilinkIndex.mockResolvedValue({
      existing: ["knowledge/source.md"],
      missing: ["knowledge/source.md"],
    });
    mocks.getWikiPage.mockImplementation(async (path: string) => {
      if (path.endsWith("source.md")) return sourcePage;
      if (path.endsWith("EXISTING.md")) return existingPage;
      throw new Error(`Unexpected page request: ${path}`);
    });
  });

  it("uses the WikiTree—not outbound-link keys—to distinguish existing and missing targets", async () => {
    renderWikiView();

    await waitFor(() => expect(mocks.getWikiTree).toHaveBeenCalledTimes(1));
    fireEvent.click(screen.getByText("source"));

    const existing = await screen.findByTitle("Navigate to EXISTING");
    const missing = screen.getByTitle("Unresolved link: MISSING");

    expect(existing.getAttribute("role")).toBe("button");
    expect(existing.getAttribute("tabindex")).toBe("0");
    expect(missing.getAttribute("role")).toBeNull();
    expect(missing.getAttribute("tabindex")).toBeNull();
    expect(missing.style.cursor).toBe("default");

    fireEvent.click(missing);
    expect(mocks.getWikiPage).toHaveBeenCalledTimes(1);

    fireEvent.click(existing);
    await waitFor(() =>
      expect(mocks.getWikiPage).toHaveBeenCalledWith(
        "C:/workspace/.lmbrain/knowledge/EXISTING.md"
      )
    );
  });
});

describe("WikiView collapsible folders", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.getWikiTree.mockResolvedValue(wikiTree);
    mocks.getWikilinkIndex.mockResolvedValue({});
    mocks.getWikiPage.mockResolvedValue(sourcePage);
  });

  it("handles folder collapse and expand", async () => {
    const customWikiTree: WikiTree = {
      root: {
        name: ".lmbrain",
        path: ".lmbrain",
        kind: "folder",
        count: 1,
        children: [
          {
            name: "knowledge",
            path: ".lmbrain/knowledge",
            kind: "knowledge",
            count: 1,
            children: [
              {
                name: "deep_folder",
                path: ".lmbrain/knowledge/deep_folder",
                kind: "folder",
                count: 1,
                children: [
                  {
                    name: "deep_file",
                    path: ".lmbrain/knowledge/deep_folder/deep_file.md",
                    kind: "file",
                    count: null,
                    children: [],
                  },
                ],
              },
            ],
          },
        ],
      },
    };
    mocks.getWikiTree.mockResolvedValue(customWikiTree);

    const context: WorkspaceContextValue = {
      state: {
        ...createWorkspaceState(),
        wikiTree: customWikiTree,
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
    };

    render(
      <WorkspaceContext.Provider value={context}>
        <WikiView />
      </WorkspaceContext.Provider>
    );

    await waitFor(() => expect(mocks.getWikiTree).toHaveBeenCalledTimes(1));

    // 'deep_folder' has depth 2, so it is collapsed by default. 'deep_file' should not be visible.
    expect(screen.queryByText("deep_file")).toBeNull();

    // Verify accessibility attributes on 'deep_folder'
    const deepFolderRow = screen.getByText("deep_folder").closest("[role='button']");
    expect(deepFolderRow).not.toBeNull();
    expect(deepFolderRow?.getAttribute("aria-expanded")).toBe("false");
    expect(deepFolderRow?.getAttribute("tabindex")).toBe("0");

    // Click to expand
    fireEvent.click(screen.getByText("deep_folder"));
    expect(deepFolderRow?.getAttribute("aria-expanded")).toBe("true");

    // 'deep_file' should now be visible
    expect(screen.getByText("deep_file")).not.toBeNull();

    // Click to collapse again
    fireEvent.click(screen.getByText("deep_folder"));
    expect(deepFolderRow?.getAttribute("aria-expanded")).toBe("false");
    expect(screen.queryByText("deep_file")).toBeNull();
  });
});

