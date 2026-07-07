import { buildHandoffPrompt, buildMigrationPrompt } from "../lib/handoffPrompt";

describe("buildHandoffPrompt", () => {
  it("includes the spec path and recommended agent", () => {
    const prompt = buildHandoffPrompt("AGENT-TEST", "SPEC-001", "ready", "SPEC-001-test.md");
    expect(prompt).toContain("AGENT-TEST");
    expect(prompt).toContain("SPEC-001");
    expect(prompt).toContain(".lmbrain/specs/ready/SPEC-001-test.md");
  });

  it("falls back to spec id when filename is not provided", () => {
    const prompt = buildHandoffPrompt("AGENT-TEST", "SPEC-001", "ready");
    expect(prompt).toContain(".lmbrain/specs/ready/SPEC-001.md");
  });

  it("uses 'specialist' when no agent is given", () => {
    const prompt = buildHandoffPrompt(null, "SPEC-001", "ready");
    expect(prompt).toContain("specialist");
    expect(prompt).not.toContain("null");
  });

  it("includes v3 context-economy workflow guidance", () => {
    const prompt = buildHandoffPrompt("AGENT-TEST", "SPEC-001", "ready");
    expect(prompt).toContain("V3 context-economy workflow");
    expect(prompt).toContain("lmbrain_spec_context");
    expect(prompt).toContain("lmbrain_project_digest");
    expect(prompt).toContain("QUALITY.md");
    expect(prompt).toContain("CONTRACT.md");
    expect(prompt).toContain("AGENT.md");
    expect(prompt).toContain("mandatory policy files");
    expect(prompt).toContain("compact spec handoff context");
  });

  it("includes spec_start and spec_submit instructions", () => {
    const prompt = buildHandoffPrompt("AGENT-TEST", "SPEC-001", "ready");
    expect(prompt).toContain("spec_start");
    expect(prompt).toContain("spec_submit");
  });

  it("keeps remediation handoffs in review without lifecycle ping-pong", () => {
    const prompt = buildHandoffPrompt("AGENT-TEST", "SPEC-001", "review", "SPEC-001-test.md");
    expect(prompt).toContain(".lmbrain/specs/review/SPEC-001-test.md");
    expect(prompt).toContain("already in `review`");
    expect(prompt).toContain("do not move it back to `working`");
    expect(prompt).not.toContain("run `spec_start`");
    expect(prompt).not.toContain("run `spec_submit`");
  });

  it("includes the lmbrain-mcp tool usage instruction", () => {
    const prompt = buildHandoffPrompt("AGENT-TEST", "SPEC-001", "ready");
    expect(prompt).toContain("lmbrain-mcp");
    expect(prompt).toContain("never edit status frontmatter");
  });
});

describe("buildMigrationPrompt", () => {
  it("generates migration prompt for migration-available status", () => {
    const prompt = buildMigrationPrompt(
      "E:/workspace",
      "2.1.2",
      "2.2.7",
      "migration-available",
      "E:/Git/LMBrain/kit/.lmbrain"
    );
    expect(prompt).toContain("You are the Project Lead");
    expect(prompt).toContain("E:/workspace");
    expect(prompt).toContain("2.1.2");
    expect(prompt).toContain("2.2.7");
    expect(prompt).toContain("Bundled kit source path: E:/Git/LMBrain/kit/.lmbrain");
    expect(prompt).toContain("authoritative source");
    expect(prompt).toContain("bundled kit `MIGRATIONS.md`");
    expect(prompt).toContain("project's current `.lmbrain/MIGRATIONS.md` only as existing project state");
    expect(prompt).toContain("agents/profiles");
    expect(prompt).toContain("agents/proposals");
    expect(prompt).toContain("merge registry rows/guidance additively");
    expect(prompt).toContain("mnemonic_name");
    expect(prompt).toContain("sticky-review/spec-closeout lifecycle guidance");
    expect(prompt).toContain("MIGRATIONS.md");
    expect(prompt).toContain("CONTRACT.md");
    expect(prompt).toContain("QUALITY.md");
    expect(prompt).toContain("AGENT.md");
    expect(prompt).toContain("STATUS.md");
    expect(prompt).toContain("VERSION");
  });

  it("generates diagnostic prompt for unknown status", () => {
    const prompt = buildMigrationPrompt("E:/workspace", "", "2.2.7", "unknown-project-version");
    expect(prompt).toContain("uncertain or unreadable kit version state");
    expect(prompt).toContain("E:/workspace");
    expect(prompt).toContain("unknown");
    expect(prompt).toContain("unknown-project-version");
  });

  it("generates diagnostic prompt for missing guidance status", () => {
    const prompt = buildMigrationPrompt("E:/workspace", "2.1.2", "2.2.7", "migration-guidance-missing");
    expect(prompt).toContain("migration guidance for the target version was not found");
    expect(prompt).toContain("E:/workspace");
    expect(prompt).toContain("2.1.2");
    expect(prompt).toContain("2.2.7");
  });

  it("normalizes Windows extended paths in migration prompts", () => {
    const prompt = buildMigrationPrompt(
      "E:/workspace",
      "2.3.4",
      "2.4.0",
      "migration-available",
      "\\\\?\\C:\\Program Files\\LMBrain\\kit\\.lmbrain"
    );

    expect(prompt).toContain("Bundled kit source path: C:/Program Files/LMBrain/kit/.lmbrain");
    expect(prompt).not.toContain("\\\\?\\");
    expect(prompt).not.toContain("file:///%3F");
  });
});
