# LMBrain

LMBrain is a portable Markdown-based project brain: a Project Lead agent maintains analysis, planning, handoffs, reviews, agent profiles, and MCP capability proposals, while work is manually offloaded to specialist agents.

The reusable starter kit lives in [`kit/.lmbrain/`](kit/.lmbrain/README.md). It is designed to be copied into any software repository and later visualized by the desktop application. LMBrain can also initialize this kit into a selected repository that does not yet contain `.lmbrain/`.

The app and kit share one release version. Before changing it, update `package.json`, `src-tauri/Cargo.toml`, and `kit/.lmbrain/VERSION`; `scripts/check-version.mjs` and the installer workflow reject mismatches.

On a push to `main`, LMBrain builds installers only when `package.json` has a new version relative to the preceding commit. It then publishes a GitHub Release tagged `vX.Y.Z`, with Windows (`.exe`, `.msi`) and Linux (`.AppImage`, `.deb`) installers as release assets. The first installers are intentionally unsigned; code-signing certificates can be added once distribution is ready.

Start with the [operator guide](.lmbrain/OPERATOR.md), then give a Project Lead the [bootstrap prompt](.lmbrain/templates/project-lead-bootstrap-prompt.md).
