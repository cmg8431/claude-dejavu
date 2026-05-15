<h1 align="center">claude-dejavu</h1>

<p align="center">
  <strong>Claude remembers its mistakes.</strong>
  <br />
  Detects antipatterns from session logs and auto-patches CLAUDE.md.
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

## How It Works

```
   [Session Logs]              [Pattern Detection]           [Rule Suggestion]
   .jsonl files    ──────▶   4 antipattern detectors  ──▶  CLAUDE.md patch
        ▲                                                        │
        │                   [Effectiveness Tracking]              │
        └──────────  rule fire counting  ◀───────────────────────┘
```

dejavu runs a **3-stage loop**:

1. **Collect** — Hooks capture tool usage, errors, and user corrections in real-time
2. **Detect** — Rust-powered detectors find patterns across sessions
3. **Learn** — High-confidence patterns become rules in CLAUDE.md

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

### ③ Silent Fix ⭐

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

## Install

```bash
npx claude-dejavu init
```

This will:
1. Create `.dejavu/` directory
2. Initialize the SQLite database
3. Register Claude Code hooks (optional)
4. Start collecting patterns on next session

> Requires Node.js 18+. The core engine is a standalone Rust binary — no runtime needed.

## Commands

| Command | Description |
|---------|-------------|
| `claude-dejavu init` | Initialize for current project |
| `claude-dejavu scan` | Scan sessions, detect patterns, propose rules |
| `claude-dejavu list` | List all learned rules with status |
| `claude-dejavu stats` | Rule effectiveness statistics |
| `claude-dejavu inject` | Output active rules as JSON (for hooks) |
| `claude-dejavu check` | Check for pending pattern proposals |
| `claude-dejavu ingest` | Process tool usage buffers from hooks |

## Claude Code Plugin

dejavu integrates into Claude Code's lifecycle via 4 hooks:

| Hook | Timing | What it does |
|------|--------|-------------|
| **SessionStart** | Session begins | Injects learned rules into Claude's context |
| **UserPromptSubmit** | Every prompt | Captures correction patterns in real-time |
| **PostToolUse** | After Edit/Write/Bash | Records tool usage to session buffer |
| **Stop** | Session ends | Ingests buffer + runs pattern detection |

### Slash Commands

| Command | Description |
|---------|-------------|
| `/dejavu` | Review proposed rules, approve/reject/edit |
| `/how-it-works` | Explain dejavu's detection mechanism |

### MCP Server

Exposes 3 tools for Claude to query dejavu data directly:

- `list_rules` — Active rules for current project
- `get_stats` — Rule effectiveness statistics
- `scan_patterns` — Trigger pattern detection

## CLAUDE.md Output Format

Rules are injected into a dedicated section with HTML comment metadata:

```md
<!-- dejavu:start -->
## Learned Rules (auto-generated by dejavu)

<!-- dejavu:rule id=r-001 -->
<!-- dejavu:evidence sessions=[a3f, 9d2, 71b] confidence=0.91 fires=4 -->
<!-- dejavu:created=2026-05-15 last-fired=2026-05-20 -->

- This project uses pnpm, not npm. Run pnpm install not npm install.
  ↳ Learned from 4 repeated errors across 3 sessions.

<!-- dejavu:end -->
```

Claude ignores HTML comments. dejavu reads and writes them for tracking. Users see the metadata at a glance.

## Memory Hierarchy

dejavu discovers and writes to multiple targets, in priority order:

| Priority | Target | Path | Use case |
|----------|--------|------|----------|
| 1 | Rules | `.claude/rules/dejavu.md` | Modular, path-scoped rules |
| 2 | Project | `CLAUDE.md` | Project-wide rules |
| 3 | Local | `CLAUDE.local.md` | Personal preferences (gitignored) |
| 4 | Global | `~/.claude/CLAUDE.md` | Cross-project rules |
| 5 | Agents | `AGENTS.md` | Industry-standard agent guidelines |

### Cross-Project Promotion

When the same pattern appears in 2+ projects, dejavu promotes it to global scope automatically.

```
Project A: "Use semantic HTML" (confidence: 0.87)
Project B: "Use semantic HTML" (confidence: 0.91)
↓
Promoted to ~/.claude/CLAUDE.md (global)
```

## User Journey

```
Day 1:  claude-dejavu init
        → hooks registered, SQLite initialized

Day 1–7:  (background learning)
        → you use Claude Code normally
        → dejavu silently collects patterns

Day 7:  first notification
        → "dejavu: 5 patterns detected. Run /dejavu to review."

        /dejavu:
        ┌─ Proposed rule r-001 ─ [repeated error] ────────┐
        │ "This project uses pnpm, not npm."               │
        │ Evidence: 4 sessions, last 2 weeks               │
        │ Confidence: 0.91                                 │
        │ Apply? [y/n/edit]                                │
        └──────────────────────────────────────────────────┘

Day 14:  claude-dejavu stats
        → "12 rules applied. 8 prevented repeated mistakes.
           2 dead rules (no fires in 14 days, suggest remove)."
```

## Comparison

| | claude-reflect | claude-mem | **claude-dejavu** |
|---|---|---|---|
| Trigger | Manual (`/reflect`) | Automatic | **Automatic** |
| Learning source | User corrections only | (recording, not learning) | **User corrections + behavior patterns** |
| Output | CLAUDE.md | SQLite memory | **CLAUDE.md + effectiveness tracking** |
| Visualization | None | None | **Web dashboard** (planned) |
| Dead rule cleanup | Manual | N/A | **Automatic suggestion** |
| Multi-language | Partial | N/A | **EN/KO/JA native** |
| Silent fix detection | No | No | **Yes** ⭐ |

## Tech Stack

| Layer | Tech |
|-------|------|
| Core engine | Rust (workspace, edition 2024) |
| Database | SQLite (rusqlite, WAL mode) |
| CLI | clap + colored |
| Plugin hooks | Node.js (ES modules) |
| MCP server | JSON-RPC over stdio |
| Distribution | npm wrapper + platform-specific binaries |

## Architecture

```
claude-dejavu/
├── crates/
│   ├── dejavu-core/        # Rust library
│   │   ├── parser/         # JSONL session parser
│   │   ├── detector/       # 4 antipattern detectors
│   │   ├── db/             # SQLite schema + CRUD
│   │   └── rule/           # CLAUDE.md patcher + target discovery
│   └── dejavu-cli/         # Rust binary (7 subcommands)
├── packages/cli/           # TypeScript npm launcher
└── plugin/                 # Claude Code plugin
    ├── hooks/              # 4 lifecycle hooks
    ├── scripts/            # Hook handlers + MCP server
    └── skills/             # /dejavu, /how-it-works
```

## Privacy

- **No network requests.** Everything runs locally in a Rust binary.
- **No code content stored.** Only patterns and metadata.
- **No accounts.** No sign-up, no login, no cloud, no telemetry.
- **Your data, your disk.** Database lives in `~/.local/share/claude-dejavu/`.

## Contributing

PRs welcome. Some areas to explore:

- New detectors (long bash sessions, asset path confusion, etc.)
- Web dashboard (Next.js, timeline visualization)
- More language patterns (Chinese, German, etc.)
- CI/CD pipeline for cross-platform binary builds

## License

MIT

---

<p align="center">
  <sub>Claude keeps making the same mistakes. Now it remembers.</sub>
</p>
