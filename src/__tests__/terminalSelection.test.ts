import { describe, expect, it } from "vitest";
import {
  restoreMouseTracking,
  SUSPEND_MOUSE_TRACKING,
  visibleViewportText,
} from "../lib/terminalSelection";
import type { ViewportReader } from "../lib/terminalSelection";

describe("select text mode sequences", () => {
  it("suspends every tracking mode without touching report encodings", () => {
    expect(SUSPEND_MOUSE_TRACKING).toBe(
      "\u001b[?1003l\u001b[?1002l\u001b[?1000l\u001b[?9l"
    );
    // SGR (1006), UTF (1005), and urxvt (1015) encodings must stay untouched.
    expect(SUSPEND_MOUSE_TRACKING).not.toContain("1006");
    expect(SUSPEND_MOUSE_TRACKING).not.toContain("1005");
    expect(SUSPEND_MOUSE_TRACKING).not.toContain("1015");
  });

  it("restores exactly the snapshotted tracking mode", () => {
    expect(restoreMouseTracking("x10")).toBe("\u001b[?9h");
    expect(restoreMouseTracking("vt200")).toBe("\u001b[?1000h");
    expect(restoreMouseTracking("drag")).toBe("\u001b[?1002h");
    expect(restoreMouseTracking("any")).toBe("\u001b[?1003h");
    expect(restoreMouseTracking("none")).toBe("");
  });

  it("suspend then restore is a lossless round trip for every tracked mode", () => {
    for (const mode of ["x10", "vt200", "drag", "any"] as const) {
      const restore = restoreMouseTracking(mode);
      const code = restore.slice(3, -1);
      expect(SUSPEND_MOUSE_TRACKING).toContain(`[?${code}l`);
    }
  });
});

function fakeViewport(rows: number, viewportY: number, lines: string[]): ViewportReader {
  return {
    rows,
    buffer: {
      active: {
        viewportY,
        getLine: (y: number) =>
          y < lines.length ? { translateToString: () => lines[y] } : undefined,
      },
    },
  };
}

describe("visible viewport copy", () => {
  it("serializes only the visible rows at the current viewport offset", () => {
    const term = fakeViewport(3, 2, ["s0", "s1", "v0", "v1", "v2", "below"]);
    expect(visibleViewportText(term)).toBe("v0\nv1\nv2");
  });

  it("drops trailing blank rows but keeps interior ones", () => {
    const term = fakeViewport(5, 0, ["first", "", "third", "", ""]);
    expect(visibleViewportText(term)).toBe("first\n\nthird");
  });

  it("returns an empty string for an entirely blank viewport", () => {
    const term = fakeViewport(4, 0, ["", "", "", ""]);
    expect(visibleViewportText(term)).toBe("");
  });
});
