export function generateRejectedPrompt(path: string, id: string): string {
  return `Please revise the rejected artifact: ${path} (${id})
This artifact has been rejected by the operator.

Instructions:
1. Review the artifact structure and contents.
2. Address the reasons for rejection or make the necessary updates to improve it.
3. Once the revisions are complete, set its status back to "proposed" so it can be reviewed again.
4. Do not make any unrelated changes to other files.`;
}

export function generateSpecApprovalPrompt(id: string, title: string, path: string): string {
  return `Please approve the specification ${id} ("${title}") by transitioning it from backlog to ready.

Artifact path: ${path}
Current status: backlog
Requested transition: backlog → ready

This transition is requested by the operator. Perform it only because the operator explicitly asked for it.

Instructions:
1. Read AGENT.md, CONTRACT.md, and QUALITY.md.
2. Use the lmbrain-mcp spec_ready tool to transition the spec status.
3. Report the resulting path, status, and any diagnostics.`;
}

export function generateAgentActivationPrompt(id: string, title: string, path: string): string {
  return `Please activate the agent profile ${id} ("${title}") by transitioning it from proposed to active.

Artifact path: ${path}
Current status: proposed
Requested transition: proposed → active

This activation is requested by the operator. Perform it only because the operator explicitly asked for it.

Instructions:
1. Read AGENT.md, CONTRACT.md, and QUALITY.md.
2. Use the lmbrain-mcp agent_activate tool to transition the profile status.
3. Report the resulting path, status, and any diagnostics.`;
}

export function generateAdrDecisionPrompt(id: string, title: string, path: string, targetStatus: "accepted" | "rejected"): string {
  const tool = targetStatus === "accepted" ? "adr_accept" : "adr_reject";
  const action = targetStatus === "accepted" ? "accept" : "reject";

  return `Please ${action} the ADR ${id} ("${title}") by transitioning it from proposed to ${targetStatus}.

Artifact path: ${path}
Current status: proposed
Requested transition: proposed -> ${targetStatus}

This decision is requested by the operator. Perform it only because the operator explicitly asked for it.

Instructions:
1. Read AGENT.md, CONTRACT.md, and QUALITY.md.
2. Use the lmbrain-mcp ${tool} tool to transition the ADR status.
3. Report the resulting path, status, and any diagnostics.`;
}
