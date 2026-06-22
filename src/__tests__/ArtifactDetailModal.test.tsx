import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { ArtifactDetailModal } from "../components/Layout/ArtifactDetailModal";
import * as commands from "../lib/commands";

vi.mock("../lib/commands", () => ({
  parseMarkdown: vi.fn(),
  setArtifactStatus: vi.fn(),
}));

const mockDispatch = vi.fn();
const mockLoadAllData = vi.fn();

vi.mock("../hooks/useWorkspace", () => ({
  useWorkspace: () => ({
    state: {
      detailArtifact: {
        title: "Test ADR",
        path: "E:/workspace/.lmbrain/decisions/ADR-001.md",
      },
    },
    dispatch: mockDispatch,
    loadAllData: mockLoadAllData,
  }),
}));

describe("ArtifactDetailModal", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("loads and renders the markdown body", async () => {
    vi.mocked(commands.parseMarkdown).mockResolvedValue({
      path: "E:/workspace/.lmbrain/decisions/ADR-001.md",
      frontmatter: {},
      body: "This is the markdown body content.",
      wikilinks: [],
      diagnostics: [],
    });

    render(<ArtifactDetailModal />);

    expect(screen.getByText("Loading content...")).toBeDefined();

    await waitFor(() => {
      expect(screen.getByText("Test ADR")).toBeDefined();
      expect(screen.getByText("This is the markdown body content.")).toBeDefined();
    });
  });

  it("calls dispatch when close button is clicked", async () => {
    vi.mocked(commands.parseMarkdown).mockResolvedValue({
      path: "E:/workspace/.lmbrain/decisions/ADR-001.md",
      frontmatter: {},
      body: "Body",
      wikilinks: [],
      diagnostics: [],
    });

    render(<ArtifactDetailModal />);

    await waitFor(() => {
      expect(screen.getByLabelText("Close modal")).toBeDefined();
    });

    fireEvent.click(screen.getByLabelText("Close modal"));
    expect(mockDispatch).toHaveBeenCalledWith({
      type: "SET_DETAIL_ARTIFACT",
      artifact: null,
    });
  });

  it("calls dispatch on pressing Escape key", async () => {
    vi.mocked(commands.parseMarkdown).mockResolvedValue({
      path: "E:/workspace/.lmbrain/decisions/ADR-001.md",
      frontmatter: {},
      body: "Body",
      wikilinks: [],
      diagnostics: [],
    });

    render(<ArtifactDetailModal />);

    await waitFor(() => {
      expect(screen.getByText("Test ADR")).toBeDefined();
    });

    fireEvent.keyDown(window, { key: "Escape" });
    expect(mockDispatch).toHaveBeenCalledWith({
      type: "SET_DETAIL_ARTIFACT",
      artifact: null,
    });
  });

  it("renders Approve and Reject buttons when status is proposed", async () => {
    vi.mocked(commands.parseMarkdown).mockResolvedValue({
      path: "E:/workspace/.lmbrain/decisions/ADR-001.md",
      frontmatter: { id: "ADR-001", status: "proposed" },
      body: "Proposed ADR content",
      wikilinks: [],
      diagnostics: [],
    });

    render(<ArtifactDetailModal />);

    await waitFor(() => {
      expect(screen.getByText("Approve")).toBeDefined();
      expect(screen.getByText("Reject")).toBeDefined();
    });

    // Test click Reject triggers confirmation
    fireEvent.click(screen.getByText("Reject"));
    await waitFor(() => {
      expect(screen.getByText("Confirm Rejection?")).toBeDefined();
      expect(screen.getByText("Cancel")).toBeDefined();
      expect(screen.getByText("Yes, Confirm")).toBeDefined();
    });
  });

  it("renders corrective prompt banner when status is rejected", async () => {
    vi.mocked(commands.parseMarkdown).mockResolvedValue({
      path: "E:/workspace/.lmbrain/decisions/ADR-001.md",
      frontmatter: { id: "ADR-001", status: "rejected" },
      body: "Rejected ADR content",
      wikilinks: [],
      diagnostics: [],
    });

    render(<ArtifactDetailModal />);

    await waitFor(() => {
      expect(screen.getByText("Artifact Rejected")).toBeDefined();
      expect(screen.getByText(/Please revise the/)).toBeDefined();
    });
  });
});
