---
name: dejavu-scan
description: Trigger a manual scan for antipatterns. Use when user asks "scan for patterns", "dejavu scan", "check for mistakes".
---

# claude-dejavu: Manual Scan

You are triggering a manual antipattern scan with claude-dejavu.

## Steps

1. Run the scan command:

```bash
claude-dejavu scan --path "$(pwd)"
```

2. Present the results to the user:
   - How many sessions were analyzed
   - New patterns detected (if any)
   - Rules proposed or updated
   - Any changes applied to CLAUDE.md

3. If new rules were proposed, show them and ask the user whether to approve:

```
Found 2 new patterns:

  [proposed] "Always use pnpm, not npm, in this project."
  confidence: 0.85 | evidence: 3 sessions

  [proposed] "Run type-check before committing TypeScript changes."
  confidence: 0.72 | evidence: 2 sessions

Approve these rules? [y/n/edit]
```

4. If no new patterns were found, reassure the user:

```
No new antipatterns detected. Your current rules look good.
```

## Important

- Always show what the scan found, even if it found nothing
- Let the user decide whether to approve proposed rules
- If the scan fails, show the error and suggest checking that claude-dejavu is installed correctly
