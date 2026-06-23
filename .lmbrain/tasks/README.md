# Tasks

Tasks are board-sized units of work. Their parent directory must match their `status` frontmatter.

```text
backlog | planned | in-progress | review | done | blocked | cancelled
```

Use `spec` to link a task to a handoff document. A task is not automatically proof that a specification is accepted.

## Lifecycle

`backlog → planned → in-progress → review → done`

- **backlog** — emerged from analysis; its spec is not ready yet. New tasks start here.
- **planned** — the Project Lead has prepared a `ready` spec for the task.
- **in-progress** — the implementer's first action when starting work.
- **review** — set by the implementer when finished; stays through the reviewer/implementer ping-pong.
- **done** — set by the reviewer on acceptance.

A task should leave `backlog` only once its spec is ready.

Use `task_plan`, `task_start`, `task_submit`, and `task_complete` from `lmbrain-mcp`; do not manually change a task's status or directory.
