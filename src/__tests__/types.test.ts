import { describe, it, expect } from "vitest";

describe("Type definitions", () => {
  it("should have correct TaskStatus values", () => {
    const statuses = [
      "backlog",
      "planned",
      "in-progress",
      "review",
      "done",
      "blocked",
      "cancelled",
    ] as const;
    expect(statuses).toHaveLength(7);
    expect(statuses).toContain("in-progress");
    expect(statuses).toContain("blocked");
  });

  it("should have correct SpecStatus values", () => {
    const statuses = [
      "proposed",
      "ready",
      "in-progress",
      "review",
      "accepted",
      "changes-requested",
      "archived",
    ] as const;
    expect(statuses).toHaveLength(7);
    expect(statuses).toContain("ready");
    expect(statuses).toContain("changes-requested");
  });

  it("should have correct ReviewStatus values", () => {
    const statuses = [
      "pending",
      "accepted",
      "changes-requested",
      "blocked",
      "superseded",
    ] as const;
    expect(statuses).toHaveLength(5);
    expect(statuses).toContain("changes-requested");
  });

  it("should have correct AdrStatus values", () => {
    const statuses = ["proposed", "accepted", "superseded", "deprecated"] as const;
    expect(statuses).toHaveLength(4);
  });

  it("should have correct HandoffStatus values", () => {
    const statuses = ["ready", "consumed", "superseded", "archived"] as const;
    expect(statuses).toHaveLength(4);
    expect(statuses).toContain("ready");
  });
});

describe("Utility types", () => {
  it("should have KitHealth values", () => {
    const health = ["ok", "warn", "none"] as const;
    expect(health).toHaveLength(3);
  });

  it("should have WikiNodeKind values", () => {
    const kinds = [
      "file",
      "folder",
      "knowledge",
      "decisions",
      "specs",
      "tasks",
      "reviews",
      "handoffs",
      "agents",
      "mcp",
    ] as const;
    expect(kinds).toHaveLength(10);
  });
});
