import type { KitDiagnostic } from "../types";

export function buildDiagnosticFixPrompt(diagnostic: KitDiagnostic): string {
  const path = diagnostic.path || "unknown file";
  const message = diagnostic.message;
  const normalizedMessage = message.toLowerCase();

  if (
    normalizedMessage.includes("malformed") ||
    normalizedMessage.includes("yaml") ||
    normalizedMessage.includes("frontmatter")
  ) {
    return `Please fix the malformed frontmatter in the file: ${path}
The parser error message is: ${message}

Instructions:
1. Fix the frontmatter block at the top of the file so that it is valid YAML.
2. Make sure any values containing a colon are enclosed in quotes (e.g. title: "React UI: list").
3. Ensure that references use bare IDs rather than [[wikilinks]].
4. Do not modify the body content of the file or any other files.
5. Preserve all intended field values.`;
  }

  if (normalizedMessage.includes("mismatch") || normalizedMessage.includes("status")) {
    return `There is a status mismatch in the file: ${path}
The status in the frontmatter does not match its directory location.
Conflict details: ${message}

Instructions:
Please align the status:
Either:
- Update the status field in the frontmatter to match the folder the file resides in.
Or:
- Move the file to the folder corresponding to its frontmatter status.
Do not make any other changes to the file or its body.`;
  }

  return `Please fix the issue in the file: ${path}
Error details: ${message}

Instructions:
Resolve the reported error while preserving the rest of the file content and structure.`;
}
