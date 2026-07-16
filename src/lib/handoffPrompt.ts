export function buildHandoffPrompt(
  recommendedAgent: string | null | undefined,
  specId: string,
  status = "ready",
  specFilename?: string | null
): string {
  // Spec files carry a slug (e.g. SPEC-017-controlled-mutation-engine.md), so use
  // the real filename when known; fall back to the bare id only if unavailable.
  const file = specFilename && specFilename.trim() ? specFilename.trim() : `${specId}.md`;
  const lifecycleInstruction =
    status === "ready"
      ? "Before you start, run `spec_start` to move this spec to `working`; only the implementation specialist should perform this transition. Then implement the complete production-grade scope exactly as specified, checking off the spec's acceptance criteria and recording evidence as you go. Preserve the repository's existing work. When you finish, run `spec_submit` to move the spec to `review` for the project lead."
      : status === "review"
        ? "This spec is already in `review`; do not move it back to `working`. Address the review findings while the spec remains in `review`, update implementation evidence as needed, and report completion to the Project Lead for re-review."
        : "Follow the spec lifecycle in `CONTRACT.md`. Do not change lifecycle state unless the assigned role and current status explicitly authorize the transition.";

  return `You are the ${recommendedAgent || "specialist"}. Read \`.lmbrain/specs/${status}/${file}\` in full. Use the repository-scoped \`lmbrain-mcp\` tools for all managed artifact mutations; never edit status frontmatter or move status-directory files by hand. ${lifecycleInstruction}

**V3 context-economy workflow:**
1. Read mandatory policy files (\`QUALITY.md\`, \`CONTRACT.md\`, \`AGENT.md\`) first.
2. Use \`lmbrain_spec_context\` for a compact spec handoff context (linked decisions, agent profile, criteria, files).
3. Read the assigned active agent profile and every applicable skill artifact in full before changing source; the context pack paths and digests identify the exact artifacts.
4. Reconcile every structured verification requirement before implementation. \`before-submit\` gates belong in implementation evidence; \`before-done\` operator/playtest gates require an explicit owner and schedule and are not implementer transcript claims.
5. Use \`lmbrain_project_digest\` for project pulse and active work.
6. Expand to full artifacts or source code when the context pack warns about legacy/unstructured requirements or direct verification requires it.
7. Record evidence when you expand scope beyond the context pack.`;
}

