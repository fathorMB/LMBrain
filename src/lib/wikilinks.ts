import type { WikiNode } from "../types";

/**
 * Resolve a [[wikilink]] target against the wiki tree.
 * Returns the full path if found, or null if unresolved.
 */
export function resolveWikilink(
  target: string,
  tree: WikiNode | null
): string | null {
  if (!tree) return null;

  // Normalize: strip .md extension, lowercase
  const normalized = target.toLowerCase().replace(/\.md$/i, "");

  // Search the tree recursively
  const found = findInTree(tree, normalized);
  return found;
}

function findInTree(node: WikiNode, target: string): string | null {
  // Check if this node matches
  const nodeName = node.name.toLowerCase().replace(/\.md$/i, "");
  if (nodeName === target) {
    return node.path;
  }

  // Search children
  for (const child of node.children) {
    const result = findInTree(child, target);
    if (result) return result;
  }

  return null;
}

/**
 * Compute backlinks for a given page path from all wikilinks in the tree.
 */
export function computeBacklinks(
  pagePath: string,
  allWikilinks: Map<string, string[]>
): string[] {
  const backlinks: string[] = [];
  const normalizedTarget = pagePath.toLowerCase();

  for (const [source, links] of allWikilinks) {
    if (links.some((l) => l.toLowerCase() === normalizedTarget)) {
      backlinks.push(source);
    }
  }

  return backlinks;
}
