export function buildHandoffPrompt(
  recommendedAgent: string | null | undefined,
  specId: string,
  status = "ready"
): string {
  return `You are the ${recommendedAgent || "specialist"}. Read \`.lmbrain/specs/${status}/${specId}.md\` in full. Before you start, move the task(s) linked to this spec to \`in-progress\`. Then implement the complete production-grade scope exactly as specified. Preserve the repository's existing work. When you finish, move those task(s) to \`review\` for the project lead.`;
}
