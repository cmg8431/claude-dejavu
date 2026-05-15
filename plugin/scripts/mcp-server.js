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
  {
    name: 'add_rule',
    description: 'Add a manual rule to CLAUDE.md. Use when user says "remember this", "add a rule", or states a preference.',
    inputSchema: {
      type: 'object',
      properties: {
        text: {
          type: 'string',
          description: 'The rule text to add',
        },
        project_path: {
          type: 'string',
          description: 'Project path',
        },
        scope: {
          type: 'string',
          description: 'Rule scope: project, global, or personal',
          enum: ['project', 'global', 'personal'],
        },
      },
      required: ['text'],
    },
  },
  {
    name: 'edit_rule',
    description: 'Edit an existing rule text. Use when user wants to modify or refine a rule.',
    inputSchema: {
      type: 'object',
      properties: {
        rule_id: {
          type: 'string',
          description: 'Rule ID (e.g. r-001)',
        },
        new_text: {
          type: 'string',
          description: 'New rule text',
        },
        project_path: {
          type: 'string',
          description: 'Project path',
        },
      },
      required: ['rule_id', 'new_text'],
    },
  },
  {
    name: 'remove_rule',
    description: 'Remove a rule. Use when user wants to delete or disable a rule.',
    inputSchema: {
      type: 'object',
      properties: {
        rule_id: {
          type: 'string',
          description: 'Rule ID (e.g. r-001)',
        },
        project_path: {
          type: 'string',
          description: 'Project path',
        },
      },
      required: ['rule_id'],
    },
  },
  {
    name: 'rule_feedback',
    description: 'Get effectiveness feedback for all rules. Shows which rules are working and which are dormant.',
    inputSchema: {
      type: 'object',
      properties: {
        project_path: {
          type: 'string',
          description: 'Project path',
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
      case 'add_rule': {
        const scope = args?.scope || 'project';
        const output = callBinary(['rules', 'add', args.text, '--scope', scope, '--path', projectPath]);
        content = output || `Rule added: "${args.text}"`;
        break;
      }
      case 'edit_rule': {
        const output = callBinary(['rules', 'edit', args.rule_id, args.new_text, '--path', projectPath]);
        content = output || `Rule ${args.rule_id} updated.`;
        break;
      }
      case 'remove_rule': {
        const output = callBinary(['rules', 'remove', args.rule_id, '--path', projectPath]);
        content = output || `Rule ${args.rule_id} removed.`;
        break;
      }
      case 'rule_feedback': {
        const output = callBinary(['rules', 'feedback', '--path', projectPath]);
        content = output || 'No rules to report on.';
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
