import { useEffect } from "react";
import { useWorkspace } from "../../hooks/useWorkspace";
import { getSpecs, getTasks } from "../../lib/commands";

export function RoadmapView() {
  const { state, dispatch } = useWorkspace();

  useEffect(() => {
    Promise.all([getSpecs(), getTasks()])
      .then(([specs, tasks]) => {
        dispatch({ type: "SET_SPECS", specs });
        dispatch({ type: "SET_TASKS", tasks });
      })
      .catch(console.error);
  }, [dispatch]);

  // Group specs by milestone
  const milestones = new Map<string, { specs: typeof state.specs; tasks: typeof state.tasks }>();
  for (const spec of state.specs) {
    const m = spec.milestone || "Unassigned";
    if (!milestones.has(m)) milestones.set(m, { specs: [], tasks: [] });
    milestones.get(m)!.specs.push(spec);
  }
  for (const task of state.tasks) {
    const m = task.milestone || "Unassigned";
    if (!milestones.has(m)) milestones.set(m, { specs: [], tasks: [] });
    milestones.get(m)!.tasks.push(task);
  }

  const sortedMilestones = Array.from(milestones.entries()).sort();

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
          Roadmap
        </h1>
        <p
          style={{
            fontSize: 13.5,
            color: "var(--text-tertiary)",
            margin: "0 0 22px",
          }}
        >
          Milestones and their associated specifications and tasks.
        </p>

        {sortedMilestones.length === 0 && (
          <div
            style={{
              textAlign: "center",
              padding: 40,
              color: "var(--text-tertiary)",
            }}
          >
            No milestones defined yet.
          </div>
        )}

        <div style={{ display: "flex", flexDirection: "column", gap: 16 }}>
          {sortedMilestones.map(([milestone, { specs, tasks }]) => {
            const totalTasks = tasks.length;
            const doneTasks = tasks.filter((t) => t.status === "done").length;
            const progress = totalTasks > 0 ? Math.round((doneTasks / totalTasks) * 100) : 0;

            return (
              <div
                key={milestone}
                style={{
                  background: "var(--bg-tertiary)",
                  border: "1px solid var(--border-secondary)",
                  borderRadius: 13,
                  padding: "17px 18px",
                }}
              >
                <div
                  style={{
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "space-between",
                    marginBottom: 12,
                  }}
                >
                  <div
                    style={{
                      display: "flex",
                      alignItems: "center",
                      gap: 9,
                    }}
                  >
                    <i
                      className="material-symbols-outlined"
                      style={{ fontSize: 18, color: "var(--accent-light)" }}
                    >
                      flag
                    </i>
                    <span style={{ fontWeight: 700, fontSize: 14.5 }}>
                      {milestone}
                    </span>
                  </div>
                  <span
                    style={{
                      fontFamily: "var(--font-mono)",
                      fontSize: 11.5,
                      color: "var(--text-tertiary)",
                    }}
                  >
                    {doneTasks}/{totalTasks} tasks
                  </span>
                </div>

                {totalTasks > 0 && (
                  <div
                    style={{
                      display: "flex",
                      alignItems: "center",
                      gap: 12,
                      marginBottom: 12,
                    }}
                  >
                    <div
                      style={{
                        flex: 1,
                        height: 6,
                        background: "#211d27",
                        borderRadius: 3,
                        overflow: "hidden",
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
                    <span
                      style={{
                        fontFamily: "var(--font-mono)",
                        fontSize: 11,
                        fontWeight: 600,
                        color: "#cfc9d6",
                      }}
                    >
                      {progress}%
                    </span>
                  </div>
                )}

                <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
                  {specs.map((spec) => (
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
                    >
                      {spec.id}
                    </span>
                  ))}
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
