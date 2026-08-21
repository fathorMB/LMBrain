import { describe, it, expect } from "vitest";
import { renderHook } from "@testing-library/react";
import { useFilteredList } from "../hooks/useFilteredList";

interface TestItem {
  id: string;
  name: string;
  category: string;
  rank: number;
}

const ITEMS: TestItem[] = [
  { id: "1", name: "Alpha feature", category: "core", rank: 3 },
  { id: "2", name: "Beta improvements", category: "ui", rank: 1 },
  { id: "3", name: "Gamma core service", category: "core", rank: 2 },
];

describe("useFilteredList hook", () => {
  it("filters items by search query", () => {
    const { result } = renderHook(() =>
      useFilteredList(ITEMS, {
        query: "core",
        getSearchText: (item) => `${item.id} ${item.name} ${item.category}`,
      })
    );

    expect(result.current).toHaveLength(2);
    expect(result.current.map((i) => i.id)).toEqual(["1", "3"]);
  });

  it("applies predicate filters", () => {
    const { result } = renderHook(() =>
      useFilteredList(ITEMS, {
        filters: [(item) => item.category === "ui"],
      })
    );

    expect(result.current).toHaveLength(1);
    expect(result.current[0].name).toBe("Beta improvements");
  });

  it("sorts filtered results according to sort comparator", () => {
    const { result } = renderHook(() =>
      useFilteredList(ITEMS, {
        sort: (a, b) => a.rank - b.rank,
      })
    );

    expect(result.current.map((i) => i.id)).toEqual(["2", "3", "1"]);
  });
});
