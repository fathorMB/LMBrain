// ─── Shared unread/read-state model for sidebar badges ────────────
//
// One policy for every workspace page that owns a collection of items:
//
//   * an item contributes a signature built from its lifecycle state
//     (`status`) and its last modification (`updated`);
//   * an item is UNREAD when its id is unknown for that page, or when its
//     signature differs from the one recorded the last time the page was
//     displayed;
//   * opening a page marks everything currently displayed on it as read;
//   * read state is persisted per workspace, so it survives refreshes,
//     watcher updates, and application restarts.
//
// Wiki, Design and Repository are intentionally absent: they render
// documents and Git state rather than governed item collections.
// Pulse, Insights, Roadmap and Sessions are also absent: they are derived
// or runtime surfaces whose contents are already counted on the pages that
// own the source artifacts.

export const UNREAD_PAGES = [
  "taskboard",
  "reviews",
  "debts",
  "feedback",
  "decisions",
  "agents",
  "mcp",
  "skills",
] as const;

export type UnreadPage = (typeof UNREAD_PAGES)[number];

export interface UnreadItem {
  /** Stable identity of the item, namespaced by collection. */
  id: string;
  /** Changes whenever the item should count as unread again. */
  signature: string;
}

export type PageItems = Record<UnreadPage, UnreadItem[]>;

/** Per page: item id → signature recorded when the page was last displayed. */
export type ReadState = Partial<Record<UnreadPage, Record<string, string>>>;

interface ArtifactLike {
  id?: string | null;
  status?: string | null;
  updated?: string | null;
  path?: string | null;
  malformed?: boolean | null;
}

interface FeedbackNoteLike {
  id?: string | null;
  timestamp?: string | null;
  severity?: string | null;
}

export interface UnreadSource {
  specs?: ArtifactLike[] | null;
  reviews?: ArtifactLike[] | null;
  debts?: ArtifactLike[] | null;
  adrs?: ArtifactLike[] | null;
  agents?: ArtifactLike[] | null;
  agentProposals?: ArtifactLike[] | null;
  mcpRecords?: ArtifactLike[] | null;
  mcpProposals?: ArtifactLike[] | null;
  skills?: ArtifactLike[] | null;
  kitFeedbackNotes?: FeedbackNoteLike[] | null;
}

const STORAGE_PREFIX = "lmbrain.readState.v1:";

export function isUnreadPage(view: string): view is UnreadPage {
  return (UNREAD_PAGES as readonly string[]).includes(view);
}

function emptyPageItems(): PageItems {
  return {
    taskboard: [],
    reviews: [],
    debts: [],
    feedback: [],
    decisions: [],
    agents: [],
    mcp: [],
    skills: [],
  };
}

/**
 * Governed artifacts are identified by their id, namespaced by collection so
 * that two collections rendered on the same page cannot collide. Malformed or
 * partially parsed records still get a deterministic identity so they remain
 * visible instead of silently disappearing from the count.
 */
export function toUnreadItems(namespace: string, items: ArtifactLike[] | null | undefined): UnreadItem[] {
  if (!Array.isArray(items)) return [];
  return items.map((item, index) => {
    const rawId = typeof item?.id === "string" ? item.id.trim() : "";
    const rawPath = typeof item?.path === "string" ? item.path.trim() : "";
    const identity = rawId || rawPath || `#${index}`;
    const status = typeof item?.status === "string" ? item.status : "";
    const updated = typeof item?.updated === "string" ? item.updated : "";
    const malformed = item?.malformed ? "malformed" : "";
    return {
      id: `${namespace}:${identity}`,
      signature: `${status}|${updated}|${malformed}`,
    };
  });
}

function toFeedbackItems(notes: FeedbackNoteLike[] | null | undefined): UnreadItem[] {
  if (!Array.isArray(notes)) return [];
  return notes.map((note, index) => {
    const rawId = typeof note?.id === "string" ? note.id.trim() : "";
    const timestamp = typeof note?.timestamp === "string" ? note.timestamp : "";
    const severity = typeof note?.severity === "string" ? note.severity : "";
    return {
      id: `feedback:${rawId || `#${index}`}`,
      signature: `${severity}|${timestamp}`,
    };
  });
}

/** Build the per-page item collections the badge counts are derived from. */
export function collectPageItems(source: UnreadSource | null | undefined): PageItems {
  const items = emptyPageItems();
  if (!source) return items;
  items.taskboard = toUnreadItems("spec", source.specs);
  items.reviews = toUnreadItems("review", source.reviews);
  items.debts = toUnreadItems("debt", source.debts);
  items.feedback = toFeedbackItems(source.kitFeedbackNotes);
  items.decisions = toUnreadItems("adr", source.adrs);
  items.agents = [
    ...toUnreadItems("agent", source.agents),
    ...toUnreadItems("agent-proposal", source.agentProposals),
  ];
  items.mcp = [
    ...toUnreadItems("mcp", source.mcpRecords),
    ...toUnreadItems("mcp-proposal", source.mcpProposals),
  ];
  items.skills = toUnreadItems("skill", source.skills);
  return items;
}

