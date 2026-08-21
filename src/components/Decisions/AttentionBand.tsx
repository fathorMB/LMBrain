import type { Adr } from "../../types";
import type { AttentionItem } from "../../lib/decisionIndex";
import { ATTENTION_ICONS } from "./decisionColors";

export interface AttentionBandProps {
  items: AttentionItem[];
  onOpen: (adr: { title: string; path: string }) => void;
  byId: Map<string, Adr>;
}

export function AttentionBand({ items, onOpen, byId }: AttentionBandProps) {
  return (
    <section
      aria-label="Needs attention"
      style={{
        border: "1px solid var(--border-secondary)",
        borderRadius: "var(--radius-lg)",
        background: "var(--bg-tertiary)",
        padding: "var(--space-3)",
        marginBottom: "var(--space-4)",
      }}
    >
      <h2
        style={{
          fontSize: "var(--text-sm)",
          fontWeight: 700,
          color: "var(--text-secondary)",
          margin: "0 0 var(--space-2)",
        }}
      >
        Needs attention · {items.length}
      </h2>
      <ul style={{ listStyle: "none", margin: 0, padding: 0, display: "grid", gap: "var(--space-1)" }}>
        {items.map((item) => {
          const visual = ATTENTION_ICONS[item.kind];
          const target = byId.get(item.adrId.toUpperCase());
          return (
            <li key={`${item.kind}:${item.adrId}:${item.message}`}>
              <button
                type="button"
                onClick={() => onOpen({ title: target?.title ?? item.adrId, path: item.path })}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: "var(--space-2)",
                  width: "100%",
                  textAlign: "left",
                  background: "none",
                  border: "none",
                  padding: "var(--space-1) 0",
                  font: "inherit",
                  fontSize: "var(--text-sm)",
                  color: "var(--text-secondary)",
                  cursor: "pointer",
                }}
              >
                <i
                  className="material-symbols-outlined"
                  aria-hidden="true"
                  style={{ fontSize: "var(--text-md)", color: visual.color }}
                >
                  {visual.icon}
                </i>
                {item.message}
              </button>
            </li>
          );
        })}
      </ul>
    </section>
  );
}
