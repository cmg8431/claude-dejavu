#!/usr/bin/env node

/**
 * PostToolUse hook: Collect tool usage data for pattern detection.
 *
 * Captures Edit, Write, and Bash tool uses and appends them to a
 * session buffer file. The Rust binary processes this on session end.
 */

import { getDataDir, collectStdin } from './find-binary.js';
import { existsSync, mkdirSync, appendFileSync } from 'fs';
import { join } from 'path';

async function main() {
  const stdinData = await collectStdin();
  if (!stdinData) {
    process.exit(0);
  }

  let input;
  try {
    input = JSON.parse(stdinData);
  } catch {
    process.exit(0);
  }

  const toolName = input.tool_name || input.toolName;
  if (!toolName) {
    process.exit(0);
  }

  const cwd = input.cwd || process.cwd();
  const sessionId = input.session_id || input.sessionId || 'unknown';

  // Only collect relevant tools
  const trackedTools = ['Edit', 'Write', 'Bash', 'NotebookEdit', 'Read'];
  if (!trackedTools.includes(toolName)) {
    process.exit(0);
  }

  const event = {
    timestamp: new Date().toISOString(),
    session_id: sessionId,
    tool_name: toolName,
    tool_input: input.tool_input || input.toolInput || null,
    tool_response: input.tool_response || input.toolResponse || null,
    cwd,
  };

  // Append to session buffer
  const dataDir = getDataDir();
  const bufferDir = join(dataDir, 'buffers');

  try {
    mkdirSync(bufferDir, { recursive: true });

    const bufferFile = join(bufferDir, `${sessionId}.jsonl`);
    appendFileSync(bufferFile, JSON.stringify(event) + '\n');
  } catch (err) {
    // Silent failure — don't interrupt Claude
    console.error(`[claude-dejavu] buffer write failed: ${err.message}`);
  }

  // Output nothing — PostToolUse hook should be invisible
  process.exit(0);
}

main().catch(() => process.exit(0));
