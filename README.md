<h1 align="center">claude-dejavu</h1>

<p align="center">
  <strong>Claude remembers its mistakes.</strong>
  <br />
  Detects antipatterns from session logs and auto-patches CLAUDE.md.
</p>

<p align="center">
  <a href="README.md">English</a> ·
  <a href="README.ko.md">한국어</a>
</p>

<p align="center">
  <a href="https://www.npmjs.com/package/claude-dejavu"><img src="https://img.shields.io/npm/v/claude-dejavu" alt="npm version" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue" alt="MIT License" /></a>
  <a href="https://github.com/cmg8431/claude-dejavu"><img src="https://img.shields.io/github/stars/cmg8431/claude-dejavu?style=social" alt="GitHub Stars" /></a>
</p>

---

## The Problem

Claude makes the same mistakes across sessions. It runs `npm install` in a pnpm project — again. It reverts code you just fixed — again. It ignores your style preferences — again.

Every session starts from zero. No memory of past failures.

## The Solution

**dejavu** watches your Claude Code sessions in the background. When it detects a repeating antipattern, it writes a rule into `CLAUDE.md` — the one file Claude always reads at session start.

```
Day 1:  Claude runs `npm install` → fails → you fix it
Day 3:  Claude runs `npm install` → fails → you fix it again
Day 5:  Claude runs `npm install` → fails
        ↓
        dejavu: "This project uses pnpm, not npm." → CLAUDE.md
        ↓
Day 6:  Claude reads CLAUDE.md → runs `pnpm install` → works
```

The loop closes itself.

## Install

```bash
npm install -g claude-dejavu
claude-dejavu install
```

That's it. Two commands, zero config. The `install` command:

1. Downloads the platform-specific Rust binary from GitHub Releases
2. Initializes the SQLite database
3. Registers hooks in `~/.claude/settings.local.json`
4. From now on, everything is automatic

> Requires Node.js 18+. The core engine is a standalone Rust binary — no runtime needed.

### Session Start Message

After install, every Claude Code session starts with:

```
SessionStart says: # claude-dejavu status

This project has no learned rules yet. The current session
will be analyzed; subsequent sessions will benefit from
detected antipatterns written to CLAUDE.md.

`/dejavu` is available to review detected patterns.
Dashboard: http://localhost:7777
```

Once rules are learned:

```
SessionStart says: # claude-dejavu status

3 rules active | 12 patterns detected | 78% effectiveness | 5 fires

## Active Rules
- This project uses pnpm, not npm.
- Use semantic HTML tags instead of divs.
- Check existing JWT logic before adding auth middleware.
```

## How It Works

```
   [Session Logs]              [Pattern Detection]           [Rule Suggestion]
   .jsonl files    ──────▶   5 antipattern detectors  ──▶  CLAUDE.md patch
        ▲                                                        │
        │                   [Effectiveness Tracking]              │
        └──────────  rule fire counting  ◀───────────────────────┘
```

1. **Collect** — Hooks capture tool usage, errors, and user corrections in real-time
2. **Detect** — Rust-powered detectors find patterns across sessions
3. **Learn** — High-confidence patterns become rules in CLAUDE.md
4. **Track** — Rule fires are counted; dead rules are flagged for cleanup

All processing happens locally. No API calls, no cloud, no telemetry.

## Detectors

### ① Revert Cycle

Claude edits a file. Someone reverts it. Claude edits it again the same way.

```
Edit auth.ts → revert → re-edit auth.ts → revert → ...
↓
Rule: "In src/api/auth.ts, always check existing JWT verification
       logic before adding new middleware."
```

### ② Repeated Error

The same error keeps appearing across sessions, always followed by the same fix.

```
Session 1: "command not found: npm" → fix: use pnpm
Session 3: "command not found: npm" → fix: use pnpm
Session 5: "command not found: npm" → fix: use pnpm
↓
Rule: "This project uses pnpm, not npm."
```

### ③ Silent Fix

The killer feature. Claude finishes work, then the user **quietly edits the same file** without saying anything. The user is correcting Claude through action, not words.

```
Claude writes <div> everywhere → user silently changes to <article>, <nav>, <section>
↓
Rule: "Use semantic HTML tags (article, section, nav) instead of divs."
```

This is what separates dejavu from everything else. Most tools only learn from **explicit** corrections. dejavu learns from **behavior**.

### ④ User Correction

Explicit corrections detected via regex patterns. Supports English, Korean, and Japanese.

```
"no, use pnpm not npm"       → Rule: Use pnpm, not npm.
"npm 말고 pnpm 쓰세요"        → Rule: Use pnpm, not npm.
"覚えて: pnpmを使うこと"       → Rule: Use pnpm, not npm.
```

### ⑤ Long Bash Session

Detects when Claude gets stuck in a debugging loop with excessive Bash calls.

```
Session average: 10 Bash calls
This session: 45 Bash calls (4.5x)
↓
Rule: "When TypeScript builds fail with TS2307, check tsconfig.json paths."
```

## Commands

| Command | Description |
|---------|-------------|
| `claude-dejavu install` | One-time setup: DB + hooks registration |
| `claude-dejavu uninstall` | Remove hooks, keep learned rules |
| `claude-dejavu scan` | Interactive scan: detect patterns, approve rules [y/n/edit] |
| `claude-dejavu list` | List rules with quality grades (A/B/C/D) |
| `claude-dejavu stats` | Effectiveness statistics |
| `claude-dejavu watch` | Daemon mode: auto-scan every 30s |
| `claude-dejavu cleanup` | Find dead rules (no fires in 14 days) |
| `claude-dejavu ui` | Launch web dashboard at localhost:7777 |
| `claude-dejavu inject` | Output rules for hook injection |
| `claude-dejavu check` | Check pending proposals |
| `claude-dejavu ingest` | Process hook buffers |
| `claude-dejavu init` | Alias for install |

