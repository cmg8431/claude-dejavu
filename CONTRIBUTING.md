# Contributing to claude-dejavu

## Getting Started

```bash
git clone https://github.com/cmg8431/claude-dejavu.git
cd claude-dejavu
cargo build --workspace
cargo test --workspace
```

Requires: Rust (edition 2024), Node.js 18+

## Project Structure

```
crates/dejavu-core/     # Rust library (parser, detectors, DB, rules)
crates/dejavu-cli/      # Rust CLI binary
packages/cli/           # npm launcher (TypeScript)
packages/dashboard/     # Web dashboard (Next.js)
plugin/                 # Claude Code plugin (hooks, skills, MCP)
```

## Development Workflow

### Run locally

```bash
cargo run -p dejavu-cli -- scan           # Run scan
cargo run -p dejavu-cli -- stats          # View stats
npm run dashboard                         # Start dashboard
```

### Before submitting

```bash
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

All three must pass. CI enforces this.

## Adding a New Detector

1. Create `crates/dejavu-core/src/detector/{name}.rs`
2. Implement: `pub fn detect(sessions: &[ParsedSession]) -> Vec<Detection>`
3. Add `pub mod {name};` to `detector/mod.rs`
4. Add call in `run_all_detectors()`
5. Add label in `crates/dejavu-cli/src/commands/scan.rs`
6. Add tests (minimum 3)

## Adding a New CLI Command

1. Create `crates/dejavu-cli/src/commands/{name}.rs`
2. Add `pub mod {name};` to `commands/mod.rs`
3. Add variant to `Commands` enum in `main.rs`
4. Add match arm in `main()`

## Adding a Slash Command

Create `plugin/skills/{name}/SKILL.md` with YAML frontmatter:

```yaml
---
name: dejavu-{name}
description: When to trigger this skill.
---
```

## Commit Convention

```
<type>(<scope>): <description>
```

| Type | When |
|------|------|
| `feat` | New feature |
| `fix` | Bug fix |
| `refactor` | No behavior change |
| `test` | Tests only |
| `docs` | Documentation |
| `chore` | Maintenance |
| `perf` | Performance |

Examples:
```
feat(detector): add asset path confusion detector
fix(parser): handle empty JSONL lines
test(core): add revert cycle edge cases
docs: update Korean README
```

## Release Process

1. Merge to `main`
2. Push tag: `git tag v0.X.0 && git push origin v0.X.0`
3. GitHub Actions automatically:
   - Builds 5-platform binaries
   - Creates GitHub Release
   - Publishes to npm

## Areas to Contribute

### Easy

- More correction patterns (Chinese, German, Spanish, etc.)
- Dashboard UI improvements
- Better error messages in CLI
- More unit tests

### Medium

- New detectors (asset path confusion, import cycle, etc.)
- Cross-session regression detection for Silent Fix
- Dashboard project selector filtering
- `config.toml` validation

### Hard

- Haiku-based rule text refinement (natural language polish)
- Real-time file system watcher (replace polling)
- Browser extension for dashboard
- Plugin marketplace submission

## Code Style

- Rust edition 2024, `anyhow::Result` for errors
- No comments unless logic is non-obvious
- No `unwrap()` — use `?` or `unwrap_or_else`
- CSS Modules for dashboard (no Tailwind)
- Functions that touch DB take `&Connection`, not owned
