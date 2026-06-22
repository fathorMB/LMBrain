# LMBrain

LMBrain is a portable Markdown-based project brain: a Project Lead agent maintains analysis, planning, handoffs, reviews, agent profiles, and MCP capability proposals, while work is manually offloaded to specialist agents.

The reusable starter kit lives in [`kit/.lmbrain/`](kit/.lmbrain/README.md). It is designed to be copied into any software repository and later visualized by the desktop application. LMBrain can also initialize this kit into a selected repository that does not yet contain `.lmbrain/`.

The app and kit share one release version. Before changing it, update `package.json`, `src-tauri/Cargo.toml`, and `kit/.lmbrain/VERSION`; `scripts/check-version.mjs` and the installer workflow reject mismatches.

Every push to `main` builds Windows (`.exe`, `.msi`) and Linux (`.AppImage`, `.deb`) installers and uploads them as GitHub Actions artifacts. The first installers are intentionally unsigned; code-signing certificates and release publishing can be added once distribution is ready.

Start with the [operator guide](.lmbrain/OPERATOR.md), then give a Project Lead the [bootstrap prompt](.lmbrain/templates/project-lead-bootstrap-prompt.md).
