import { useMemo } from "react";

export interface FilterOptions<T> {
  query?: string;
  getSearchText?: (item: T) => string;
  filters?: Array<(item: T) => boolean>;
  sort?: (a: T, b: T) => number;
}

export function useFilteredList<T>(items: T[], options: FilterOptions<T>): T[] {
  const { query = "", getSearchText, filters = [], sort } = options;

  return useMemo(() => {
    const trimmedQuery = query.trim().toLowerCase();

    const filtered = items.filter((item) => {
      // 1. Check custom filter predicates
      for (const predicate of filters) {
        if (!predicate(item)) return false;
      }

      // 2. Check search query text if specified
      if (trimmedQuery && getSearchText) {
        const text = getSearchText(item).toLowerCase();
        if (!text.includes(trimmedQuery)) return false;
      }

      return true;
    });

    if (sort) {
      return [...filtered].sort(sort);
    }

    return filtered;
  }, [items, query, getSearchText, filters, sort]);
}
