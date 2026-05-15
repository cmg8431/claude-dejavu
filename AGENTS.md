# AI Agent Guidelines

## Commit Message Convention

```
<type>(<scope>): <description>
```

### Types

| Type | Description |
|------|-------------|
| `feat` | New feature |
| `fix` | Bug fix |
| `refactor` | Code refactoring (no behavior change) |
| `docs` | Documentation only |
| `test` | Adding or updating tests |
| `chore` | Maintenance tasks |
| `perf` | Performance improvements |

## Architecture Overview

claude-dejavu is a **Rust + TypeScript hybrid monorepo** (same pattern as codagotchi):

- **Rust library** (`crates/dejavu-core/`) — JSONL parser, detectors, DB, rule engine
- **Rust binary** (`crates/dejavu-cli/`) — CLI with subcommands (init, scan, list, stats)
- **TypeScript launcher** (`packages/cli/`) — platform detection + `spawnSync` execution

### Core Loop

```
Session logs (.jsonl) → Parser → Detectors → Patterns (SQLite) → Rules → CLAUDE.md patch
                                                                            │
                          Rule effectiveness scoring ◀──────────────────────┘
```

### Detectors (v0)

1. **Revert Cycle** — Edit → revert → re-edit on same file
2. **Repeated Error** — Same error appears across sessions, same fix each time
3. **Silent Fix** — User silently corrects Claude's output without asking

### Key Design Decisions

- **SQLite with WAL mode.** `~/.local/share/claude-dejavu/dejavu.db`
- **CLAUDE.md metadata in HTML comments.** Claude ignores them, dejavu reads/writes them.
- **No traits for detectors.** Each detector is a standalone `detect(&[ParsedSession])` function.
- **`anyhow::Result`** for all fallible functions.

### Adding a New Detector

1. Create `detector/{name}.rs` with `pub fn detect(sessions: &[ParsedSession]) -> Vec<Detection>`
2. Add `pub mod {name};` to `detector/mod.rs`
3. Add call in `run_all_detectors()`

### Adding a New CLI Command

1. Create `commands/{name}.rs` with `pub fn run(...) -> Result<()>`
2. Add `pub mod {name};` to `commands/mod.rs`
3. Add variant to `Commands` enum in `main.rs`
4. Add match arm in `main()`

## Code Conventions

- Rust edition 2024, workspace dependencies in root `Cargo.toml`
- No comments unless the logic is non-obvious
- Functions that touch DB take `&Connection`, not owned
- CLI output uses `colored` crate
