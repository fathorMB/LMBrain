export function buildHandoffPrompt(
  recommendedAgent: string | null | undefined,
  specId: string,
  status = "ready",
  specFilename?: string | null
): string {
  // Spec files carry a slug (e.g. SPEC-017-controlled-mutation-engine.md), so use
  // the real filename when known; fall back to the bare id only if unavailable.
  const file = specFilename && specFilename.trim() ? specFilename.trim() : `${specId}.md`;
  return `You are the ${recommendedAgent || "specialist"}. Read \`.lmbrain/specs/${status}/${file}\` in full. Use the repository-scoped \`lmbrain-mcp\` tools for all managed artifact mutations; never edit status frontmatter or move status-directory files by hand. Before you start, run \`spec_start\` to move this spec to \`working\`. Then implement the complete production-grade scope exactly as specified, checking off the spec's acceptance criteria and recording evidence as you go. Preserve the repository's existing work. When you finish, run \`spec_submit\` to move the spec to \`review\` for the project lead.`;
}
