# Specifications

Specifications are implementation-ready handoff documents. The Project Lead creates them; the user manually gives the chosen `SPEC-*.md` to a specialist agent.

Directory/status mapping:

```text
backlog -> ready -> working -> review -> done
                         |        |
                         |        +-- stays in review while changes are requested and remediated
                         +-- implementer-owned transition
any status -> discarded (operator-approved)
```

The Project Lead creates and maintains specs, but the assigned implementer owns `ready -> working` and `working -> review`. The Project Lead owns `review -> done` only after the review is accepted and the implementation is committed.

The specialist completes only the `Implementation evidence` section. The Project Lead reviews only when the user explicitly requests it. A `changes-requested` review does not move the spec back to `working`; remediation happens while the spec remains in `review`.
