#!/usr/bin/env node

/**
 * Stop hook: Process collected tool data and run pattern detection.
 *
 * On session end, calls the Rust binary to:
 * 1. Process the session buffer
 * 2. Run detectors on accumulated data
 * 3. Update pattern/rule database
 */

import { findBinary, getDataDir, collectStdin } from './find-binary.js';
import { spawnSync } from 'child_process';
import { existsSync, readdirSync, unlinkSync } from 'fs';
import { join } from 'path';

async function main() {
  const stdinData = await collectStdin();
  let cwd = process.cwd();
  let sessionId = 'unknown';

  if (stdinData) {
    try {
      const input = JSON.parse(stdinData);
      if (input.cwd) cwd = input.cwd;
      if (input.session_id || input.sessionId) {
        sessionId = input.session_id || input.sessionId;
      }
    } catch {}
  }

  const binary = findBinary();
  if (!binary) {
    process.exit(0);
  }

  const dataDir = getDataDir();
  const bufferDir = join(dataDir, 'buffers');

  // Check if we have buffer data
  if (!existsSync(bufferDir)) {
    process.exit(0);
  }

  const bufferFiles = readdirSync(bufferDir).filter(f => f.endsWith('.jsonl'));
  if (bufferFiles.length === 0) {
    process.exit(0);
  }

  // Process buffer: ingest collected tool events
  const ingestResult = spawnSync(binary, ['ingest', '--buffer-dir', bufferDir, '--path', cwd], {
    encoding: 'utf-8',
    stdio: ['pipe', 'pipe', 'pipe'],
    timeout: 30000,
  });

  if (ingestResult.status === 0) {
    // Clean up processed buffers
    for (const file of bufferFiles) {
      try {
        unlinkSync(join(bufferDir, file));
      } catch {}
    }
  }

  // Run background scan (don't block session exit)
  spawnSync(binary, ['scan', '--path', cwd, '--auto'], {
    encoding: 'utf-8',
    stdio: ['pipe', 'pipe', 'pipe'],
    timeout: 45000,
  });

  process.exit(0);
}

main().catch(() => process.exit(0));
