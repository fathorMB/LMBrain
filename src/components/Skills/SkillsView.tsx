import { useMemo, useState } from "react";
import { useWorkspace } from "../../hooks/useWorkspace";
import type { Skill, SkillStatus } from "../../types";

const STATUSES: Array<"all" | SkillStatus> = ["all", "active", "proposed", "retired"];

const STATUS_COLORS: Record<SkillStatus, { color: string; bg: string }> = {
  active: { color: "#46b07d", bg: "rgba(70,176,125,.12)" },
  proposed: { color: "#e0a23a", bg: "rgba(224,162,58,.12)" },
  retired: { color: "#6c6671", bg: "rgba(108,102,113,.12)" },
};

const RISK_COLORS: Record<string, { color: string; bg: string }> = {
  low: { color: "#46b07d", bg: "rgba(70,176,125,.1)" },
  medium: { color: "#e0a23a", bg: "rgba(224,162,58,.1)" },
  high: { color: "#e0584a", bg: "rgba(224,88,74,.12)" },
};

export function SkillsView() {
  const { state } = useWorkspace();
  const [statusFilter, setStatusFilter] = useState<"all" | SkillStatus>("all");
  const [kindFilter, setKindFilter] = useState("all");

  const kinds = useMemo(() => {
    const values = state.skills
      .map((skill) => skill.kind?.trim())
      .filter((kind): kind is string => Boolean(kind));
    return ["all", ...Array.from(new Set(values)).sort()];
  }, [state.skills]);

  const filteredSkills = useMemo(() => {
    return state.skills.filter((skill) => {
      const statusMatches = statusFilter === "all" || skill.status === statusFilter;
      const kindMatches = kindFilter === "all" || skill.kind === kindFilter;
      return statusMatches && kindMatches;
    });
  }, [kindFilter, state.skills, statusFilter]);

  const skillDiagnostics = state.diagnostics.filter((diagnostic) =>
    diagnostic.path?.startsWith("skills/") ||
    diagnostic.message.toLowerCase().includes("skill")
  );

  return (
    <div style={{ overflowY: "auto", height: "100%" }}>
      <div style={{ maxWidth: 980, margin: "0 auto", padding: "24px 36px 70px" }}>
        <div style={{ display: "flex", alignItems: "flex-start", justifyContent: "space-between", gap: 18, marginBottom: 22 }}>
          <div>
            <h1 style={{ fontSize: 24, fontWeight: 800, letterSpacing: "-.025em", margin: "0 0 5px" }}>
              Skills
            </h1>
            <p style={{ fontSize: 13.5, color: "var(--text-tertiary)", margin: 0 }}>
              Project-scoped procedures available to manually started agents.
            </p>
          </div>
          <div style={{ display: "flex", gap: 8, flexWrap: "wrap", justifyContent: "flex-end" }}>
            <SegmentedFilter
              options={STATUSES}
              value={statusFilter}
              onChange={(value) => setStatusFilter(value as "all" | SkillStatus)}
            />
            <select
              value={kindFilter}
              onChange={(event) => setKindFilter(event.target.value)}
              style={{
                height: 30,
                background: "var(--bg-tertiary)",
                color: "var(--text-secondary)",
                border: "1px solid var(--border-secondary)",
                borderRadius: 7,
                padding: "0 9px",
                fontSize: 12,
              }}
            >
              {kinds.map((kind) => (
                <option key={kind} value={kind}>
                  {kind === "all" ? "All kinds" : kind}
                </option>
              ))}
            </select>
          </div>
        </div>

        {skillDiagnostics.length > 0 && (
          <div
            style={{
              border: "1px solid rgba(224,162,58,.28)",
              background: "rgba(224,162,58,.08)",
              borderRadius: 8,
              padding: "11px 13px",
              marginBottom: 18,
            }}
          >
            <div style={{ fontSize: 11, fontWeight: 700, color: "#e0a23a", textTransform: "uppercase", letterSpacing: ".08em", marginBottom: 7 }}>
              Skill Diagnostics
            </div>
            <div style={{ display: "flex", flexDirection: "column", gap: 5 }}>
              {skillDiagnostics.slice(0, 4).map((diagnostic, index) => (
                <div key={`${diagnostic.path ?? "skill"}-${index}`} style={{ fontSize: 12.5, color: "var(--text-secondary)" }}>
                  {diagnostic.message}
                </div>
              ))}
              {skillDiagnostics.length > 4 && (
                <div style={{ fontSize: 12, color: "var(--text-tertiary)" }}>
                  {skillDiagnostics.length - 4} more warning(s)
                </div>
              )}
            </div>
          </div>
        )}

        {state.skills.length === 0 ? (
          <EmptySkills />
        ) : filteredSkills.length === 0 ? (
          <div style={{ textAlign: "center", padding: 38, color: "var(--text-tertiary)" }}>
            No skills match the current filters.
          </div>
        ) : (
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(280px, 1fr))", gap: 10 }}>
            {filteredSkills.map((skill) => (
              <SkillCard key={skill.id} skill={skill} />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function SegmentedFilter({
  options,
  value,
  onChange,
}: {
  options: string[];
  value: string;
  onChange: (value: string) => void;
}) {
  return (
    <div
      style={{
        display: "flex",
        background: "var(--bg-tertiary)",
        border: "1px solid var(--border-secondary)",
        borderRadius: 7,
        padding: 2,
        height: 30,
      }}
    >
      {options.map((option) => {
        const active = option === value;
        return (
          <button
            key={option}
            type="button"
            onClick={() => onChange(option)}
            style={{
              border: 0,
              borderRadius: 5,
              background: active ? "var(--bg-active)" : "transparent",
              color: active ? "var(--text-primary)" : "var(--text-tertiary)",
              fontSize: 11.5,
              fontWeight: active ? 700 : 500,
              padding: "0 8px",
              textTransform: "capitalize",
              cursor: "pointer",
            }}
          >
            {option}
          </button>
        );
      })}
    </div>
  );
}

function SkillCard({ skill }: { skill: Skill }) {
  const { openDetailArtifact } = useWorkspace();
  const statusColor = STATUS_COLORS[skill.status];
  const risk = skill.risk ?? "unspecified";
  const riskColor = RISK_COLORS[risk] ?? { color: "#9a949f", bg: "rgba(255,255,255,.05)" };
  const appliesTo = skill.applies_to.length > 0 ? skill.applies_to.join(", ") : "discoverable";
  const commandPreview = skill.commands.slice(0, 2).join(" - ");

  return (
    <button
      type="button"
      onClick={() => openDetailArtifact({ title: skill.title, path: skill.path })}
      style={{
        textAlign: "left",
        background: "var(--bg-tertiary)",
        border: "1px solid var(--border-secondary)",
        borderRadius: 8,
        padding: "14px 15px",
        color: "var(--text-primary)",
        cursor: "pointer",
        minHeight: 156,
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 9 }}>
        <span style={{ fontFamily: "var(--font-mono)", fontSize: 11.5, color: "#bcaef6" }}>
          {skill.id}
        </span>
        <span style={{ fontSize: 10.5, fontWeight: 800, color: statusColor.color, background: statusColor.bg, borderRadius: 5, padding: "2px 7px" }}>
          {skill.status.toUpperCase()}
        </span>
        <span style={{ fontSize: 10.5, fontWeight: 800, color: riskColor.color, background: riskColor.bg, borderRadius: 5, padding: "2px 7px" }}>
          {risk.toUpperCase()}
        </span>
      </div>
      <div style={{ fontSize: 14, fontWeight: 700, lineHeight: 1.25, marginBottom: 8 }}>
        {skill.title}
      </div>
      <div style={{ fontSize: 12, color: "var(--text-tertiary)", marginBottom: 10 }}>
        {skill.kind || "procedure"} - {appliesTo}
      </div>
      {skill.domains.length > 0 && (
        <div style={{ display: "flex", flexWrap: "wrap", gap: 4, marginBottom: 10 }}>
          {skill.domains.slice(0, 4).map((domain) => (
            <span key={domain} style={{ fontSize: 10, color: "#7fa8f5", background: "rgba(91,141,239,.1)", borderRadius: 4, padding: "1px 6px" }}>
              {domain}
            </span>
          ))}
        </div>
      )}
      <div style={{ fontFamily: "var(--font-mono)", fontSize: 11, color: commandPreview ? "#9a949f" : "var(--text-muted)", whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" }}>
        {commandPreview || "No commands declared"}
      </div>
      {skill.requires_operator_approval && (
        <div style={{ marginTop: 9, fontSize: 11.5, color: "#e0a23a" }}>
          Operator approval required
        </div>
      )}
    </button>
  );
}

function EmptySkills() {
  return (
    <div
      style={{
        border: "1px dashed var(--border-secondary)",
        borderRadius: 8,
        padding: 34,
        textAlign: "center",
        color: "var(--text-tertiary)",
      }}
    >
      <i className="material-symbols-outlined" style={{ fontSize: 28, color: "var(--text-muted)", marginBottom: 8 }}>
        psychology_alt
      </i>
      <div style={{ fontSize: 14, fontWeight: 700, color: "var(--text-secondary)", marginBottom: 4 }}>
        No project skills found.
      </div>
      <div style={{ fontSize: 12.5 }}>
        Add skill artifacts under .lmbrain/skills to make reusable project procedures visible here.
      </div>
    </div>
  );
}
