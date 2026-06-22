/**
 * Preprocess markdown content to convert [[wikilinks]] into standard
 * markdown links with a `#wikilink:` fragment URL.
 *
 * The link renderer then intercepts `#wikilink:` fragment URLs and
 * handles local navigation. This approach works with rehype-sanitize
 * because fragment-only URLs (starting with `#`) are always allowed
 * by the default schema — no schema extension is needed.
 *
 * [[ADR-001]] → [ADR-001](#wikilink:ADR-001)
 * [[target|display text]] → [display text](#wikilink:target)
 */

// Match [[target]] or [[target|display text]]
const WIKILINK_RE = /\[\[([^\]]+?)(?:\|([^\]]+))?\]\]/g;

/**
 * Transform [[wikilinks]] in a string to standard markdown links.
 * Used as a preprocessing step before markdown parsing.
 */
export function preprocessWikilinksToMarkdown(content: string): string {
  return content.replace(
    WIKILINK_RE,
    (_match: string, target: string, display?: string) => {
      const encoded = encodeURIComponent(target.trim());
      const text = display?.trim() || target.trim();
      return `[${text}](#wikilink:${encoded})`;
    }
  );
}
