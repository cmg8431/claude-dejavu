#!/usr/bin/env node

/**
 * SessionStart hook: Inject learned rules into Claude's context.
 *
 * Reads active rules from SQLite and outputs them as additionalContext
 * so Claude sees the rules at the start of every session.
 */

import { findBinary, collectStdin } from './find-binary.js';
import { spawnSync } from 'child_process';

async function main() {
  const stdinData = await collectStdin();
  let cwd = process.cwd();

  // Parse hook input for cwd
  if (stdinData) {
    try {
      const input = JSON.parse(stdinData);
      if (input.cwd) cwd = input.cwd;
    } catch {}
  }

  const binary = findBinary();

  if (!binary) {
    // No binary available — skip silently
    console.log(JSON.stringify({
      hookSpecificOutput: {
        hookEventName: 'SessionStart',
        additionalContext: '',
      },
      continue: true,
      suppressOutput: true,
    }));
    process.exit(0);
  }

  // Call Rust binary to get injectable rules
  const result = spawnSync(binary, ['inject', '--path', cwd, '--format', 'json'], {
    encoding: 'utf-8',
    stdio: ['pipe', 'pipe', 'pipe'],
    timeout: 15000,
  });

  let additionalContext = '';

  if (result.status === 0 && result.stdout.trim()) {
    try {
      const output = JSON.parse(result.stdout);

      if (output.rules && output.rules.length > 0) {
        const ruleCount = output.rules.length;
        const patternCount = output.pattern_count || 0;

        additionalContext = `# claude-dejavu status\n\n`;
        additionalContext += `${ruleCount} learned rules active, ${patternCount} patterns detected.\n\n`;

        for (const rule of output.rules) {
          additionalContext += `- ${rule.text}\n`;
          if (rule.evidence) {
            additionalContext += `  ↳ ${rule.evidence}\n`;
          }
        }

        additionalContext += `\nRun \`/dejavu\` to review rules or \`claude-dejavu stats\` for effectiveness data.\n`;
      }
    } catch {
      // JSON parse failed — use raw output
      additionalContext = result.stdout.trim();
    }
  }

  // Also check for pending proposals
  if (!additionalContext) {
    const scanResult = spawnSync(binary, ['check', '--path', cwd, '--quiet'], {
      encoding: 'utf-8',
      stdio: ['pipe', 'pipe', 'pipe'],
      timeout: 10000,
    });

    if (scanResult.status === 0 && scanResult.stdout.trim()) {
      try {
        const check = JSON.parse(scanResult.stdout);
        if (check.pending_count > 0) {
          additionalContext = `# claude-dejavu\n\n`;
          additionalContext += `dejavu: ${check.pending_count} new patterns detected. Run \`/dejavu\` to review.\n`;
        }
      } catch {}
    }
  }

  console.log(JSON.stringify({
    hookSpecificOutput: {
      hookEventName: 'SessionStart',
      additionalContext,
    },
    continue: true,
    suppressOutput: true,
  }));
}

main().catch(() => {
  console.log(JSON.stringify({ continue: true, suppressOutput: true }));
  process.exit(0);
});
