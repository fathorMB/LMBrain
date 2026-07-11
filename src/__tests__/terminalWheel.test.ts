import { describe, expect, it } from "vitest";
import { shouldDelegateTerminalWheel, terminalWheelRows } from "../lib/terminalWheel";

describe("terminal wheel policy", () => {
  it("delegates alternate-buffer wheel events to xterm for full-screen TUIs", () => {
    expect(
      shouldDelegateTerminalWheel("alternate", { ctrlKey: false, metaKey: false })
    ).toBe(true);
  });

  it("keeps normal-buffer wheel events in local scrollback", () => {
    expect(
      shouldDelegateTerminalWheel("normal", { ctrlKey: false, metaKey: false })
    ).toBe(false);
  });

  it("does not claim modifier-assisted wheel gestures", () => {
    expect(shouldDelegateTerminalWheel("normal", { ctrlKey: true, metaKey: false })).toBe(true);
    expect(shouldDelegateTerminalWheel("normal", { ctrlKey: false, metaKey: true })).toBe(true);
  });

  it("converts wheel delta to at least one scrollback row", () => {
    expect(terminalWheelRows(0)).toBe(1);
    expect(terminalWheelRows(72)).toBe(2);
    expect(terminalWheelRows(-73)).toBe(3);
  });
});