## Claude Code Plugin

### Lifecycle Hooks

dejavu automatically integrates into Claude Code via 4 hooks:

| Hook | What it does |
|------|-------------|
| **SessionStart** | Injects learned rules + status message into context |
| **UserPromptSubmit** | Captures correction patterns ("no, use X not Y") |
| **PostToolUse** | Records Edit/Write/Bash usage to session buffer |
| **Stop** | Ingests buffer + runs automatic pattern detection |

### Slash Commands

| Command | Description |
|---------|-------------|
| `/dejavu` | Review and approve/reject proposed rules |
| `/dejavu-status` | Quick inline status check |
| `/dejavu-scan` | Trigger manual pattern scan |
| `/dejavu-report` | Weekly effectiveness digest |
| `/how-it-works` | Explain dejavu's detection mechanism |

### MCP Server

3 tools exposed for Claude to query dejavu data:

- `list_rules` — Active rules for current project
- `get_stats` — Effectiveness statistics
- `scan_patterns` — Trigger pattern detection

## Web Dashboard

```bash
claude-dejavu ui
```

Opens at `http://localhost:7777`. Dark theme with warm aesthetics.

- **Overview** — Stat grid + pattern/rule feed
- **Rules** — Table with quality grades, confidence bars, fire counts
- **Console** — Live log viewer with level/component filters

Dependencies install automatically on first launch.

## CLAUDE.md Output

Rules are injected with HTML comment metadata that Claude ignores but dejavu tracks:

```md
<!-- dejavu:start -->
## Learned Rules (auto-generated by dejavu)

<!-- dejavu:rule id=r-001 -->
<!-- dejavu:evidence sessions=[a3f, 9d2, 71b] confidence=0.91 fires=4 -->
<!-- dejavu:created=2026-05-15 last-fired=2026-05-20 -->

- This project uses pnpm, not npm. Run pnpm install not npm install.

<!-- dejavu:end -->
```

## Memory Hierarchy

| Priority | Target | Path | Use case |
|----------|--------|------|----------|
| 1 | Rules | `.claude/rules/dejavu.md` | Modular, path-scoped rules |
| 2 | Project | `CLAUDE.md` | Project-wide rules |
| 3 | Local | `CLAUDE.local.md` | Personal (gitignored) |
| 4 | Global | `~/.claude/CLAUDE.md` | Cross-project rules |
| 5 | Agents | `AGENTS.md` | Industry standard |

Cross-project promotion: same pattern in 2+ projects → auto-promoted to global.

## Configuration

Optional. Create `~/.config/claude-dejavu/config.toml`:

```toml
confidence_threshold = 0.5
dead_rule_days = 14
long_bash_threshold_multiplier = 2.5
dashboard_port = 7777
excluded_paths = ["**/node_modules/**", "*.env"]
```

## Comparison

| | claude-reflect | claude-mem | **claude-dejavu** |
|---|---|---|---|
| Trigger | Manual (`/reflect`) | Automatic | **Automatic** |
| Learning source | User corrections | Recording only | **Corrections + behavior** |
| Output | CLAUDE.md | SQLite memory | **CLAUDE.md + tracking** |
| Dashboard | None | Feed viewer | **Feed + rules + console** |
| Dead rule cleanup | Manual | N/A | **Automatic** |
| Multi-language | Partial | N/A | **EN/KO/JA** |
| Silent fix detection | No | No | **Yes** |
| Config file | No | Yes | **Yes** |
| Tests | Unknown | Unknown | **43 tests** |

## Tech Stack

| Layer | Tech |
|-------|------|
| Core engine | Rust (edition 2024, workspace) |
| Database | SQLite (rusqlite, WAL mode) |
| CLI | clap + colored + inquire |
| Plugin | Node.js hooks + MCP server |
| Dashboard | Next.js 15 + CSS Modules |
| Distribution | npm + GitHub Releases |
| CI/CD | GitHub Actions (5-platform build) |

## Architecture

```
claude-dejavu/
├── crates/
│   ├── dejavu-core/           # Rust library
│   │   ├── parser/            # JSONL session parser
│   │   ├── detector/          # 5 antipattern detectors
│   │   ├── db/                # SQLite (6 tables)
│   │   ├── rule/              # CLAUDE.md patcher + target discovery
│   │   └── config.rs          # TOML config support
│   └── dejavu-cli/            # Rust binary (12 subcommands)
├── packages/
│   ├── cli/                   # npm launcher + postinstall
│   └── dashboard/             # Next.js web UI
└── plugin/                    # Claude Code plugin
    ├── hooks/                 # 4 lifecycle hooks
    ├── scripts/               # Hook handlers + MCP server
    └── skills/                # 5 slash commands
```

## Privacy

- **No network requests.** Everything runs locally in a Rust binary.
- **No code content stored.** Only patterns and metadata.
- **No accounts.** No sign-up, no login, no cloud, no telemetry.
- **`<private>` tag support.** Wrap sensitive content to exclude from detection.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT

---

<p align="center">
  <sub>Claude keeps making the same mistakes. Now it remembers.</sub>
</p>
