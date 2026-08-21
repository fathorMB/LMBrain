import type { AdrStatus } from "../../types";

export const STATUS_COLORS: Record<AdrStatus, { color: string; bg: string }> = {
  accepted: { color: "#46b07d", bg: "rgba(70,176,125,.12)" },
  proposed: { color: "#8a8d99", bg: "rgba(139,141,152,.12)" },
  superseded: { color: "#e0a23a", bg: "rgba(224,162,58,.12)" },
  deprecated: { color: "#c07ad8", bg: "rgba(192,122,216,.12)" },
  rejected: { color: "#e0584a", bg: "rgba(224,88,74,.12)" },
};

export const ATTENTION_ICONS = {
  integrity: { icon: "link_off", color: "#e0a23a" },
  malformed: { icon: "warning", color: "#e0584a" },
  pending: { icon: "pending", color: "var(--text-tertiary)" },
} as const;
