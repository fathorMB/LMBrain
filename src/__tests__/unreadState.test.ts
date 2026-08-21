import { beforeEach, describe, expect, it } from "vitest";
import {
  UNREAD_PAGES,
  collectPageItems,
  countAllUnread,
  countUnread,
  isUnreadPage,
  loadReadState,
  markItemRead,
  markItemsRead,
  navItemAccessibleName,
  readStateStorageKey,
  saveReadState,
  seedAllRead,
  toUnreadItems,
} from "../lib/unreadState";

function artifact(id: string, status = "open", updated = "2026-07-01") {
  return { id, status, updated, path: `docs/${id}.md` };
}

describe("unread page eligibility", () => {
  it("covers the workspace item collections and excludes documentary pages", () => {
    expect([...UNREAD_PAGES]).toEqual([
      "taskboard",
      "reviews",
      "operations",
      "debts",
      "feedback",
      "decisions",
      "agents",
      "mcp",
      "skills",
    ]);
    for (const excluded of ["wiki", "design", "repository", "pulse", "insights", "roadmap", "sessions", "settings"]) {
      expect(isUnreadPage(excluded)).toBe(false);
    }
  });
});

describe("item identity and signatures", () => {
  it("namespaces identity per collection so pages sharing a view cannot collide", () => {
    const records = toUnreadItems("mcp", [artifact("SHARED-1")]);
    const proposals = toUnreadItems("mcp-proposal", [artifact("SHARED-1")]);
    expect(records[0].id).not.toBe(proposals[0].id);
  });

  it("changes the signature when status or updated changes", () => {
    const [base] = toUnreadItems("spec", [artifact("SPEC-1", "ready", "2026-07-01")]);
    const [restatused] = toUnreadItems("spec", [artifact("SPEC-1", "working", "2026-07-01")]);
    const [retouched] = toUnreadItems("spec", [artifact("SPEC-1", "ready", "2026-07-02")]);
    expect(base.id).toBe(restatused.id);
    expect(base.signature).not.toBe(restatused.signature);
    expect(base.signature).not.toBe(retouched.signature);
  });

  it("keeps malformed and id-less records countable instead of dropping them", () => {
    const items = toUnreadItems("debt", [
      { id: "", path: "docs/debts/f-1.md", status: "open", updated: "2026-07-01", malformed: true },
      { status: null, updated: undefined },
    ]);
    expect(items).toHaveLength(2);
    expect(items[0].id).toBe("debt:docs/debts/f-1.md");
    expect(items[0].signature).toContain("malformed");
    expect(items[1].id).toBe("debt:#1");
  });

  it("tolerates missing or non-array collections", () => {
    const items = collectPageItems({ specs: null, reviews: undefined });
    for (const page of UNREAD_PAGES) {
      expect(items[page]).toEqual([]);
    }
    expect(collectPageItems(null).debts).toEqual([]);
  });

  it("identifies kit feedback notes by note id and timestamp", () => {
    const items = collectPageItems({
      kitFeedbackNotes: [{ id: "NOTE-1", timestamp: "2026-07-01T10:00:00Z", severity: "high" }],
    });
    expect(items.feedback).toEqual([
      { id: "feedback:NOTE-1", signature: "high|2026-07-01T10:00:00Z" },
    ]);
  });
});

describe("counting", () => {
  it("counts every item when the page was never displayed", () => {
    const items = toUnreadItems("review", [artifact("R-1"), artifact("R-2")]);
    expect(countUnread(items, undefined)).toBe(2);
  });

  it("counts new and changed items only", () => {
    const before = toUnreadItems("review", [artifact("R-1", "pending")]);
    const read = markItemsRead({}, "reviews", before);
    const after = toUnreadItems("review", [
      artifact("R-1", "accepted"),
      artifact("R-2", "pending"),
    ]);
    expect(countUnread(after, read.reviews)).toBe(2);

    const unchanged = toUnreadItems("review", [artifact("R-1", "pending")]);
    expect(countUnread(unchanged, read.reviews)).toBe(0);
  });

  it("reports zero for pages with no items", () => {
    const counts = countAllUnread(collectPageItems({}), {});
    for (const page of UNREAD_PAGES) {
      expect(counts[page]).toBe(0);
    }
  });
});

describe("read-state transitions", () => {
  it("marks a whole page read without touching other pages", () => {
    const items = collectPageItems({
      reviews: [artifact("R-1")],
      debts: [artifact("F-1")],
    });
    const read = markItemsRead({}, "reviews", items.reviews);
    const counts = countAllUnread(items, read);
    expect(counts.reviews).toBe(0);
    expect(counts.debts).toBe(1);
  });

  it("returns the same state object when nothing changed", () => {
    const items = toUnreadItems("skill", [artifact("S-1")]);
    const read = markItemsRead({}, "skills", items);
    expect(markItemsRead(read, "skills", items)).toBe(read);
  });

  it("prunes entries for items that no longer exist", () => {
    const read = markItemsRead({}, "decisions", toUnreadItems("adr", [artifact("ADR-1"), artifact("ADR-2")]));
    const pruned = markItemsRead(read, "decisions", toUnreadItems("adr", [artifact("ADR-1")]));
    expect(Object.keys(pruned.decisions ?? {})).toEqual(["adr:ADR-1"]);
  });

  it("marks a single item read when an individual spec is opened", () => {
    const items = collectPageItems({ specs: [artifact("SPEC-1"), artifact("SPEC-2")] });
    const read = markItemRead({}, "taskboard", items.taskboard[0]);
    expect(countUnread(items.taskboard, read.taskboard)).toBe(1);
    expect(markItemRead(read, "taskboard", items.taskboard[0])).toBe(read);
    expect(markItemRead(read, "taskboard", null)).toBe(read);
  });

  it("seeds an existing project as fully read", () => {
    const items = collectPageItems({
      specs: [artifact("SPEC-1")],
      debts: [artifact("F-1")],
      kitFeedbackNotes: [{ id: "NOTE-1", timestamp: "2026-07-01", severity: "low" }],
    });
    const counts = countAllUnread(items, seedAllRead(items));
    for (const page of UNREAD_PAGES) {
      expect(counts[page]).toBe(0);
    }
  });
});

describe("persistence", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it("round-trips read state per workspace path", () => {
    const read = markItemsRead({}, "debts", toUnreadItems("debt", [artifact("F-1")]));
    saveReadState("E:/projects/alpha", read);
    expect(loadReadState("E:/projects/alpha")).toEqual(read);
    expect(loadReadState("E:/projects/beta")).toBeNull();
  });

  it("reports missing state as null so a baseline can be seeded", () => {
    expect(loadReadState("E:/projects/alpha")).toBeNull();
  });

  it("discards malformed stored state instead of breaking navigation", () => {
    localStorage.setItem(readStateStorageKey("E:/projects/alpha"), "{not json");
    expect(loadReadState("E:/projects/alpha")).toBeNull();
    localStorage.setItem(readStateStorageKey("E:/projects/alpha"), JSON.stringify({ debts: 42 }));
    expect(loadReadState("E:/projects/alpha")).toBeNull();
  });
});

describe("accessible naming", () => {
  it("includes the count and its meaning, and stays plain at zero", () => {
    expect(navItemAccessibleName("Reviews", 0)).toBe("Reviews");
    expect(navItemAccessibleName("Reviews", 1)).toBe("Reviews, 1 unread item");
    expect(navItemAccessibleName("Reviews", 4)).toBe("Reviews, 4 unread items");
  });
});
