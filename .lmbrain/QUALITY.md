# Production Quality Policy

Every change made through an LMBrain handoff is production work. “Proof of concept”, “MVP-quality”, temporary shortcuts, placeholders, or knowingly incomplete implementations are not acceptable completion states unless the user has explicitly approved a documented exception.

## Required engineering standard

An assigned specialist must:

- implement a complete, maintainable solution within the approved scope;
- follow the repository's established architecture, conventions, error handling, and security model;
- validate normal flows, failure paths, boundary conditions, and relevant regressions;
- add or update automated tests when the project supports them and testing is applicable;
- avoid unnecessary dependencies, duplication, speculative abstractions, and hidden technical debt;
- document non-obvious behaviour, public contracts, configuration, and operational consequences;
- report all limitations and deviations honestly in the implementation evidence.

## Technical judgement and research

Agents are technical collaborators, not passive order takers. Do not endorse, normalize, or implement an operator input merely because it was requested when it would produce an unsafe, fragile, misleading, or unnecessarily complex result.

When a request has a material technical flaw, an agent must:

1. state the concern plainly and explain the relevant trade-off;
2. propose the cleanest production-grade alternative that remains within the product goal;
3. record a decision, risk, or open question when the choice is durable or consequential;
4. implement the clean alternative when assigned, rather than silently taking a shortcut.

For technology, framework, library, product, platform, security, or API behavior that is material to a decision and may be unfamiliar or change over time, consult current primary documentation before recommending or implementing the approach. Prefer official vendor/project documentation and record the source or the resulting constraint in the relevant spec, ADR, technical knowledge page, or implementation evidence.

## Shortcuts and exceptions

Shortcuts are not valid solutions by default. Do not substitute a prototype, mock, hard-coded assumption, broad permission, disabled safety control, untested workaround, or knowingly incomplete behavior for a clean solution.

An exception is permitted only after explicit operator approval. Document its scope, rationale, risk, expiry condition, and follow-up work in the relevant specification or ADR. Until then, present the shortcut as an option with its cost—not as the recommended implementation.

## Project Lead escalation

When the operator has enabled the escalation authority in `AGENT.md`, the Project Lead may directly repair a repeatedly missed, narrowly bounded acceptance criterion. The implementation must be cleaner and more rigorously verified than a routine handoff: keep the diff minimal, add targeted regression coverage, run all available checks, and document the takeover plus an independent verification pass. Escalation never justifies a shortcut or a broad technical change.

## Documentation is part of done

A change is not complete until the relevant LMBrain documentation is accurate.

The specialist updates only the technical evidence and specifically delegated knowledge pages. The Project Lead maintains project-level documentation, specifications, task state, roadmap, decisions, and reviews.

At minimum, assess whether the change affects:

- its `SPEC-*` implementation evidence;
- linked `TASK-*` evidence and status;
- architecture, codebase map, setup, integrations, or glossary pages;
- ADRs or the need for a new decision;
- `STATUS.md`, roadmap, or backlog.

## Verification standard

Evidence must state what was actually verified, including commands or manual checks where useful. Do not claim tests passed, requirements were met, or documentation was updated without evidence.

## Exceptions

An exception to this policy must be explicitly requested or approved by the human operator and recorded in the relevant spec or ADR with its scope, rationale, risk, and follow-up plan.
