# MCP Capability Registry

MCP documents record capabilities available to agents. A capability can be built locally or configured from an external service, but it cannot become active silently.

The Project Lead may propose and specify an MCP. The user approves any new capability with external access, credentials, or write permissions and manually arranges its implementation/configuration.

## lmbrain-mcp registration

The repository-scoped `lmbrain-mcp` server is registered **automatically**: the LMBrain app writes a `.mcp.json` at the workspace root when the workspace is opened, pointing the host at the `lmbrain-mcp` binary with `--root <workspace>`. `lmbrain-mcp.json` here is only a format example. The `command` resolves via `LMBRAIN_MCP_BIN`, then a binary next to the app executable, then `PATH`; ensure `lmbrain-mcp` is installed (e.g. `cargo install --path lmbrain-mcp`) or set `LMBRAIN_MCP_BIN` if your host cannot find it.
