---
name: how-it-works
description: Explain how claude-dejavu detects antipatterns and learns rules. Use when the user asks "how does dejavu work?" or "what is this thing doing?".
---

# How claude-dejavu works

Explain the following to the user:

## What it does

claude-dejavu watches your Claude Code sessions for repeated mistakes. When it detects a pattern — like the same error happening across sessions, or code being reverted back and forth — it creates a rule and patches your CLAUDE.md so Claude doesn't make that mistake again.

## The 3-step loop

```
[Session Logs]          [Pattern Detection]        [Rule Suggestion]
tool usage data  ──▶  antipattern detectors  ──▶  CLAUDE.md patch
     ▲                                                  │
     │               [Effectiveness Tracking]            │
     └──────  rule fire counting  ◀─────────────────────┘
```

## 3 Detectors (v0)

1. **Revert Cycle** — Claude edits a file, then someone reverts it, then Claude edits it again. Sign that Claude isn't checking existing logic before changing things.

2. **Repeated Error** — The same error message keeps appearing across sessions (e.g., "command not found: npm" in a pnpm project). Claude keeps falling into the same trap.

3. **Silent Fix** — Claude finishes work, then the user quietly edits the same file without saying anything. The user is correcting Claude's output through action, not words. This is the killer feature that differentiates dejavu from other tools.

## Where data lives

- Rules database: `~/.local/share/claude-dejavu/dejavu.db` (SQLite)
- Session buffers: `~/.local/share/claude-dejavu/buffers/` (temporary)
- Output: `CLAUDE.md` in your project root

## Rule format in CLAUDE.md

```md
<!-- dejavu:rule id=r-001 -->
<!-- dejavu:evidence sessions=[a3f, 9d2] confidence=0.91 fires=4 -->

- This project uses pnpm, not npm. Run pnpm install not npm install.
  ↳ Learned from 4 repeated errors across 3 sessions.
```

The HTML comments are metadata that Claude ignores but dejavu reads/writes for tracking.
