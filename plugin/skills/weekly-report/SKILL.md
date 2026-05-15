---
name: dejavu-report
description: Generate a weekly report of detected patterns and rule effectiveness. Use when user asks "dejavu report", "what did you learn this week", "dejavu weekly".
---

# claude-dejavu: Weekly Report

You are generating a weekly effectiveness report for claude-dejavu.

## Steps

1. First, gather statistics:

```bash
claude-dejavu stats --path "$(pwd)"
```

2. Then list all rules with their grades:

```bash
claude-dejavu list --path "$(pwd)"
```

3. Format a report with the following sections:

### New Patterns This Week

List any rules created in the last 7 days. If stats show no new rules, say "No new patterns detected this week."

### Rules That Fired (Prevented Mistakes)

Highlight rules with fire_count > 0 that have fired recently. These are the rules that actively prevented Claude from repeating past mistakes. Show each rule's grade badge and fire count.

### Dead Rules to Consider Removing

List rules with status "dead" or rules that have not fired in 14+ days with low confidence (< 0.6). Suggest the user remove these to keep the rule set lean.

### Effectiveness Trend

Summarize overall health:
- Total active rules
- Average confidence across active rules
- Total fires this period
- Grade distribution (how many A/B/C/D rules)

Present the report in a clean, readable format:

```
=== claude-dejavu Weekly Report ===

New patterns:     2 discovered
Rules fired:      5 times (prevented 5 mistakes)
Dead rules:       1 candidate for removal
Overall health:   Good (avg confidence: 0.82)

Grade distribution: A:3  B:2  C:1  D:0
```

## Important

- Be concise but informative
- Highlight wins (rules that fired = mistakes prevented)
- If there are no rules yet, encourage the user to run more sessions so dejavu can learn
- If effectiveness is declining (many dead rules, low confidence), suggest running a fresh scan