export function countUnread(items: UnreadItem[], read: Record<string, string> | undefined): number {
  if (!Array.isArray(items) || items.length === 0) return 0;
  if (!read) return items.length;
  let count = 0;
  for (const item of items) {
    if (read[item.id] !== item.signature) count += 1;
  }
  return count;
}

export function countAllUnread(pageItems: PageItems, readState: ReadState): Record<UnreadPage, number> {
  const counts = {} as Record<UnreadPage, number>;
  for (const page of UNREAD_PAGES) {
    counts[page] = countUnread(pageItems[page] ?? [], readState[page]);
  }
  return counts;
}

function sameEntries(a: Record<string, string> | undefined, b: Record<string, string>): boolean {
  if (!a) return false;
  const aKeys = Object.keys(a);
  if (aKeys.length !== Object.keys(b).length) return false;
  return aKeys.every((key) => a[key] === b[key]);
}

/**
 * Record the given items as read for a page. Entries for items that no longer
 * exist are pruned so the persisted state cannot grow without bound. Returns
 * the original state object when nothing changed, so callers can use identity
 * to skip re-renders and writes.
 */
export function markItemsRead(readState: ReadState, page: UnreadPage, items: UnreadItem[]): ReadState {
  const next: Record<string, string> = {};
  const previous = readState[page];
  const present = new Set<string>();
  for (const item of items ?? []) {
    next[item.id] = item.signature;
    present.add(item.id);
  }
  // Preserve nothing for vanished items: identity is derived from the artifact,
  // so a reappearing artifact with the same signature stays read only while it
  // is present in the workspace.
  if (sameEntries(previous, next)) return readState;
  return { ...readState, [page]: next };
}

/** Mark a single item as read without touching the rest of the page. */
export function markItemRead(readState: ReadState, page: UnreadPage, item: UnreadItem | null): ReadState {
  if (!item) return readState;
  const previous = readState[page] ?? {};
  if (previous[item.id] === item.signature) return readState;
  return { ...readState, [page]: { ...previous, [item.id]: item.signature } };
}

/**
 * Baseline for a workspace opened for the first time: everything already in the
 * project counts as seen, so badges report what happened since, instead of
 * showing the whole backlog on first launch.
 */
export function seedAllRead(pageItems: PageItems): ReadState {
  const state: ReadState = {};
  for (const page of UNREAD_PAGES) {
    const entries: Record<string, string> = {};
    for (const item of pageItems[page] ?? []) {
      entries[item.id] = item.signature;
    }
    state[page] = entries;
  }
  return state;
}

export function readStateStorageKey(workspacePath: string): string {
  return `${STORAGE_PREFIX}${workspacePath}`;
}

function isValidReadState(value: unknown): value is ReadState {
  if (!value || typeof value !== "object" || Array.isArray(value)) return false;
  return Object.entries(value as Record<string, unknown>).every(([page, entries]) => {
    if (!isUnreadPage(page)) return true;
    if (!entries || typeof entries !== "object" || Array.isArray(entries)) return false;
    return Object.values(entries as Record<string, unknown>).every((signature) => typeof signature === "string");
  });
}

/** Returns null when no usable state is stored, so callers can seed a baseline. */
export function loadReadState(workspacePath: string): ReadState | null {
  try {
    const raw = globalThis.localStorage?.getItem(readStateStorageKey(workspacePath));
    if (!raw) return null;
    const parsed: unknown = JSON.parse(raw);
    if (!isValidReadState(parsed)) return null;
    const state: ReadState = {};
    for (const page of UNREAD_PAGES) {
      const entries = (parsed as ReadState)[page];
      if (entries) state[page] = entries;
    }
    return state;
  } catch {
    return null;
  }
}

export function saveReadState(workspacePath: string, readState: ReadState): void {
  try {
    globalThis.localStorage?.setItem(readStateStorageKey(workspacePath), JSON.stringify(readState));
  } catch {
    // Read state is a convenience: never break navigation over storage limits
    // or a locked-down storage backend.
  }
}

/** Accessible name for a navigation entry, so the count is never colour-only. */
export function navItemAccessibleName(label: string, unread: number): string {
  if (unread <= 0) return label;
  return `${label}, ${unread} unread item${unread === 1 ? "" : "s"}`;
}
