import { describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { RoadmapView } from "../components/Roadmap/RoadmapView";
import * as commands from "../lib/commands";

vi.mock("../lib/commands", () => ({
  getRoadmap: vi.fn(),
  getSpecs: vi.fn(),
  getTasks: vi.fn(),
}));

vi.mock("../hooks/useWorkspace", () => ({
  useWorkspace: () => ({
    state: { specs: [], tasks: [] },
    dispatch: vi.fn(),
  }),
}));

describe("RoadmapView", () => {
  it("does not render a legacy temporal target", async () => {
    vi.mocked(commands.getRoadmap).mockResolvedValue({
      title: "Roadmap",
      milestones: [{
        id: "M-02",
        title: "Operator workflow",
        status: "active",
        outcome: "Operator-controlled status writes.",
        specs: [],
        decisions: [],
        risks: [],
        depends_on: null,
        target: "2026-Q4",
      } as never],
    });
    vi.mocked(commands.getSpecs).mockResolvedValue([]);
    vi.mocked(commands.getTasks).mockResolvedValue([]);

    render(<RoadmapView />);

    await waitFor(() => expect(screen.getByText("Operator workflow")).toBeDefined());
    expect(screen.queryByText(/Target:/)).toBeNull();
    expect(screen.queryByText("2026-Q4")).toBeNull();
  });
});
