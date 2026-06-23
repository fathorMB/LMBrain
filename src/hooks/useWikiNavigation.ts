import { useWorkspace } from "./useWorkspace";
import { getWikiTree, getWikiPage } from "../lib/commands";
import { resolveWikilink } from "../lib/wikilinks";

/**
 * Returns a handler that opens a `[[wikilink]]` target in the Wiki view.
 * The wiki tree/page are fetched lazily on click so callers stay lightweight.
 * Shared by any view that renders wikilinks outside the Markdown renderer
 * (Pulse, Roadmap, …).
 */
export function useWikiNavigation() {
  const { state, dispatch } = useWorkspace();

  return async (target: string) => {
    try {
      let tree = state.wikiTree;
      if (!tree) {
        tree = await getWikiTree();
        dispatch({ type: "SET_WIKI_TREE", tree });
      }
      const resolved = resolveWikilink(target, tree.root);
      if (resolved) {
        const fullPath = state.currentWorkspace
          ? `${state.currentWorkspace.path}/${resolved}`
          : resolved;
        const page = await getWikiPage(fullPath);
        dispatch({ type: "SET_WIKI_PAGE", page });
      }
      dispatch({ type: "SET_VIEW", view: "wiki" });
    } catch (err) {
      console.error("Failed to open wiki target:", err);
    }
  };
}
