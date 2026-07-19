import { beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { RepositoryView } from "../components/Repository/RepositoryView";
import * as commands from "../lib/commands";
import type { GitDetails, GitHubDashboard } from "../types";

const mockGitDetails: GitDetails = {
  branch: "main",
  current_commit: "a1b2c3d",
  ahead: 1,
  behind: 2,
  remote_url: "https://github.com/fathorMB/LMBrain.git",
  owner: "fathorMB",
  repo: "LMBrain",
  files: [
    { path: "src/App.tsx", status: "unstaged", diff_target: "unstaged", original_path: null },
    { path: "src/index.css", status: "staged", diff_target: "staged", original_path: null },
  ],
};

const mockGitHubDashboard: GitHubDashboard = {
  has_token: true,
  pull_requests: [
    {
      number: 12,
      title: "Fix sidebar layout",
      html_url: "https://github.com/fathorMB/LMBrain/pull/12",
      state: "open",
      user: "fathorMB",
      draft: false,
      created_at: "2026-07-16T12:00:00Z",
      updated_at: "2026-07-16T12:00:00Z",
    },
  ],
  workflow_runs: [
    {
      id: 999,
      name: "CI Build",
      display_title: "Merge pull request #12",
      head_branch: "main",
      head_sha: "a1b2c3d4e5f6a7b8",
      status: "completed",
      conclusion: "success",
      event: "push",
      run_number: 41,
      run_attempt: 1,
      actor: "fathorMB",
      html_url: "https://github.com/fathorMB/LMBrain/actions/runs/999",
      created_at: "2026-07-17T10:00:00Z",
      updated_at: "2026-07-17T10:05:00Z",
      run_started_at: "2026-07-17T10:00:30Z",
    },
    {
      id: 998,
      name: "Nightly Audit",
      display_title: "Nightly Audit",
      head_branch: "main",
      head_sha: "b2c3d4e5f6a7b8c9",
      status: "completed",
      conclusion: "skipped",
      event: "schedule",
      run_number: 40,
      run_attempt: 1,
      actor: null,
      html_url: "https://github.com/fathorMB/LMBrain/actions/runs/998",
      created_at: "2026-07-17T02:00:00Z",
      updated_at: "2026-07-17T02:00:10Z",
      run_started_at: null,
    },
    {
      id: 997,
      name: "Deploy",
      display_title: "Deploy",
      head_branch: "release/next",
      head_sha: "c3d4e5f6a7b8c9d0",
      status: "in_progress",
      conclusion: null,
      event: "workflow_dispatch",
      run_number: 39,
      run_attempt: 2,
      actor: "fathorMB",
      html_url: "https://github.com/fathorMB/LMBrain/actions/runs/997",
      created_at: "2026-07-17T09:00:00Z",
      updated_at: "2026-07-17T09:01:00Z",
      run_started_at: "2026-07-17T09:00:10Z",
    },
    {
      id: 996,
      name: "CI Build",
      display_title: "Broken change",
      head_branch: "feature/x",
      head_sha: "d4e5f6a7b8c9d0e1",
      status: "completed",
      conclusion: "failure",
      event: "pull_request",
      run_number: 38,
      run_attempt: 1,
      actor: "contributor",
      html_url: "https://github.com/fathorMB/LMBrain/actions/runs/996",
      created_at: "2026-07-16T18:00:00Z",
      updated_at: "2026-07-16T18:04:00Z",
      run_started_at: "2026-07-16T18:00:20Z",
    },
  ],
};

vi.mock("../lib/commands", () => ({
  getGitDetails: vi.fn(async () => mockGitDetails),
  getGitFileDiff: vi.fn(async () => ({ path: "src/App.tsx", diff: "", binary: false, truncated: false })),
  getGitHubPatConfigured: vi.fn(async () => true),
  getGitHubDashboard: vi.fn(async () => mockGitHubDashboard),
  saveGitHubPat: vi.fn(async () => {}),
  deleteGitHubPat: vi.fn(async () => {}),
}));

describe("RepositoryView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(commands.getGitDetails).mockResolvedValue(mockGitDetails);
    vi.mocked(commands.getGitFileDiff).mockResolvedValue({ path: "src/App.tsx", diff: "", binary: false, truncated: false });
    vi.mocked(commands.getGitHubPatConfigured).mockResolvedValue(true);
    vi.mocked(commands.getGitHubDashboard).mockResolvedValue(mockGitHubDashboard);
    vi.mocked(commands.saveGitHubPat).mockResolvedValue(undefined);
    vi.mocked(commands.deleteGitHubPat).mockResolvedValue(undefined);
  });

  it("renders branch name, commit SHA, and tracking offset", async () => {
    render(<RepositoryView />);

    await waitFor(() => {
      expect(screen.getAllByText(/main/).length).toBeGreaterThan(0);
    });

    expect(screen.getAllByText(/a1b2c3d/).length).toBeGreaterThan(0);
    expect(screen.getByText("↑ 1 ahead")).toBeDefined();
    expect(screen.getByText("↓ 2 behind")).toBeDefined();
  });

  it("renders list of modified files with status badges", async () => {
    render(<RepositoryView />);

    await waitFor(() => {
      expect(screen.getByText("src/App.tsx")).toBeDefined();
    });

    expect(screen.getByText("src/index.css")).toBeDefined();
    expect(screen.getAllByText(/unstaged/i).length).toBeGreaterThan(0);
    expect(screen.getAllByText(/staged/i).length).toBeGreaterThan(0);
  });

  it("opens the selected file in the diff viewer", async () => {
    vi.mocked(commands.getGitFileDiff).mockResolvedValueOnce({
      path: "src/App.tsx",
      diff: "@@ -1 +1 @@\n-old\n+new",
      binary: false,
      truncated: false,
    });
    render(<RepositoryView />);

    const fileButton = await screen.findByRole("button", { name: "View diff for src/App.tsx, status unstaged, unstaged" });
    expect(fileButton.className).toContain("repository-file-row");
    fireEvent.click(fileButton);

    await waitFor(() => expect(screen.getByRole("dialog", { name: /src\/App\.tsx/ })).toBeDefined());
    expect(commands.getGitFileDiff).toHaveBeenCalledWith("src/App.tsx", "unstaged");
  });

  it("renders open Pull Requests and Action runs", async () => {
    render(<RepositoryView />);

    await waitFor(() => {
      expect(screen.getAllByText(/#12 Fix sidebar/i).length).toBeGreaterThan(0);
    });

    expect(screen.getAllByText("CI Build").length).toBeGreaterThan(0);
    expect(screen.getAllByText(/success/i).length).toBeGreaterThan(0);
  });

  it("shows runs of every outcome with distinct status labels", async () => {
    render(<RepositoryView />);

    await waitFor(() => {
      expect(screen.getByText("Success")).toBeDefined();
    });

    expect(screen.getByText("Skipped")).toBeDefined();
    expect(screen.getByText("In progress")).toBeDefined();
    expect(screen.getByText("Failure")).toBeDefined();
  });

  it("opens the workflow run details modal with metadata and GitHub link", async () => {
    render(<RepositoryView />);

    const runButton = await screen.findByRole("button", {
      name: "View details for run CI Build, Success, branch main",
    });
    fireEvent.click(runButton);

    const dialog = await screen.findByRole("dialog", { name: /CI Build/ });
    expect(dialog).toBeDefined();
    expect(screen.getByText("push")).toBeDefined();
    expect(screen.getByText("a1b2c3d4e5f6")).toBeDefined();
    expect(screen.getAllByText("fathorMB").length).toBeGreaterThan(0);
    const link = screen.getByRole("link", { name: /open on github/i });
    expect(link.getAttribute("href")).toBe("https://github.com/fathorMB/LMBrain/actions/runs/999");
  });

  it("renders missing run metadata as placeholders without breaking", async () => {
    render(<RepositoryView />);

    const runButton = await screen.findByRole("button", {
      name: "View details for run Nightly Audit, Skipped, branch main",
    });
    fireEvent.click(runButton);

    await screen.findByRole("dialog", { name: /Nightly Audit/ });
    // actor and run_started_at are null → placeholder dashes
    expect(screen.getAllByText("—").length).toBeGreaterThanOrEqual(2);
  });

  it("closes the workflow run modal with Escape", async () => {
    render(<RepositoryView />);

    const runButton = await screen.findByRole("button", {
      name: "View details for run Deploy, In progress, branch release/next",
    });
    fireEvent.click(runButton);

    await screen.findByRole("dialog", { name: /Deploy/ });
    fireEvent.keyDown(window, { key: "Escape" });

    await waitFor(() => {
      expect(screen.queryByRole("dialog", { name: /Deploy/ })).toBeNull();
    });
  });

  it("renders secure PAT credential information", async () => {
    render(<RepositoryView />);

    await waitFor(() => {
      expect(screen.getAllByText(/securely configured/i).length).toBeGreaterThan(0);
    });

    expect(screen.getByText("Delete Token")).toBeDefined();
  });

  it("refreshes repository data on demand", async () => {
    render(<RepositoryView />);

    await waitFor(() => {
      expect(screen.getByText("a1b2c3d")).toBeDefined();
    });

    vi.mocked(commands.getGitDetails).mockResolvedValueOnce({
      ...mockGitDetails,
      branch: "release/3.0.1",
      current_commit: "d4e5f6a",
    });
    fireEvent.click(screen.getByRole("button", { name: /refresh/i }));

    await waitFor(() => {
      expect(screen.getByText("release/3.0.1")).toBeDefined();
      expect(screen.getByText("d4e5f6a")).toBeDefined();
    });
  });

  it("shows an actionable initial-load error", async () => {
    vi.mocked(commands.getGitDetails).mockRejectedValueOnce(new Error("repository unavailable"));
    render(<RepositoryView />);

    await waitFor(() => {
      expect(screen.getByText("repository unavailable")).toBeDefined();
    });
  });

  it("saves a configured GitHub token", async () => {
    vi.mocked(commands.getGitHubPatConfigured)
      .mockResolvedValueOnce(false)
      .mockResolvedValue(true);
    render(<RepositoryView />);

    await waitFor(() => {
      expect(screen.getByText("Setup GitHub PAT Token")).toBeDefined();
    });

    fireEvent.click(screen.getByText("Setup GitHub PAT Token"));
    fireEvent.change(screen.getByPlaceholderText("ghp_..."), { target: { value: "secret-token" } });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => {
      expect(commands.saveGitHubPat).toHaveBeenCalledWith("secret-token");
      expect(screen.getByText("SECURELY CONFIGURED")).toBeDefined();
    });
  });

  it("deletes a configured GitHub token after confirmation", async () => {
    const confirmSpy = vi.spyOn(window, "confirm").mockReturnValue(true);
    render(<RepositoryView />);

    await waitFor(() => {
      expect(screen.getByText("Delete Token")).toBeDefined();
    });

    fireEvent.click(screen.getByText("Delete Token"));

    await waitFor(() => {
      expect(commands.deleteGitHubPat).toHaveBeenCalledOnce();
    });
    confirmSpy.mockRestore();
  });
});
