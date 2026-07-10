import { describe, expect, it } from "vitest";
import { terminalClipboardAction } from "../lib/terminalClipboard";

function keyEvent(
  key: string,
  overrides: Partial<Pick<KeyboardEvent, "type" | "ctrlKey" | "metaKey" | "shiftKey">> = {}
) {
  return {
    type: "keydown",
    key,
    ctrlKey: false,
    metaKey: false,
    shiftKey: false,
    ...overrides,
  };
}

describe("terminalClipboardAction", () => {
  it("leaves bare Ctrl+C available for SIGINT without a selection", () => {
    expect(terminalClipboardAction(keyEvent("c", { ctrlKey: true }), false)).toBeNull();
  });

  it("copies bare Ctrl+C only when terminal text is selected", () => {
    expect(terminalClipboardAction(keyEvent("c", { ctrlKey: true }), true)).toBe("copy");
  });

  it("supports terminal and macOS copy shortcuts", () => {
    expect(
      terminalClipboardAction(keyEvent("C", { ctrlKey: true, shiftKey: true }), false)
    ).toBe("copy");
    expect(terminalClipboardAction(keyEvent("c", { metaKey: true }), false)).toBe("copy");
  });

  it("supports terminal and macOS paste shortcuts without claiming Ctrl+V", () => {
    expect(
      terminalClipboardAction(keyEvent("v", { ctrlKey: true, shiftKey: true }), false)
    ).toBe("paste");
    expect(terminalClipboardAction(keyEvent("v", { metaKey: true }), false)).toBe("paste");
    expect(terminalClipboardAction(keyEvent("v", { ctrlKey: true }), false)).toBeNull();
  });

  it("ignores keyup events", () => {
    expect(
      terminalClipboardAction(keyEvent("c", { type: "keyup", metaKey: true }), true)
    ).toBeNull();
  });
});
