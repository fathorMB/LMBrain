import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { GitDiffModal } from "../components/Repository/GitDiffModal";
import * as commands from "../lib/commands";
import type { GitFile } from "../types";

const file: GitFile = {
  path: "src/very/long/path/RepositoryView.tsx",
  status: "unstaged",
  diff_target: "unstaged",
  original_path: null,
};

vi.mock("../lib/commands", () => ({
  getGitFileDiff: vi.fn(),
}));

describe("GitDiffModal", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(commands.getGitFileDiff).mockResolvedValue({
      path: file.path,
      diff: "diff --git a/file b/file\n@@ -1,2 +1,2 @@\n-old\n+new\n context",
      binary: false,
      truncated: false,
    });
  });

  it("loads and renders a safe unified diff with line numbers", async () => {
    render(<GitDiffModal file={file} onClose={vi.fn()} />);
    expect(screen.getByText("Loading diff...")).toBeDefined();

    await waitFor(() => expect(screen.getByText("+new")).toBeDefined());
    expect(commands.getGitFileDiff).toHaveBeenCalledWith(file.path, "unstaged");
    expect(screen.getByText("-old")).toBeDefined();
    expect(screen.getAllByText("1").length).toBeGreaterThan(0);
  });

  it("represents binary, truncated, and failure states", async () => {
    vi.mocked(commands.getGitFileDiff).mockResolvedValueOnce({
      path: file.path,
      diff: "Binary files differ",
      binary: true,
      truncated: true,
    });
    const { unmount } = render(<GitDiffModal file={file} onClose={vi.fn()} />);

    await waitFor(() => expect(screen.getByText(/Binary file changed/)).toBeDefined());
    expect(screen.getByRole("status").textContent).toContain("truncated");
    unmount();

    vi.mocked(commands.getGitFileDiff).mockRejectedValueOnce(new Error("diff unavailable"));
    render(<GitDiffModal file={file} onClose={vi.fn()} />);
    await waitFor(() => expect(screen.getByRole("alert").textContent).toBe("diff unavailable"));
  });

  it("closes from the button, Escape key, and backdrop", () => {
    const onClose = vi.fn();
    render(<GitDiffModal file={file} onClose={onClose} />);

    fireEvent.click(screen.getByRole("button", { name: "Close diff" }));
    fireEvent.keyDown(window, { key: "Escape" });
    fireEvent.mouseDown(screen.getByRole("dialog").parentElement as HTMLElement);

    expect(onClose).toHaveBeenCalledTimes(3);
  });
});
