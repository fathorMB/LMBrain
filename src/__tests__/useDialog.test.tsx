import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { useState } from "react";
import { useDialog } from "../hooks/useDialog";
import { Modal } from "../components/Shared/Modal";

function TestDialog({
  isOpen = true,
  onClose,
}: {
  isOpen?: boolean;
  onClose: () => void;
}) {
  const { dialogRef, handleKeyDown } = useDialog<HTMLDivElement>({
    isOpen,
    onClose,
  });

  if (!isOpen) return null;

  return (
    <div role="presentation" onKeyDown={handleKeyDown} onMouseDown={onClose}>
      <div
        ref={dialogRef}
        role="dialog"
        aria-label="Test Dialog"
        tabIndex={-1}
        onMouseDown={(e) => e.stopPropagation()}
      >
        <h2>Dialog Title</h2>
        <button type="button">First button</button>
        <button type="button" data-safe-action>
          Safe action
        </button>
        <button type="button">Last button</button>
      </div>
    </div>
  );
}

describe("useDialog and Modal primitives", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("focuses data-safe-action if present on mount and restores previous focus on unmount", () => {
    const trigger = document.createElement("button");
    trigger.textContent = "Open";
    document.body.appendChild(trigger);
    trigger.focus();
    expect(document.activeElement).toBe(trigger);

    const onClose = vi.fn();
    const { unmount } = render(<TestDialog isOpen={true} onClose={onClose} />);

    const safeBtn = screen.getByRole("button", { name: "Safe action" });
    expect(document.activeElement).toBe(safeBtn);

    unmount();
    expect(document.activeElement).toBe(trigger);
    document.body.removeChild(trigger);
  });

  it("traps Tab and Shift+Tab focus within dialog", () => {
    const onClose = vi.fn();
    render(<TestDialog isOpen={true} onClose={onClose} />);

    const first = screen.getByRole("button", { name: "First button" });
    const last = screen.getByRole("button", { name: "Last button" });

    // Focus last button and press Tab -> should cycle to first
    last.focus();
    expect(document.activeElement).toBe(last);
    fireEvent.keyDown(window, { key: "Tab" });
    expect(document.activeElement).toBe(first);

    // Focus first button and press Shift+Tab -> should cycle to last
    first.focus();
    expect(document.activeElement).toBe(first);
    fireEvent.keyDown(window, { key: "Tab", shiftKey: true });
    expect(document.activeElement).toBe(last);
  });

  it("calls onClose when Escape key is pressed", () => {
    const onClose = vi.fn();
    render(<TestDialog isOpen={true} onClose={onClose} />);

    fireEvent.keyDown(window, { key: "Escape" });
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("renders Modal component with header, title, close button and children", () => {
    const onClose = vi.fn();
    render(
      <Modal
        isOpen={true}
        onClose={onClose}
        title="My Modal"
        subtitle="Subtitle text"
      >
        <p>Modal content</p>
      </Modal>
    );

    expect(screen.getByRole("dialog", { name: "My Modal" })).toBeDefined();
    expect(screen.getByText("My Modal")).toBeDefined();
    expect(screen.getByText("Subtitle text")).toBeDefined();
    expect(screen.getByText("Modal content")).toBeDefined();

    const closeBtn = screen.getByLabelText("Close modal");
    fireEvent.click(closeBtn);
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("closes Modal on clicking backdrop", () => {
    const onClose = vi.fn();
    const { container } = render(
      <Modal isOpen={true} onClose={onClose} title="Backdrop Test">
        <p>Inside</p>
      </Modal>
    );

    const backdrop = container.querySelector("[role='presentation']");
    expect(backdrop).not.toBeNull();
    fireEvent.mouseDown(backdrop!);
    expect(onClose).toHaveBeenCalledTimes(1);
  });
});
