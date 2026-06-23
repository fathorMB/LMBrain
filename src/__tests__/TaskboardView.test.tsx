import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { TaskboardView } from "../components/Taskboard/TaskboardView";
import * as commands from "../lib/commands";
import type { Task } from "../types";

vi.mock("../lib/commands", () => ({
  getTasks: vi.fn(),
  getDiagnostics: vi.fn(),
}));

const baseTask: Task = {
  id: "TASK-001",
  title: "A task in the wrong folder",
  status: "planned",
  priority: "High",
  area: "desktop",
  milestone: null,
  spec: null,
  dependencies: [],
  criteria: [],
  activity: [],
  block_reason: null,
  body: "",
  path: "C:/ws/.lmbrain/tasks/planned/TASK-001.md",
  created: "2026-06-22",
  updated: "2026-06-22",
  tags: [],
  links: [],
};

const dispatch = vi.fn();

vi.mock("../hooks/useWorkspace", () => ({
  useWorkspace: () => ({
    state: { tasks: [baseTask] },
    dispatch,
    openTaskDrawer: vi.fn(),
  }),
}));

describe("TaskboardView status mismatch", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(commands.getTasks).mockResolvedValue([baseTask]);
  });

  it("surfaces a folder/frontmatter status mismatch on the card", async () => {
    vi.mocked(commands.getDiagnostics).mockResolvedValue([
      {
        message:
          "Status mismatch: file is in 'tasks/planned' but frontmatter status is 'done'",
        severity: "warning",
        path: "tasks/planned/TASK-001.md",
      },
    ]);

    render(<TaskboardView />);

    await waitFor(() =>
      expect(screen.getByText("status: done ≠ folder: planned")).toBeDefined()
    );
  });

  it("shows no mismatch badge when folder and frontmatter agree", async () => {
    vi.mocked(commands.getDiagnostics).mockResolvedValue([]);

    render(<TaskboardView />);

    await waitFor(() => expect(commands.getDiagnostics).toHaveBeenCalled());
    expect(screen.queryByText(/≠ folder:/)).toBeNull();
  });
});
