---
id: ADR-001
title: Desktop-first application using Tauri
status: accepted
decision_date: 2026-06-22
decider: user
supersedes: []
superseded_by: []
links: []
tags: [architecture, desktop, tauri]
---

# Desktop-first application using Tauri

## Context

LMBrain must work directly with many local software repositories that contain a `.lmbrain/` directory. Its core value depends on reliable local filesystem access, Markdown read/write operations, file watching, and optional local Git context.

A browser-only application would impose avoidable filesystem permission and synchronization friction. The human operator also needs a quick way to switch among authorized repository folders.

## Decision

LMBrain will be a desktop-first, local-first application built with Tauri.

The user explicitly selects repository folders. The app reads and writes `.lmbrain/` in place; it does not import, duplicate, or move repository content.

The application will keep only local UI state outside repositories, such as recent/pinned workspace paths, preferences, and regenerable search indexes. Project knowledge, planning, task state, specifications, reviews, agent profiles, and MCP records remain versioned Markdown in the repository.

## Consequences

### Positive

- Reliable local filesystem access and file watching.
- Natural multi-repository workspace switching.
- Local/offline use is possible.
- Git-friendly source of truth remains in the project repository.
- Lower runtime footprint is an explicit product preference over an Electron-based shell.

### Constraints

- The application needs a native desktop build and release pipeline.
- Filesystem access must be scoped only to folders explicitly selected by the user.
- UI and native filesystem/Git services must remain clearly separated.
- Any future remote synchronization is an extension, not a replacement for local Markdown.

## Alternatives considered

### Browser-only web application

Rejected for the primary product because local repository access and continuous filesystem synchronization would be less direct.

### Electron desktop application

Not selected initially. It remains a viable fallback if a required Tauri capability becomes a material blocker, but Tauri better matches the product preference for a lean local application.

## Review conditions

Revisit this decision only if Tauri prevents critical filesystem, Git, editor, or cross-platform requirements from being delivered at acceptable complexity.
