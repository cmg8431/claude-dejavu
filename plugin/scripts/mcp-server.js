#!/usr/bin/env node

/**
 * MCP Server for claude-dejavu.
 *
 * Exposes dejavu data to Claude via the Model Context Protocol.
 * Tools: list_rules, get_stats, search_patterns
 */

import { findBinary } from './find-binary.js';
import { spawnSync } from 'child_process';

const PROTOCOL_VERSION = '2024-11-05';

function writeMessage(message) {
  const json = JSON.stringify(message);
  const header = `Content-Length: ${Buffer.byteLength(json)}\r\n\r\n`;
  process.stdout.write(header + json);
}

function readMessage() {
  return new Promise((resolve) => {
    let buffer = '';

    process.stdin.on('data', (chunk) => {
      buffer += chunk.toString();

      while (true) {
        const headerEnd = buffer.indexOf('\r\n\r\n');
        if (headerEnd === -1) break;

        const header = buffer.substring(0, headerEnd);
        const match = header.match(/Content-Length:\s*(\d+)/i);
        if (!match) break;

        const contentLength = parseInt(match[1], 10);
        const contentStart = headerEnd + 4;

        if (buffer.length < contentStart + contentLength) break;

        const content = buffer.substring(contentStart, contentStart + contentLength);
        buffer = buffer.substring(contentStart + contentLength);

        try {
          resolve(JSON.parse(content));
        } catch {
          // Skip malformed messages
        }
      }
    });
  });
}

function callBinary(args) {
  const binary = findBinary();
  if (!binary) return null;

  const result = spawnSync(binary, args, {
    encoding: 'utf-8',
    stdio: ['pipe', 'pipe', 'pipe'],
    timeout: 15000,
  });

  if (result.status !== 0) return null;
  return result.stdout.trim();
}

const TOOLS = [
  {
    name: 'list_rules',
    description: 'List all learned antipattern rules for the current project',
    inputSchema: {
      type: 'object',
      properties: {
        project_path: {
          type: 'string',
          description: 'Project path to list rules for',
        },
      },
    },
  },
  {
    name: 'get_stats',
    description: 'Get rule effectiveness statistics',
    inputSchema: {
      type: 'object',
      properties: {
        project_path: {
          type: 'string',
          description: 'Project path to get stats for',
        },
      },
    },
  },
  {
    name: 'scan_patterns',
    description: 'Scan session logs for new antipatterns',
    inputSchema: {
      type: 'object',
      properties: {
        project_path: {
          type: 'string',
          description: 'Project path to scan',
        },
      },
    },
  },
];

async function handleMessage(message) {
  if (message.method === 'initialize') {
    writeMessage({
      jsonrpc: '2.0',
      id: message.id,
      result: {
        protocolVersion: PROTOCOL_VERSION,
        capabilities: {
          tools: {},
        },
        serverInfo: {
          name: 'claude-dejavu',
          version: '0.1.0',
        },
      },
    });
    return;
  }

  if (message.method === 'notifications/initialized') {
    return;
  }

  if (message.method === 'tools/list') {
    writeMessage({
      jsonrpc: '2.0',
      id: message.id,
      result: { tools: TOOLS },
    });
    return;
  }

  if (message.method === 'tools/call') {
    const { name, arguments: args } = message.params;
    const projectPath = args?.project_path || process.cwd();
    let content = '';

    switch (name) {
      case 'list_rules': {
        const output = callBinary(['inject', '--path', projectPath, '--format', 'json']);
        content = output || JSON.stringify({ rules: [], pattern_count: 0 });
        break;
      }
      case 'get_stats': {
        const output = callBinary(['stats', '--path', projectPath]);
        content = output || 'No stats available.';
        break;
      }
      case 'scan_patterns': {
        const output = callBinary(['scan', '--path', projectPath, '--auto']);
        content = output || 'Scan complete.';
        break;
      }
      default:
        content = `Unknown tool: ${name}`;
    }

    writeMessage({
      jsonrpc: '2.0',
      id: message.id,
      result: {
        content: [{ type: 'text', text: content }],
      },
    });
    return;
  }

  // Unknown method
  writeMessage({
    jsonrpc: '2.0',
    id: message.id,
    error: {
      code: -32601,
      message: `Method not found: ${message.method}`,
    },
  });
}

async function main() {
  process.stdin.setEncoding('utf-8');

  while (true) {
    const message = await readMessage();
    await handleMessage(message);
  }
}

main().catch((err) => {
  console.error(`[claude-dejavu] MCP server error: ${err.message}`);
  process.exit(1);
});
