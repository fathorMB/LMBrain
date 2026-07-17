import { describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { RepositoryView } from "../components/Repository/RepositoryView";
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
    { path: "src/App.tsx", status: "unstaged", original_path: null },
    { path: "src/index.css", status: "staged", original_path: null },
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
      head_branch: "main",
      head_sha: "a1b2c3d",
      status: "completed",
      conclusion: "success",
      html_url: "https://github.com/fathorMB/LMBrain/actions/runs/999",
      created_at: "2026-07-17T10:00:00Z",
    },
  ],
};

vi.mock("../lib/commands", () => ({
  getGitDetails: vi.fn(async () => mockGitDetails),
  getGitHubPatConfigured: vi.fn(async () => true),
  getGitHubDashboard: vi.fn(async () => mockGitHubDashboard),
  saveGitHubPat: vi.fn(async () => {}),
  deleteGitHubPat: vi.fn(async () => {}),
}));

describe("RepositoryView", () => {
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

  it("renders open Pull Requests and Action runs", async () => {
    render(<RepositoryView />);

    await waitFor(() => {
      expect(screen.getAllByText(/#12 Fix sidebar/i).length).toBeGreaterThan(0);
    });

    expect(screen.getByText("CI Build")).toBeDefined();
    expect(screen.getAllByText(/success/i).length).toBeGreaterThan(0);
  });

  it("renders secure PAT credential information", async () => {
    render(<RepositoryView />);

    await waitFor(() => {
      expect(screen.getAllByText(/securely configured/i).length).toBeGreaterThan(0);
    });

    expect(screen.getByText("Delete Token")).toBeDefined();
  });
});
