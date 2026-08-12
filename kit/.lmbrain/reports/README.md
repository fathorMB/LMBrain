# Reports

Store optional periodic project reports and agent-run summaries here. Reports are snapshots, not the source of truth: update `STATUS.md`, specs and reviews instead when project state changes.

`lmbrain-kit-feedback.md` is the exception with a dedicated contract: it is an append-only, typed field report about LMBrain itself. The Project Lead maintains it autonomously through `lmbrain_feedback_record`; it is not project status, backlog, or a `DEBT-*` artifact.

Text fields may contain newlines, blank lines, quotes, Unicode, and tabs. The
writer stores them as escaped YAML double-quoted scalars and validates the
serialized report before and after the atomic write. Input character limits
remain enforced per field; failed validation leaves the previous report intact.
