---
name: dejavu-status
description: Quick status check for dejavu. Use when user asks "dejavu status", "how's dejavu doing".
---

# claude-dejavu: Quick Status

You are providing a quick inline status check for claude-dejavu.

## Steps

1. Run the stats command:

```bash
claude-dejavu stats --path "$(pwd)"
```

2. Present the output inline in a compact format. No need for a full report -- just show the key numbers:
   - Total rules (active / proposed / dead)
   - Total fires
   - Last scan time
   - Overall health (good/fair/needs attention)

Keep it to 3-5 lines. Example:

```
dejavu: 6 rules (5 active, 1 proposed) | 12 fires | last scan: 2h ago | health: good
```
