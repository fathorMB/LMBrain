import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { WorkflowRunModal } from "../components/Repository/WorkflowRunModal";
import type { GitHubWorkflowRun } from "../types";

const run: GitHubWorkflowRun = {
  id: 4242,
  name: "CI Build",
  display_title: "Merge pull request #12",
  head_branch: "main",
  head_sha: "a1b2c3d4e5f6a7b8c9d0",
  status: "completed",
  conclusion: "failure",
  event: "push",
  run_number: 41,
  run_attempt: 2,
  actor: "fathorMB",
  html_url: "https://github.com/fathorMB/LMBrain/actions/runs/4242",
  created_at: "2026-07-17T10:00:00Z",
  updated_at: "2026-07-17T10:05:00Z",
  run_started_at: "2026-07-17T10:00:30Z",
};

describe("WorkflowRunModal", () => {
  it("renders run metadata, status badge, and the GitHub link", () => {
    render(<WorkflowRunModal run={run} onClose={vi.fn()} />);

    const dialog = screen.getByRole("dialog", { name: /CI Build/ });
    expect(dialog).toBeDefined();
    expect(screen.getByText("#41")).toBeDefined();
    expect(screen.getByText("· attempt 2")).toBeDefined();
    expect(screen.getByText("Failure")).toBeDefined();
    expect(screen.getByText("Merge pull request #12")).toBeDefined();
    expect(screen.getByText("main")).toBeDefined();
    expect(screen.getByText("push")).toBeDefined();
    expect(screen.getByText("a1b2c3d4e5f6")).toBeDefined();
    expect(screen.getByText("fathorMB")).toBeDefined();
    expect(screen.getByText("completed / failure")).toBeDefined();

    const link = screen.getByRole("link", { name: /open on github/i });
    expect(link.getAttribute("href")).toBe(run.html_url);
    expect(link.getAttribute("target")).toBe("_blank");
    expect(link.getAttribute("rel")).toContain("noopener");
  });

  it("renders placeholders for missing metadata without crashing", () => {
    const partial: GitHubWorkflowRun = {
      ...run,
      display_title: "",
      head_branch: "",
      event: "",
      run_number: 0,
      run_attempt: 1,
      actor: null,
      conclusion: null,
      status: "queued",
      run_started_at: null,
      created_at: "",
      updated_at: "not-a-date",
      head_sha: "",
    };
    render(<WorkflowRunModal run={partial} onClose={vi.fn()} />);

    expect(screen.getByRole("dialog")).toBeDefined();
    expect(screen.getByText("Queued")).toBeDefined();
    expect(screen.queryByText("#0")).toBeNull();
    expect(screen.getAllByText("—").length).toBeGreaterThanOrEqual(6);
  });

  it("closes from the button, Escape key, and backdrop", () => {
    const onClose = vi.fn();
    render(<WorkflowRunModal run={run} onClose={onClose} />);

    fireEvent.click(screen.getByRole("button", { name: "Close run details" }));
    fireEvent.keyDown(window, { key: "Escape" });
    fireEvent.mouseDown(screen.getByRole("dialog").parentElement as HTMLElement);

    expect(onClose).toHaveBeenCalledTimes(3);
  });
});
