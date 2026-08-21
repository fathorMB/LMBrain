import { InlineRichText } from "../../lib/inlineRichText";
import { useWikiNavigation } from "../../hooks/useWikiNavigation";
import type { MilestoneDetail } from "../../types";

export interface MilestoneSidebarProps {
  milestones: MilestoneDetail[];
  selectedId: string | null;
  onSelectMilestone: (id: string) => void;
}

const statusColors: Record<string, { color: string; bg: string }> = {
  active: { color: "#5b8def", bg: "rgba(91,141,239,0.13)" },
  planned: { color: "#8a8d99", bg: "rgba(138,141,153,0.13)" },
  completed: { color: "#46b07d", bg: "rgba(70,176,125,0.13)" },
};

export function MilestoneSidebar({
  milestones,
  selectedId,
  onSelectMilestone,
}: MilestoneSidebarProps) {
  const navigateToWiki = useWikiNavigation();

  return (
    <div style={{ flex: "0 0 280px", display: "flex", flexDirection: "column", gap: 8 }}>
      {milestones.map((m) => {
        const sc = statusColors[m.status] ?? { color: "#8a8d99", bg: "rgba(138,141,153,0.13)" };
        const isSelected = m.id === selectedId;
        return (
          <button
            type="button"
            key={m.id}
            onClick={() => onSelectMilestone(m.id)}
            aria-pressed={isSelected}
            style={{
              width: "100%",
              display: "flex",
              alignItems: "center",
              gap: 10,
              padding: "10px 12px",
              borderRadius: 10,
              background: isSelected ? "rgba(124,108,246,0.08)" : "transparent",
              border: `1px solid ${isSelected ? "rgba(124,108,246,0.25)" : "transparent"}`,
              cursor: "pointer",
              textAlign: "left",
              fontFamily: "inherit",
            }}
          >
            <div
              style={{
                width: 8,
                height: 8,
                borderRadius: "50%",
                background: sc.color,
                flex: "none",
              }}
            />
            <div style={{ flex: 1, minWidth: 0 }}>
              <div
                style={{
                  fontSize: "var(--text-sm)",
                  fontWeight: 650,
                  color: isSelected ? "var(--text-primary)" : "var(--text-secondary)",
                  whiteSpace: "nowrap",
                  overflow: "hidden",
                  textOverflow: "ellipsis",
                }}
              >
                <InlineRichText text={m.title} onWikilinkClick={navigateToWiki} />
              </div>
              <div
                style={{
                  fontSize: "var(--text-xs)",
                  color: "var(--text-tertiary)",
                  display: "flex",
                  gap: 8,
                  marginTop: 2,
                }}
              >
                <span>{m.spec_count} specs</span>
                {m.progress_pct > 0 && <span>{Math.round(m.progress_pct)}%</span>}
              </div>
            </div>
            <span
              style={{
                fontSize: "var(--text-2xs)",
                fontWeight: 700,
                color: sc.color,
                background: sc.bg,
                borderRadius: 4,
                padding: "2px 6px",
                flex: "none",
              }}
            >
              {m.status.toUpperCase()}
            </span>
          </button>
        );
      })}
    </div>
  );
}
