---
name: dejavu-rules
description: Manage dejavu rules manually. Use when user says "add a rule", "remember this", "edit rule", "remove rule", "이거 기억해", "룰 추가해", or wants to manage CLAUDE.md rules through conversation.
---

# Manage dejavu Rules

You can help the user manage their claude-dejavu rules through conversation.

## Adding Rules

When the user states a preference or rule, use the `add_rule` MCP tool:

Examples:
- "이 프로젝트에서는 pnpm만 써" → add_rule("This project uses pnpm, not npm.")
- "항상 테스트 먼저 돌려" → add_rule("Always run tests before committing.")
- "Remember to use semantic HTML" → add_rule("Use semantic HTML tags instead of generic divs.")

Or via CLI:
```bash
claude-dejavu rules add "This project uses pnpm, not npm."
```

## Editing Rules

When the user wants to modify a rule:

1. First show current rules using `list_rules`
2. Then use `edit_rule` with the rule ID and new text

```bash
claude-dejavu rules edit r-001 "Use pnpm for all package operations, including scripts."
```

## Removing Rules

```bash
claude-dejavu rules remove r-001
```

## Reviewing Rules with Feedback

Show the user which rules are working and which aren't:

```bash
claude-dejavu rules feedback
```

This shows:
- Fire count per rule (how many times it prevented a mistake)
- Quality grade (A/B/C/D)
- Dormant rules that might need removal

## Guidelines

- When adding rules, write them as clear, actionable instructions
- Translate user's natural language into concise rule text
- If the user speaks Korean/Japanese, still write the rule in English (Claude reads CLAUDE.md in English)
- After adding/editing, confirm what was done and show the rule ID
- Suggest reviewing with `/dejavu` or `rules feedback` periodically
