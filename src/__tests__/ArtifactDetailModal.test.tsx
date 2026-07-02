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

  it("renders Approve and Reject buttons when status is proposed for ADR", async () => {
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

  it.each([
    ["Approve", "accepted"],
    ["Reject", "rejected"],
  ])("refreshes the modal after ADR %s", async (action, resultingStatus) => {
    vi.mocked(commands.parseMarkdown)
      .mockResolvedValueOnce({
        path: "E:/workspace/.lmbrain/decisions/ADR-001.md",
        frontmatter: { id: "ADR-001", status: "proposed" },
        body: "Proposed ADR content",
        wikilinks: [],
        diagnostics: [],
      })
      .mockResolvedValueOnce({
        path: "E:/workspace/.lmbrain/decisions/ADR-001.md",
        frontmatter: { id: "ADR-001", status: resultingStatus },
        body: "Updated ADR content",
        wikilinks: [],
        diagnostics: [],
      });
    vi.mocked(commands.setArtifactStatus).mockResolvedValue("E:/workspace/.lmbrain/decisions/ADR-001.md");

    render(<ArtifactDetailModal />);

    await waitFor(() => expect(screen.getByText(action)).toBeDefined());
    fireEvent.click(screen.getByText(action));
    fireEvent.click(screen.getByText("Yes, Confirm"));

    await waitFor(() => {
      expect(screen.queryByText("Approve")).toBeNull();
      expect(screen.queryByText("Reject")).toBeNull();
      expect(screen.getByText("Updated ADR content")).toBeDefined();
    });
  });

  // ─── SPEC-026-A: Governance tests ─────────────────────────────────

  it("shows governance notice and no Approve button for backlog spec", async () => {
    vi.mocked(commands.parseMarkdown).mockResolvedValue({
      path: "E:/workspace/.lmbrain/specs/backlog/SPEC-001.md",
      frontmatter: { id: "SPEC-001", status: "backlog" },
      body: "Backlog spec content",
      wikilinks: [],
      diagnostics: [],
    });

    render(<ArtifactDetailModal />);

    await waitFor(() => {
      expect(screen.getByText("Spec Approval")).toBeDefined();
      expect(screen.getByText(/Spec approval is performed by the Project Lead/)).toBeDefined();
    });

    // No Approve button
    expect(screen.queryByText("Approve")).toBeNull();
  });

  it("shows governance notice and no Approve button for proposed agent profile", async () => {
    vi.mocked(commands.parseMarkdown).mockResolvedValue({
      path: "E:/workspace/.lmbrain/agents/profiles/AGENT-TEST.md",
      frontmatter: { id: "AGENT-TEST", status: "proposed" },
      body: "Proposed agent profile content",
      wikilinks: [],
      diagnostics: [],
    });

    render(<ArtifactDetailModal />);

    await waitFor(() => {
      expect(screen.getByText("Agent Profile Activation")).toBeDefined();
      expect(screen.getByText(/Agent profile activation is performed through the Project Lead workflow/)).toBeDefined();
    });

    // No Approve button
    expect(screen.queryByText("Approve")).toBeNull();
  });

  it("generates spec approval prompt with correct context", async () => {
    vi.mocked(commands.parseMarkdown).mockResolvedValue({
      path: "E:/workspace/.lmbrain/specs/backlog/SPEC-001.md",
      frontmatter: { id: "SPEC-001", status: "backlog" },
      body: "Backlog spec content",
      wikilinks: [],
      diagnostics: [],
    });

    render(<ArtifactDetailModal />);

    await waitFor(() => expect(screen.getByText("Spec Approval")).toBeDefined());

    const textareas = screen.getAllByRole("textbox");
    expect(textareas.length).toBeGreaterThanOrEqual(1);
    const promptText = textareas[textareas.length - 1].textContent || (textareas[textareas.length - 1] as HTMLTextAreaElement).value;
    expect(promptText).toContain("SPEC-001");
    expect(promptText).toContain("backlog → ready");
    expect(promptText).toContain("AGENT.md");
    expect(promptText).toContain("CONTRACT.md");
    expect(promptText).toContain("QUALITY.md");
    expect(promptText).toContain("operator explicitly asked");
  });

  it("generates agent activation prompt with correct context", async () => {
    vi.mocked(commands.parseMarkdown).mockResolvedValue({
      path: "E:/workspace/.lmbrain/agents/profiles/AGENT-TEST.md",
      frontmatter: { id: "AGENT-TEST", status: "proposed" },
      body: "Proposed agent profile content",
      wikilinks: [],
      diagnostics: [],
    });

    render(<ArtifactDetailModal />);

    await waitFor(() => expect(screen.getByText("Agent Profile Activation")).toBeDefined());

    const textareas = screen.getAllByRole("textbox");
    expect(textareas.length).toBeGreaterThanOrEqual(1);
    const promptText = textareas[textareas.length - 1].textContent || (textareas[textareas.length - 1] as HTMLTextAreaElement).value;
    expect(promptText).toContain("AGENT-TEST");
    expect(promptText).toContain("proposed → active");
    expect(promptText).toContain("AGENT.md");
    expect(promptText).toContain("CONTRACT.md");
    expect(promptText).toContain("QUALITY.md");
    expect(promptText).toContain("operator explicitly asked");
  });

  it("still shows approve/reject for ADR (unaffected artifact kind)", async () => {
    vi.mocked(commands.parseMarkdown).mockResolvedValue({
      path: "E:/workspace/.lmbrain/decisions/ADR-005.md",
      frontmatter: { id: "ADR-005", status: "proposed" },
      body: "ADR content",
      wikilinks: [],
      diagnostics: [],
    });

    render(<ArtifactDetailModal />);

    await waitFor(() => {
      expect(screen.getByText("Approve")).toBeDefined();
      expect(screen.getByText("Reject")).toBeDefined();
    });
  });

  it("still shows approve/reject for agent proposals (unaffected artifact kind)", async () => {
    vi.mocked(commands.parseMarkdown).mockResolvedValue({
      path: "E:/workspace/.lmbrain/agents/proposals/AGENT-PROP-001.md",
      frontmatter: { id: "AGENT-PROP-001", status: "proposed" },
      body: "Agent proposal content",
      wikilinks: [],
      diagnostics: [],
    });

    render(<ArtifactDetailModal />);

    await waitFor(() => {
      expect(screen.getByText("Approve")).toBeDefined();
      expect(screen.getByText("Reject")).toBeDefined();
    });

    expect(screen.queryByText("Agent Profile Activation")).toBeNull();
  });

  it("shows no Approve button and no misleading prompt for ready spec", async () => {
    vi.mocked(commands.parseMarkdown).mockResolvedValue({
      path: "E:/workspace/.lmbrain/specs/ready/SPEC-002.md",
      frontmatter: { id: "SPEC-002", status: "ready" },
      body: "Ready spec content",
      wikilinks: [],
      diagnostics: [],
    });

    render(<ArtifactDetailModal />);

    await waitFor(() => {
      expect(screen.getByText("Ready spec content")).toBeDefined();
    });

    // No Approve button for any spec
    expect(screen.queryByText("Approve")).toBeNull();
    // No governance prompt for non-backlog specs
    expect(screen.queryByText("Spec Approval")).toBeNull();
  });

  it("shows no Approve button and no misleading prompt for inactive agent profile", async () => {
    vi.mocked(commands.parseMarkdown).mockResolvedValue({
      path: "E:/workspace/.lmbrain/agents/profiles/AGENT-INACTIVE.md",
      frontmatter: { id: "AGENT-INACTIVE", status: "inactive" },
      body: "Inactive agent profile content",
      wikilinks: [],
      diagnostics: [],
    });

    render(<ArtifactDetailModal />);

    await waitFor(() => {
      expect(screen.getByText("Inactive agent profile content")).toBeDefined();
    });

    // No Approve button for any agent profile
    expect(screen.queryByText("Approve")).toBeNull();
    // No governance prompt for non-proposed profiles
    expect(screen.queryByText("Agent Profile Activation")).toBeNull();
  });
});
