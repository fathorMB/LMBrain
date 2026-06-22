# Session Handoffs

A handoff preserves the working context when a Project Lead session ends and another Project Lead session must resume it.

The Project Lead creates a handoff only when the human operator requests a session handoff or explicitly asks to close the current session. The next Project Lead reads the latest `HANDOFF-*` in `active/` before continuing project-management work.

A handoff is a context snapshot, not proof of repository state. The receiving agent must validate relevant code, Git state, `.lmbrain` artifacts, and open work before acting on it.

Lifecycle:

```text
ready → consumed → archived
           ↘ superseded
```

Keep at most one active `ready` handoff for a project. Archive or supersede the older one when creating a replacement.
