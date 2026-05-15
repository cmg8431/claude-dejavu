---
name: dejavu
description: Review and manage learned antipattern rules. Use when Claude should review its past mistakes, apply learned rules to CLAUDE.md, or check rule effectiveness. Triggers on "/dejavu", "review rules", "what did you learn".
---

# claude-dejavu: Review Learned Rules

You are helping the user review antipattern rules that claude-dejavu has detected from past sessions.

## Steps

1. First, run the dejavu binary to get current status:

```bash
claude-dejavu stats --path "$(pwd)"
```

2. Then list all rules:

```bash
claude-dejavu list --path "$(pwd)"
```

3. Present the rules to the user in a clear format:

For each rule, show:
- **Rule ID** and **status** (active/proposed/dead)
- **Rule text** — what Claude should do differently
- **Detector type** — how it was discovered (revert cycle / repeated error / silent fix)
- **Confidence** and **fire count**
- **Action options**: keep, edit, or remove

4. If there are proposed rules, ask the user to approve them:

```
┌─ Proposed rule r-001 ─ [repeated error] ─────────┐
│ "This project uses pnpm, not npm."                │
│ Evidence: 4 sessions, last 2 weeks                │
│ Confidence: 0.91                                  │
│ Apply? [y/n/edit]                                 │
└───────────────────────────────────────────────────┘
```

5. If the user approves, apply rules to CLAUDE.md:

```bash
claude-dejavu scan --path "$(pwd)"
```

6. After applying, show a summary of what changed in CLAUDE.md.

## Important

- Never modify rules without user confirmation
- Dead rules (no fires in 14+ days) should be suggested for removal
- Rules with high fire counts are valuable — highlight them
- If no rules exist yet, explain that dejavu needs more sessions to detect patterns
