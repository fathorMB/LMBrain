---
id: SPEC-048
# Note: Quote the title if it contains a colon
title: "LMBrain 3.0.1 tranche 5: replace default Tauri icon with LMBrain brain mark"
status: review
kind: feature
priority: medium
area: 
milestone: 
# References use IDs only (e.g. [TASK-001]); use [[wikilinks]] in prose
recommended_agent: AGENT-XXX
related_tasks: []
related_decisions: []
links: []
created: 2026-07-18
updated: 2026-07-18
tags: []
activity:
  - date: 2026-07-18
    action: "created"
activity:
  - date: 2026-07-18
    action: "transitioned backlog -> ready"
activity:
  - date: 2026-07-18
    action: "transitioned ready -> working"
activity:
  - date: 2026-07-18
    action: "transitioned working -> review"
---
# LMBrain 3.0.1 tranche 5: replace default Tauri icon with LMBrain brain mark

## Objective
Replace the scaffolded Tauri logo shown by the desktop application and installers with the purple stylized-brain identity already established in the project picker.

## Context
Operator testing identified that the running application still presented Tauri's default icon. The picker rendered the intended identity dynamically from the Material Symbols `neurology` glyph, so no canonical native icon source existed.

## Scope
### Included
- Create a scalable square source using the same purple gradient, rounded tile, and official outlined `neurology` glyph.
- Generate the complete desktop Tauri PNG, ICO, ICNS, Windows tile, and Store assets.
- Include the generated 64 px desktop icon in bundle configuration.
- Reuse the same mark for the web favicon and project-picker header.
- Verify small-size readability and the icon embedded in the Windows executable.

### Excluded
- A broader brand redesign, wordmark, splash screen, or marketing artwork.
- Mobile application packaging.

## Existing-project analysis
`src-tauri/icons` contained Tauri's default cyan/yellow logo and was already referenced correctly by `tauri.conf.json`. The defect is therefore asset provenance, not missing bundle wiring. The picker used a font glyph and CSS gradient that could not be consumed directly by native packagers.

## Technical proposal
Materialize the existing picker composition as an SVG with transparent outer corners, then use the official Tauri icon generator to produce platform assets. Point the picker and favicon at the same visual source to prevent future identity drift.

## Files and areas involved
- `src-tauri/icons/app-icon.svg`
- `src-tauri/icons/*` generated desktop assets
- `src-tauri/tauri.conf.json`
- `public/favicon.svg`
- `src/components/Picker/RepositoryPicker.tsx`
- `src/__tests__/RepositoryPicker.test.tsx`
- `kit/.lmbrain/CHANGELOG.md`

## Acceptance criteria
- [x] The source icon contains the purple rounded tile and white stylized brain used by the picker.
- [x] ICO, ICNS, PNG, Windows tile, and Store assets no longer contain the Tauri logo.
- [x] The 32 px output remains recognizable and preserves transparent outer corners.
- [x] Bundle configuration references the generated desktop icon set.
- [x] The picker and favicon use the same brain mark.
- [x] A rebuilt Windows executable exposes the new associated icon.

## Implementation plan
1. Create the scalable canonical mark from the existing picker design.
2. Generate desktop platform assets with the Tauri CLI.
3. Align favicon and picker with the shared mark.
4. Rebuild and inspect the Windows executable icon.
5. Run frontend, Rust/build, and diff gates.

## Required verification
- `pnpm exec vitest run src/__tests__/RepositoryPicker.test.tsx`
- `pnpm lint`
- `pnpm test`
- `pnpm build`
- `cargo check -p lmbrain --all-targets`
- Windows executable associated-icon extraction and visual inspection
- `git diff --check`

## Production quality and documentation
- Follow [[QUALITY]]; this is production work, not a prototype.
- Identify and update all relevant technical LMBrain knowledge pages delegated by this spec.
- Report any quality-policy exception explicitly; do not silently accept shortcuts.

## Risks and open decisions
Windows may cache taskbar icons for an unchanged executable path; verification inspects the rebuilt executable resource directly, while manual testing uses a restarted process.

## Instructions for the assigned specialist
- If this spec is in `ready`, run `spec_start` as your first implementation action and `spec_submit` when the implementation is complete. If this spec is already in `review` for remediation, do not move it back to `working`; update evidence and report completion for re-review.
- Implement only the stated scope.
- Report changed files, tests run, and known limitations.
- Produce production-grade, maintainable code; do not ship placeholder, POC, or knowingly incomplete behaviour.
- Update only the technical documentation explicitly delegated by this spec, plus implementation evidence.
- Challenge flawed or fragile technical assumptions and propose the clean alternative; consult current official documentation when material behavior is uncertain or changeable.
- Do not adopt shortcuts without the explicit operator-approved exception required by [[QUALITY]].
- Do not change product scope, roadmap, or ADRs.

## Implementation evidence
> Filled in by the specialist after completion.

### Changes made
- Replaced every existing desktop/platform icon asset generated from the Tauri logo.
- Added a canonical SVG and generated the icon set through `pnpm tauri icon`.
- Reused the brain mark in the picker and favicon.

### Files changed
- `src-tauri/icons/app-icon.svg`
- `src-tauri/icons/32x32.png`, `64x64.png`, `128x128.png`, `128x128@2x.png`, `icon.png`, `icon.ico`, `icon.icns`
- `src-tauri/icons/Square*Logo.png`, `StoreLogo.png`
- `src-tauri/tauri.conf.json`
- `public/favicon.svg`
- `src/components/Picker/RepositoryPicker.tsx`
- `src/__tests__/RepositoryPicker.test.tsx`
- `kit/.lmbrain/CHANGELOG.md`

### Verification performed
- Tauri icon generation completed for all desktop formats.
- 512 px and 32 px PNG outputs visually inspected and recognizable.
- Rebuilt `target/debug/lmbrain.exe`; Windows associated-icon extraction returned the purple brain mark.
- Focused RepositoryPicker test: 1 passed.
- `pnpm lint`: passed.
- `pnpm test`: 26 files / 139 tests passed.
- `pnpm build`: passed with the existing Vite chunk-size advisory only.
- `cargo check -p lmbrain --all-targets`: passed.
- `git diff --check`: passed.

### Verification transcript
```text
$ pnpm tauri icon src-tauri/icons/app-icon.svg
Appx, ICNS, ICO, and desktop PNG icon generation completed
exit code: 0

$ cargo build -p lmbrain
Finished dev profile
exit code: 0

$ extract associated icon from target/debug/lmbrain.exe
Extracted 32 px Windows icon: purple LMBrain brain mark
visual inspection: passed

$ pnpm exec vitest run src/__tests__/RepositoryPicker.test.tsx
Test Files  1 passed (1)
Tests       1 passed (1)

$ pnpm lint
$ eslint .
exit code: 0

$ pnpm test
Test Files  26 passed (26)
Tests       139 passed (139)

$ pnpm build
320 modules transformed
built in 333ms
exit code: 0

$ cargo check -p lmbrain --all-targets
Finished dev profile
exit code: 0

$ git diff --check
exit code: 0
```

### Deviations from the specification
- Generated mobile-only Android and iOS assets were removed because mobile packaging is outside this desktop tranche.

### Handoff status
- [x] Ready for Project Lead review
