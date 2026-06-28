# Release

LMBrain uses one shared release version for the desktop app and bundled kit.

## Version Alignment

The release gate checks:

- `package.json`
- `src-tauri/Cargo.toml`
- `kit/.lmbrain/VERSION`

Run:

```bash
node scripts/check-version.mjs
```

`src-tauri/tauri.conf.json` reads the app version from `../package.json`.

## CI

The GitHub Actions workflow is `.github/workflows/build-installers.yml`.

On pushes to `main`, the workflow:

1. verifies version alignment;
2. compares the current package version with the previous commit;
3. skips installer builds when the version did not change;
4. runs frontend lint/tests and Rust tests;
5. builds the `lmbrain-mcp` binary;
6. prepares the MCP binary as a Tauri sidecar;
7. builds Windows and Linux installers;
8. uploads installers and MCP server artifacts;
9. creates or updates the GitHub Release for the version tag.

## Release Artifacts

The workflow builds:

- Windows NSIS and MSI installers;
- Linux AppImage and Debian package;
- `lmbrain-mcp` binary for each build platform.

Installer builds include the bundled kit from `kit/.lmbrain/`.
