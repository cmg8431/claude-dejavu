#!/usr/bin/env node

/**
 * SessionStart hook: Inject learned rules into Claude's context.
 *
 * Calls `claude-dejavu inject --format text` and passes the output
 * as additionalContext so Claude sees it at session start.
 */

import { findBinary, collectStdin } from './find-binary.js';
import { spawnSync } from 'child_process';

async function main() {
  const stdinData = await collectStdin();
  let cwd = process.cwd();

  if (stdinData) {
    try {
      const input = JSON.parse(stdinData);
      if (input.cwd) cwd = input.cwd;
    } catch {}
  }

  const binary = findBinary();

  if (!binary) {
    // No binary — show install hint
    const context = [
      '# claude-dejavu',
      '',
      'claude-dejavu binary not found.',
      'Run `npx claude-dejavu install` to set up.',
    ].join('\n');

    console.log(JSON.stringify({
      hookSpecificOutput: {
        hookEventName: 'SessionStart',
        additionalContext: context,
      },
      continue: true,
      suppressOutput: true,
    }));
    process.exit(0);
  }

  // Get status message from Rust binary (text format)
  const result = spawnSync(binary, ['inject', '--path', cwd, '--format', 'text'], {
    encoding: 'utf-8',
    stdio: ['pipe', 'pipe', 'pipe'],
    timeout: 15000,
  });

  let additionalContext = '';

  if (result.status === 0 && result.stdout.trim()) {
    additionalContext = result.stdout.trim();
  }

  // If no output from inject, check for pending proposals
  if (!additionalContext) {
    const checkResult = spawnSync(binary, ['check', '--path', cwd, '--quiet'], {
      encoding: 'utf-8',
      stdio: ['pipe', 'pipe', 'pipe'],
      timeout: 10000,
    });

    if (checkResult.status === 0 && checkResult.stdout.trim()) {
      try {
        const check = JSON.parse(checkResult.stdout);
        if (check.pending_count > 0) {
          additionalContext = [
            '# claude-dejavu status',
            '',
            `${check.pending_count} new patterns detected. Run \`/dejavu\` to review.`,
            '',
            'Dashboard: http://localhost:7777',
          ].join('\n');
        }
      } catch {}
    }
  }

  // If still nothing, show first-time message
  if (!additionalContext) {
    additionalContext = [
      '# claude-dejavu status',
      '',
      'This project has no learned rules yet. The current session',
      'will be analyzed; subsequent sessions will benefit from',
      'detected antipatterns written to CLAUDE.md.',
      '',
      'Rule injection starts after the first `claude-dejavu scan`.',
      '',
      '`/dejavu` is available to review detected patterns.',
      'Otherwise learning happens passively as you work.',
      '',
      'Dashboard: http://localhost:7777',
      'How it works: `/how-it-works`',
      '',
      'This message disappears once the first rule is applied.',
    ].join('\n');
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
