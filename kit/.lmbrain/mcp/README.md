# MCP Capability Registry

MCP documents record capabilities available to agents. A capability can be built locally or configured from an external service, but it cannot become active silently.

The Project Lead may propose and specify an MCP. The user approves any new capability with external access, credentials, or write permissions and manually arranges its implementation/configuration.

## lmbrain-mcp registration

The repository-scoped `lmbrain-mcp` server is registered **automatically** when the LMBrain app opens a workspace.

For Claude Code, LMBrain writes `.mcp.json` at the workspace root, pointing the host at the `lmbrain-mcp` binary with `--root <workspace>`. `lmbrain-mcp.json` here is only a format example.

For Codex, LMBrain writes `.codex/config.toml` at the workspace root with `[mcp_servers.lmbrain]`, and ensures the workspace is trusted in the user's Codex config (`$CODEX_HOME/config.toml`, default `~/.codex/config.toml`). `codex-config.toml` here is only a format example.

The `command` resolves via `LMBRAIN_MCP_BIN`, then a binary next to the app executable, then `PATH`; ensure `lmbrain-mcp` is installed (e.g. `cargo install --path lmbrain-mcp`) or set `LMBRAIN_MCP_BIN` if your host cannot find it.
