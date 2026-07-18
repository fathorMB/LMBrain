import { describe, expect, it } from "vitest";
import { stripAnsi } from "../lib/ansi";

describe("stripAnsi", () => {
  it("removes ANSI color sequences while preserving text", () => {
    expect(stripAnsi("\u001b[31mError\u001b[0m: details")).toBe("Error: details");
  });

  it("leaves plain transcript text unchanged", () => {
    expect(stripAnsi("plain output\nnext line")).toBe("plain output\nnext line");
  });
});
