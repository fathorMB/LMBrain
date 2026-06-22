import { useEffect, useState } from "react";
import { useWorkspace } from "../../hooks/useWorkspace";
import { getRoadmap, getSpecs, getTasks } from "../../lib/commands";
import type { Roadmap, Spec, Task } from "../../types";

export function RoadmapView() {
  const { state, dispatch } = useWorkspace();
  const [roadmap, setRoadmap] = useState<Roadmap | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    Promise.all([getRoadmap(), getSpecs(), getTasks()])
      .then(([rm, specs, tasks]) => {
        setRoadmap(rm);
        dispatch({ type: "SET_SPECS", specs });
        dispatch({ type: "SET_TASKS", tasks });
        setLoading(false);
      })
      .catch((err) => {
        console.error(err);
        setLoading(false);
      });
  }, [dispatch]);

  if (loading) {
    return (
      <div style={{ padding: 40, textAlign: "center", color: "var(--text-tertiary)" }}>
        Loading roadmap...
      </div>
    );
  }

  const definedMilestones = roadmap?.milestones || [];
  const definedMilestoneIds = new Set(definedMilestones.map((m) => m.id));

  // Group specs and tasks
  const specsByMilestone: Record<string, Spec[]> = {};
  const tasksByMilestone: Record<string, Task[]> = {};
  const unmappedSpecs: Spec[] = [];
  const unmappedTasks: Task[] = [];

  for (const spec of state.specs) {
    const m = spec.milestone;
    if (m && definedMilestoneIds.has(m)) {
      if (!specsByMilestone[m]) specsByMilestone[m] = [];
      specsByMilestone[m].push(spec);
    } else {
      unmappedSpecs.push(spec);
    }
  }

  for (const task of state.tasks) {
    const m = task.milestone;
    if (m && definedMilestoneIds.has(m)) {
      if (!tasksByMilestone[m]) tasksByMilestone[m] = [];
      tasksByMilestone[m].push(task);
    } else {
      unmappedTasks.push(task);
    }
  }

  const statusColors: Record<string, { color: string; bg: string }> = {
    active: { color: "#5b8def", bg: "rgba(91,141,239,0.13)" },
    planned: { color: "#8a8d99", bg: "rgba(138,141,153,0.13)" },
    completed: { color: "#46b07d", bg: "rgba(70,176,125,0.13)" },
  };

  const getStatusStyle = (status: string) => {
    return statusColors[status.toLowerCase()] || { color: "var(--text-tertiary)", bg: "var(--bg-secondary)" };
  };

  return (
    <div style={{ overflowY: "auto", height: "100%" }}>
      <div style={{ maxWidth: 920, margin: "0 auto", padding: "24px 36px 70px" }}>
        <h1
          style={{
            fontSize: 24,
            fontWeight: 800,
            letterSpacing: "-.025em",
            margin: "0 0 5px",
          }}
        >
          {roadmap?.title || "Roadmap"}
        </h1>
        <p
          style={{
            fontSize: 13.5,
            color: "var(--text-tertiary)",
            margin: "0 0 22px",
          }}
        >
          Milestones driven by source of truth in{" "}
          <span
            style={{
              fontFamily: "var(--font-mono)",
              fontSize: 12,
              color: "#9a949f",
            }}
          >
            .lmbrain/ROADMAP.md
          </span>
          .
        </p>

        {definedMilestones.length === 0 && (
          <div
            style={{
              textAlign: "center",
              padding: 40,
              color: "var(--text-tertiary)",
              background: "var(--bg-tertiary)",
              border: "1px solid var(--border-secondary)",
              borderRadius: 13,
              marginBottom: 20,
            }}
          >
            No milestones defined in ROADMAP.md.
          </div>
        )}

        <div style={{ display: "flex", flexDirection: "column", gap: 20 }}>
          {definedMilestones.map((m) => {
            const milestoneSpecs = specsByMilestone[m.id] || [];
            const milestoneTasks = tasksByMilestone[m.id] || [];
            const totalTasks = milestoneTasks.length;
            const doneTasks = milestoneTasks.filter((t) => t.status === "done").length;
            const progress = totalTasks > 0 ? Math.round((doneTasks / totalTasks) * 100) : 0;
            const sc = getStatusStyle(m.status);

            return (
              <div
                key={m.id}
                style={{
                  background: "var(--bg-tertiary)",
                  border: "1px solid var(--border-secondary)",
                  borderRadius: 13,
                  padding: "20px 22px",
                  display: "flex",
                  flexDirection: "column",
                  gap: 14,
                }}
              >
                {/* Header Row */}
                <div
                  style={{
                    display: "flex",
                    alignItems: "flex-start",
                    justifyContent: "space-between",
                  }}
                >
                  <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
                    <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                      <span
                        style={{
                          fontFamily: "var(--font-mono)",
                          fontWeight: 700,
                          fontSize: 15,
                          color: "var(--accent-light)",
                        }}
                      >
                        {m.id}
                      </span>
                      <span style={{ fontWeight: 700, fontSize: 16, color: "var(--text-primary)" }}>
                        {m.title}
                      </span>
                    </div>
                    {m.target && (
                      <span style={{ fontSize: 12, color: "var(--text-tertiary)" }}>
                        Target: {m.target}
                      </span>
                    )}
                  </div>
                  <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                    <span
                      style={{
                        fontSize: 11,
                        fontWeight: 700,
                        color: sc.color,
                        background: sc.bg,
                        borderRadius: 5,
                        padding: "3px 8px",
                        letterSpacing: "0.03em",
                      }}
                    >
                      {m.status.toUpperCase()}
                    </span>
                  </div>
                </div>

                {/* Outcome */}
                {m.outcome && (
                  <div
                    style={{
                      fontSize: 13.5,
                      color: "var(--text-secondary)",
                      lineHeight: 1.45,
                      borderLeft: "2px solid #3c3547",
                      paddingLeft: 10,
                    }}
                  >
                    {m.outcome}
                  </div>
                )}

                {/* Progress Bar */}
                {totalTasks > 0 && (
                  <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
                    <div style={{ display: "flex", justifyContent: "space-between", fontSize: 11.5, color: "var(--text-tertiary)" }}>
                      <span>Progress</span>
                      <span>
                        {doneTasks}/{totalTasks} tasks ({progress}%)
                      </span>
                    </div>
                    <div
                      style={{
                        height: 6,
                        background: "#211d27",
                        borderRadius: 3,
                        overflow: "hidden",
                        display: "flex",
                      }}
                    >
                      <div
                        style={{
                          width: `${progress}%`,
                          height: "100%",
                          background: "linear-gradient(90deg,#7c6cf6,#9384f8)",
                          borderRadius: 3,
                        }}
                      />
                    </div>
                  </div>
                )}

                {/* Specifications Pills */}
                {milestoneSpecs.length > 0 && (
                  <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
                    <span style={{ fontSize: 11.5, fontWeight: 600, color: "var(--text-tertiary)" }}>
                      Associated Specs
                    </span>
                    <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
                      {milestoneSpecs.map((spec) => (
                        <span
                          key={spec.id}
                          style={{
                            fontFamily: "var(--font-mono)",
                            fontSize: 11,
                            color: "#bcaef6",
                            background: "rgba(124,108,246,.1)",
                            border: "1px solid rgba(124,108,246,.2)",
                            borderRadius: 5,
                            padding: "3px 8px",
                          }}
                          title={spec.title}
                        >
                          {spec.id}
                        </span>
                      ))}
                    </div>
                  </div>
                )}

                {/* Risks list */}
                {m.risks && m.risks.length > 0 && (
                  <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
                    <span style={{ fontSize: 11.5, fontWeight: 600, color: "var(--text-tertiary)" }}>
                      Identified Risks
                    </span>
                    <ul style={{ margin: 0, paddingLeft: 18, fontSize: 12.5, color: "#f0a89f", display: "flex", flexDirection: "column", gap: 3 }}>
                      {m.risks.map((risk, index) => (
                        <li key={index}>{risk}</li>
                      ))}
                    </ul>
                  </div>
                )}
              </div>
            );
          })}

          {/* Unmapped Artifacts Section */}
          {(unmappedSpecs.length > 0 || unmappedTasks.length > 0) && (
            <div
              style={{
                border: "1px dashed var(--border-secondary)",
                borderRadius: 13,
                padding: "20px 22px",
                display: "flex",
                flexDirection: "column",
                gap: 14,
                marginTop: 10,
              }}
            >
              <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                <i className="material-symbols-outlined" style={{ fontSize: 18, color: "#8a8d99" }}>
                  link_off
                </i>
                <span style={{ fontWeight: 700, fontSize: 15, color: "var(--text-secondary)" }}>
                  Unmapped Artifacts
                </span>
              </div>
              <p style={{ margin: 0, fontSize: 12.5, color: "var(--text-tertiary)" }}>
                These specifications or tasks do not map to any milestone currently defined in ROADMAP.md.
              </p>

              {unmappedSpecs.length > 0 && (
                <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
                  <span style={{ fontSize: 11.5, fontWeight: 600, color: "var(--text-tertiary)" }}>
                    Specs
                  </span>
                  <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
                    {unmappedSpecs.map((spec) => (
                      <span
                        key={spec.id}
                        style={{
                          fontFamily: "var(--font-mono)",
                          fontSize: 11,
                          color: "#8a8d99",
                          background: "rgba(138,141,153,0.1)",
                          border: "1px solid rgba(138,141,153,0.2)",
                          borderRadius: 5,
                          padding: "3px 8px",
                        }}
                        title={spec.title}
                      >
                        {spec.id}
                      </span>
                    ))}
                  </div>
                </div>
              )}

              {unmappedTasks.length > 0 && (
                <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
                  <span style={{ fontSize: 11.5, fontWeight: 600, color: "var(--text-tertiary)" }}>
                    Tasks
                  </span>
                  <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
                    {unmappedTasks.map((task) => (
                      <span
                        key={task.id}
                        style={{
                          fontFamily: "var(--font-mono)",
                          fontSize: 11,
                          color: "#8a8d99",
                          background: "rgba(138,141,153,0.1)",
                          border: "1px solid rgba(138,141,153,0.2)",
                          borderRadius: 5,
                          padding: "3px 8px",
                        }}
                        title={task.title}
                      >
                        {task.id}
                      </span>
                    ))}
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
