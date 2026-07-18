import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { RepositoryPicker } from "../components/Picker/RepositoryPicker";

vi.mock("../lib/commands", () => ({
  listRecentWorkspaces: vi.fn().mockResolvedValue([]),
}));

vi.mock("../hooks/useWorkspace", () => ({
  useWorkspace: () => ({
    state: {
      currentWorkspace: null,
      loading: true,
      loadingMessage: "Preparing Pi agent integration...",
      loadingPath: "E:\\Git\\Example",
    },
    openWorkspace: vi.fn(),
  }),
}));

describe("RepositoryPicker workspace preparation", () => {
  it("shows the active loading stage and selected path", () => {
    const { container } = render(<RepositoryPicker />);

    expect(screen.getByRole("status", { name: "Preparing workspace" })).toBeDefined();
    expect(screen.getByText("Preparing Pi agent integration...")).toBeDefined();
    expect(screen.getByText("E:\\Git\\Example")).toBeDefined();
    expect(container.querySelector('img[src="/favicon.svg"]')).not.toBeNull();
  });
});
