# AGENTS.md — Monorepo Overview

Photonic is a self-hosted photo management system: a Rust backend (Axum, PostgreSQL,
event sourcing) exposing a REST API, and a native iOS/macOS client (SwiftUI,
OAuth2+PKCE) consuming it. Detailed component guidance lives in
[`server/CLAUDE.md`](server/CLAUDE.md) and [`apple/CLAUDE.md`](apple/CLAUDE.md).

## API contract

`openapi.yaml` at the repo root is the **single source of truth** for the HTTP API.
After changing Rust handlers or utoipa annotations: `cd server && cargo xtask
generate-openapi`. `asyncapi.yaml` documents the event bus contract.

## Worktrees (agent isolation)

Parallel/agent work happens in git worktrees, never by switching branches in the main
checkout. The layout is fixed:

```
photonic/                          # main checkout (the daily IDE window)
└── .work/photonic-<branch>/       # one worktree per workstream
```

- Create worktrees with `wt new photonic <branch>` (creates `.work/photonic-<branch>`,
  best-effort `cargo fetch` prewarm). Plain `git worktree add` is fine as long as the
  path and naming match.
- Never create worktrees outside `.work/`, and never name them `<branch>` alone.
- direnv auto-allows everything under `~/projects`; nix-direnv provides the environment
  on first `cd`. No `direnv allow`, no manual setup.
- List/clean up: `wt list photonic` / `wt prune photonic` (removes merged worktrees).
- Build cache (sccache) is shared across all worktrees via `$HOME/.cache/photonic-sccache`.

## Entry points

- **Server dev loop**: `nix develop .#test` (from repo root) — local Postgres in
  `server/tmpdata/`, migrations run, `cargo test` works.
- **Swift app**: open `apple/Photonic.xcodeproj` in Xcode.
- **Local server + deps**: `docker-compose up` at the repo root.

## CI

Path-filtered workflows in `.github/workflows/`; `server.yml` triggers on `server/**`,
shared API specs, the flake, or cargo config.
