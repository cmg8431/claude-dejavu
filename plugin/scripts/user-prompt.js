#!/usr/bin/env node

/**
 * UserPromptSubmit hook: Capture user corrections in real-time.
 *
 * Detects correction patterns ("no, use X not Y", "don't use X", etc.)
 * and queues them for pattern detection.
 */

import { getDataDir, collectStdin } from './find-binary.js';
import { existsSync, mkdirSync, appendFileSync, readFileSync, writeFileSync } from 'fs';
import { join } from 'path';

// Correction patterns (mirrors Rust detector for real-time capture)
const CORRECTION_PATTERNS = [
  // Explicit
  { re: /(?:remember|rule):\s+(.+)/i, type: 'explicit', confidence: 0.90 },
  { re: /always\s+(?:do|use|check|run)\s+(.+)/i, type: 'explicit', confidence: 0.85 },
  { re: /never\s+(?:do|use|run|delete)\s+(.+)/i, type: 'explicit', confidence: 0.85 },
  // Korean
  { re: /기억해[:\s]+(.+)/i, type: 'explicit', confidence: 0.90 },
  { re: /항상\s+(.+)(?:해|하세요|해야)/i, type: 'explicit', confidence: 0.85 },
  { re: /절대\s+(.+)\s*(?:하지|하면)\s*(?:마|안)/i, type: 'explicit', confidence: 0.85 },
  // Redirect
  { re: /no,?\s+use\s+(.+?)\s+not\s+(.+)/i, type: 'redirect', confidence: 0.80 },
  { re: /use\s+(.+?)\s+instead\s+of\s+(.+)/i, type: 'redirect', confidence: 0.80 },
  { re: /use\s+(.+?)\s+instead/i, type: 'redirect', confidence: 0.75 },
  // Korean redirect
  { re: /(.+)\s*말고\s+(.+)\s*(?:써|사용|쓰세요)/i, type: 'redirect', confidence: 0.80 },
  { re: /(.+)\s*대신\s+(.+)\s*(?:써|사용|쓰세요)/i, type: 'redirect', confidence: 0.80 },
  // Negative
  { re: /don'?t\s+use\s+(.+)/i, type: 'negative', confidence: 0.75 },
  { re: /stop\s+(?:using|doing)\s+(.+)/i, type: 'negative', confidence: 0.75 },
  // Korean negative
  { re: /(.+)\s*쓰지\s*마/i, type: 'negative', confidence: 0.75 },
  { re: /(.+)\s*하지\s*마/i, type: 'negative', confidence: 0.75 },
  // Preference
  { re: /actually,?\s+(.+)/i, type: 'preference', confidence: 0.60 },
  { re: /i\s+prefer\s+(.+)/i, type: 'preference', confidence: 0.60 },
];

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

  const prompt = input.prompt || input.userPrompt || '';
  if (!prompt || prompt.length < 5) {
    process.exit(0);
  }

  const cwd = input.cwd || process.cwd();
  const sessionId = input.session_id || input.sessionId || 'unknown';

  // Check for correction patterns
  let detected = null;
  for (const pattern of CORRECTION_PATTERNS) {
    const match = prompt.match(pattern.re);
    if (match) {
      detected = {
        type: pattern.type,
        confidence: pattern.confidence,
        matched: match[0],
        captured: match[1] || '',
      };
      break;
    }
  }

  if (!detected) {
    process.exit(0);
  }

  // Queue the correction
  const dataDir = getDataDir();
  const queueDir = join(dataDir, 'corrections');

  try {
    mkdirSync(queueDir, { recursive: true });

    const queueFile = join(queueDir, 'queue.jsonl');
    const entry = {
      timestamp: new Date().toISOString(),
      session_id: sessionId,
      project_path: cwd,
      prompt: prompt.substring(0, 500), // Truncate long prompts
      correction_type: detected.type,
      confidence: detected.confidence,
      matched: detected.matched,
      captured: detected.captured,
    };

    appendFileSync(queueFile, JSON.stringify(entry) + '\n');
  } catch {
    // Silent failure
  }

  // Don't output anything — transparent to user
  process.exit(0);
}

main().catch(() => process.exit(0));
