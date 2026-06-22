export function buildHandoffPrompt(
  recommendedAgent: string | null | undefined,
  specId: string,
  status = "ready"
): string {
  return `You are the ${recommendedAgent || "specialist"}. Read \`.lmbrain/specs/${status}/${specId}.md\` in full, then implement the complete production-grade scope exactly as specified. Preserve the repository's existing work.`;
}