export function buildMigrationPrompt(
  projectPath: string,
  projectVersion: string,
  bundledVersion: string,
  status: string,
  bundledKitPath?: string
): string {
  const isUnknown = status === "unknown-project-version" || status === "unknown-bundled-version";
  const safeBundledKitPath = normalizePromptPath(bundledKitPath);
  const bundledSource = safeBundledKitPath
    ? `Bundled kit source path: ${safeBundledKitPath}\n`
    : "";
  
  if (isUnknown) {
    return `You are the Project Lead. The LMBrain application detected an uncertain or unreadable kit version state for this project.
Project path: ${projectPath}
Project version: ${projectVersion || "unknown"}
Bundled kit version: ${bundledVersion || "unknown"}
${bundledSource}Status: ${status}

The project's own \`.lmbrain/MIGRATIONS.md\` may be stale. If a bundled kit source path is provided above, use that bundled kit as the authoritative source for target-version migration guidance and templates.

Please perform a diagnostic review of the workspace:
1. Work in the current repository at ${projectPath}.
2. Read the mandatory policy files: \`.lmbrain/CONTRACT.md\` and \`.lmbrain/QUALITY.md\`.
3. Check the current project status in \`.lmbrain/STATUS.md\` and read the kit version from \`.lmbrain/VERSION\`.
4. Inspect the git state and make sure the repository is clean or you are in a safe branch.
5. Review the migration guidelines in the bundled kit \`MIGRATIONS.md\` when available; compare them with the project's current \`.lmbrain/MIGRATIONS.md\` only as historical/project-local context.
6. Check whether bundled v3 agent profiles/proposals and registry guidance are missing from the project. Plan additive copies/merges only; preserve existing project-specific agents and proposals.
7. For 2.4.0 or newer, check whether agent profiles need \`mnemonic_name\`, agent proposals need \`proposed_mnemonic_name\`, and lifecycle/spec closeout guidance needs to be merged.
8. Propose a plan to resolve any version parsing or compatibility issues, keeping the operator informed of any risks.
9. Do not blindly overwrite any customized files or project-specific content.
10. Only update the \`VERSION\` file in \`.lmbrain/VERSION\` once the version issues have been resolved and validation checks succeed.`;
  }

  if (status === "migration-guidance-missing") {
    return `You are the Project Lead. The LMBrain application detected that a kit version migration is available from ${projectVersion} to ${bundledVersion}, but migration guidance for the target version was not found in \`MIGRATIONS.md\`.
Project path: ${projectPath}
Project version: ${projectVersion}
Bundled kit version: ${bundledVersion}
${bundledSource}
The project's own \`.lmbrain/MIGRATIONS.md\` may be stale. If a bundled kit source path is provided above, inspect that bundled kit before concluding the migration notes are unavailable.

Please review the workspace and manually determine the required migration steps:
1. Work in the current repository at ${projectPath}.
2. Read the mandatory policy files: \`.lmbrain/CONTRACT.md\`, \`.lmbrain/QUALITY.md\`, and \`.lmbrain/STATUS.md\`.
3. Check the bundled kit \`MIGRATIONS.md\` for target-version notes, then compare the project's current \`.lmbrain/MIGRATIONS.md\` as historical/project-local context.
4. Inspect the git state before performing any writes.
5. Compare the files in the project's \`.lmbrain\` folder with the app's bundled kit templates, profiles, proposals, and registry guidance.
6. Produce a custom migration plan to update the kit to ${bundledVersion} while preserving all project-specific customizations.
7. Add missing bundled profiles/proposals only when their IDs or filenames do not already exist; merge registry guidance additively.
8. For 2.4.0 or newer, add \`mnemonic_name\` / \`proposed_mnemonic_name\` metadata where appropriate and merge the corrected sticky-review/spec-closeout lifecycle guidance.
9. Do not blindly overwrite any customized files or project-specific content.
10. Request operator confirmation before performing any changes.
11. Update the version identifier in \`.lmbrain/VERSION\` only after all manual migration steps and validation succeed.`;
  }

  return `You are the Project Lead. The LMBrain application detected that this project's kit version is older than the application's bundled kit version, and a migration is available.
Project path: ${projectPath}
Project version: ${projectVersion}
Bundled kit version: ${bundledVersion}
${bundledSource}
Use the bundled kit source path above as the authoritative source for ${bundledVersion} migration guidance and templates. Treat the project's current \`.lmbrain/MIGRATIONS.md\` as old project-local context that may not contain the target-version notes.

Please execute the migration workflow:
1. Work in the current repository at ${projectPath}.
2. Read the mandatory policy files: \`.lmbrain/CONTRACT.md\`, \`.lmbrain/QUALITY.md\`, \`.lmbrain/AGENT.md\`, and \`.lmbrain/STATUS.md\`.
3. Read the migration log and requirements in the bundled kit \`MIGRATIONS.md\`; then compare against the project's current \`.lmbrain/MIGRATIONS.md\` only as existing project state.
4. Inspect the git state before performing any writes to ensure a clean working directory or isolated branch.
5. Compare the migration notes across the version interval from ${projectVersion} to ${bundledVersion} to understand all necessary updates.
6. Compare bundled \`agents/profiles/\`, \`agents/proposals/\`, and \`agents/registry.md\` with the project. Add missing bundled v3 profiles/proposals only when IDs or filenames do not already exist, and merge registry rows/guidance additively.
7. For 2.4.0 or newer, add or preserve agent \`mnemonic_name\` values, carry \`proposed_mnemonic_name\` into profile materialization plans, and merge the corrected sticky-review/spec-closeout lifecycle guidance from the bundled kit.
8. Produce a detailed migration plan, preserving any project-specific customizations and existing active/inactive agent statuses.
9. Do NOT blindly overwrite or discard project-specific edits or customized kit files.
10. Present the migration plan and ask for operator confirmation before performing any additive migration writes.
11. If any required changes are breaking or ambiguous, create a migration spec or report the conflicts clearly to the operator.
12. Update the version identifier in \`.lmbrain/VERSION\` to "${bundledVersion}" only after all migration steps and validation succeed, and the operator approves.`;
}

function normalizePromptPath(path?: string | null): string {
  const trimmed = path?.trim();
  if (!trimmed) return "";
  return trimmed
    .replace(/^\\\\\?\\/, "")
    .replace(/^\/\/\?\//, "")
    .replace(/\\/g, "/");
}
