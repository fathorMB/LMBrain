import { describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { SessionsView } from "../components/Sessions/SessionsView";
import type { SessionWindowState } from "../types";

const mockWorkspace = vi.hoisted(() => ({
  closeSession: vi.fn(),
  createSession: vi.fn(),
  bringSessionToFront: vi.fn(),
  updateSessionGeometry: vi.fn(),
}));

const highZSession: SessionWindowState = {
  id: "session-1",
  label: "Existing agent session",
  mode: "codex",
  model: null,
  status: "running",
  exit_code: null,
  geometry: {
    x: 20,
    y: 30,
    width: 640,
    height: 360,
  },
  zIndex: 999,
};

vi.mock("../lib/commands", () => ({
  listOllamaModels: vi.fn(),
}));

vi.mock("../hooks/useWorkspace", () => ({
  useWorkspace: () => ({
    state: {
      sessions: [highZSession],
    },
    createSession: mockWorkspace.createSession,
    closeSession: mockWorkspace.closeSession,
    updateSessionGeometry: mockWorkspace.updateSessionGeometry,
    bringSessionToFront: mockWorkspace.bringSessionToFront,
  }),
}));

vi.mock("react-rnd", () => ({
  Rnd: ({
    children,
    style,
  }: {
    children: React.ReactNode;
    style: React.CSSProperties;
  }) => (
    <div data-testid="session-window" style={style}>
      {children}
    </div>
  ),
}));

vi.mock("../components/Sessions/SessionTerminal", () => ({
  SessionTerminal: () => <div data-testid="session-terminal" />,
}));

describe("SessionsView", () => {
  it("renders the new-session modal above existing session windows", () => {
    render(<SessionsView active />);

    fireEvent.click(screen.getByText("New session"));

    const dialog = screen.getByRole("dialog", { name: "Start session" });
    const overlay = dialog.parentElement;
    const sessionWindow = screen.getByTestId("session-window");

    expect(overlay).not.toBeNull();
    expect(Number(overlay?.style.zIndex)).toBeGreaterThan(
      Number(sessionWindow.style.zIndex)
    );
  });
});
