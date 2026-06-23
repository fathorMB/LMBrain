import { Fragment } from "react";

/**
 * Lightweight inline renderer for short strings (pulse breadcrumbs, focus
 * sentences, milestone titles) that may contain `**bold**` and `[[wikilink]]`
 * markup. Unlike the full {@link MarkdownRenderer}, it does not wrap content in
 * block elements, so it can be dropped inside an existing inline context.
 *
 * Supported syntax:
 *   - `**bold**`              → <strong>
 *   - `[[Target]]`            → clickable wikilink (label = target)
 *   - `[[Target|Label]]`      → clickable wikilink with a custom label
 */
const TOKEN = /(\*\*[^*]+\*\*|\[\[[^\]]+\]\])/g;

export function InlineRichText({
  text,
  onWikilinkClick,
}: {
  text: string;
  onWikilinkClick?: (target: string) => void;
}) {
  const parts = text.split(TOKEN).filter((p) => p !== "");

  return (
    <>
      {parts.map((part, i) => {
        if (part.startsWith("**") && part.endsWith("**")) {
          return (
            <strong key={i} style={{ fontWeight: 700, color: "var(--text-primary)" }}>
              {part.slice(2, -2)}
            </strong>
          );
        }

        if (part.startsWith("[[") && part.endsWith("]]")) {
          const inner = part.slice(2, -2);
          const [rawTarget, rawLabel] = inner.split("|");
          const target = rawTarget.trim();
          const label = (rawLabel ?? rawTarget).trim();

          return (
            <span
              key={i}
              role="button"
              tabIndex={0}
              aria-label={`Navigate to ${target}`}
              title={`Navigate to ${target}`}
              onClick={() => onWikilinkClick?.(target)}
              onKeyDown={(e) => {
                if (e.key === "Enter" || e.key === " ") {
                  e.preventDefault();
                  onWikilinkClick?.(target);
                }
              }}
              style={{
                color: "var(--accent-light)",
                borderBottom: "1px dashed rgba(165,150,245,.5)",
                cursor: "pointer",
                fontFamily: "var(--font-mono)",
                fontSize: "0.95em",
              }}
            >
              {label}
            </span>
          );
        }

        return <Fragment key={i}>{part}</Fragment>;
      })}
    </>
  );
}
