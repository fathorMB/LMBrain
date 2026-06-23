# Specifications

Specifications are implementation-ready handoff documents. The Project Lead creates them; the user manually gives the chosen `SPEC-*.md` to a specialist agent.

Directory/status mapping:

```text
proposed → ready → in-progress → review → accepted
                              ↘ changes-requested
                              ↘ archived
```

The specialist completes only the `Implementation evidence` section. The Project Lead reviews only when the user explicitly requests it.

Use the repository-scoped `lmbrain-mcp` tools for status changes and `recommended_agent`; implementation evidence remains ordinary Markdown body content.
