export type TerminalBufferType = "normal" | "alternate";

export function shouldDelegateTerminalWheel(
  bufferType: TerminalBufferType,
  event: Pick<WheelEvent, "ctrlKey" | "metaKey">
) {
  return bufferType === "alternate" || event.ctrlKey || event.metaKey;
}

export function terminalWheelRows(deltaY: number) {
  return Math.max(1, Math.ceil(Math.abs(deltaY) / 36));
}

export type TerminalPageAction =
  | { kind: "local"; pages: -1 | 1 }
  | { kind: "input"; data: string };

export function terminalPageAction(
  bufferType: TerminalBufferType,
  direction: -1 | 1,
  host?: "opencode"
): TerminalPageAction {
  return bufferType === "normal"
    ? { kind: "local", pages: direction }
    : {
        kind: "input",
        // OpenCode documents Ctrl+Alt+B/F as alternate page bindings. Under
        // ConPTY these arrive reliably as ESC + the corresponding control byte,
        // unlike PageUp CSI sequences in its embedded OpenTUI path.
        data:
          host === "opencode"
            ? direction < 0
              ? "\u001b\u0002"
              : "\u001b\u0006"
            : direction < 0
              ? "\u001b[5~"
              : "\u001b[6~",
      };
}

export function opencodeWheelInput(direction: -1 | 1, rows: number) {
  // OpenCode default bindings: Ctrl+Alt+Y/E scroll messages one line. Cap a
  // single wheel frame to avoid flooding the PTY on high-resolution devices.
  const sequence = direction < 0 ? "\u001b\u0019" : "\u001b\u0005";
  return sequence.repeat(Math.max(1, Math.min(rows, 6)));
}

export function tuiWheelInput(
  host: "claude" | "pi" | "opencode",
  direction: -1 | 1,
  rows: number
) {
  return host === "opencode"
    ? opencodeWheelInput(direction, rows)
    : direction < 0
      ? "\u001b[5~"
      : "\u001b[6~";
}

export const OPENCODE_SCROLL_TO_BOTTOM = "\u001b\u0007"; // Ctrl+Alt+G
