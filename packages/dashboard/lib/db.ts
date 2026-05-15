import Database from "better-sqlite3";
import path from "path";
import os from "os";
import fs from "fs";

// ── Types ──

export interface Rule {
  id: string;
  project_path: string;
  scope: string;
  scope_target: string | null;
  text: string;
  confidence: number;
  created_at: string;
  last_fired: string | null;
  fire_count: number;
  source_patterns: string;
  status: string;
}

export interface Pattern {
  id: number;
  detector_type: string;
  project_path: string;
  session_id: string;
  detected_at: string;
  evidence_json: string;
  cluster_id: string | null;
}

export interface FeedItem {
  id: string;
  type: "pattern" | "rule";
  detector_type: string;
  project_path: string;
  text: string;
  timestamp: string;
  confidence?: number;
  status?: string;
  fire_count?: number;
}

export interface OverviewStats {
  totalPatterns: number;
  activeRules: number;
  effectiveness: number;
  totalFires: number;
}

export interface LogEntry {
  timestamp: string;
  level: string;
  component: string;
  message: string;
}

// ── DB connection ──

let db: Database.Database | null = null;

function getDbPath(): string {
  if (process.platform === "darwin") {
    return path.join(
      os.homedir(),
      "Library",
      "Application Support",
      "claude-dejavu",
      "dejavu.db"
    );
  }
  const dataDir =
    process.env.XDG_DATA_HOME || path.join(os.homedir(), ".local", "share");
  return path.join(dataDir, "claude-dejavu", "dejavu.db");
}

export function getDb(): Database.Database {
  if (db) return db;

  const dbPath = getDbPath();

  if (!fs.existsSync(dbPath)) {
    fs.mkdirSync(path.dirname(dbPath), { recursive: true });
  }

  db = new Database(dbPath, { readonly: false });
  db.pragma("journal_mode = WAL");
  db.pragma("foreign_keys = ON");

  db.exec(`
    CREATE TABLE IF NOT EXISTS sessions (
      id TEXT PRIMARY KEY,
      project_path TEXT NOT NULL,
      started_at TEXT NOT NULL DEFAULT (datetime('now')),
      ended_at TEXT,
      message_count INTEGER DEFAULT 0,
      tool_call_count INTEGER DEFAULT 0
    );
    CREATE TABLE IF NOT EXISTS patterns (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      detector_type TEXT NOT NULL,
      project_path TEXT NOT NULL,
      session_id TEXT NOT NULL,
      detected_at TEXT NOT NULL DEFAULT (datetime('now')),
      evidence_json TEXT NOT NULL,
      cluster_id TEXT,
      FOREIGN KEY (session_id) REFERENCES sessions(id)
    );
    CREATE TABLE IF NOT EXISTS rules (
      id TEXT PRIMARY KEY,
      project_path TEXT NOT NULL,
      scope TEXT NOT NULL DEFAULT 'project',
      scope_target TEXT,
      text TEXT NOT NULL,
      confidence REAL NOT NULL DEFAULT 0.0,
      created_at TEXT NOT NULL DEFAULT (datetime('now')),
      last_fired TEXT,
      fire_count INTEGER DEFAULT 0,
      source_patterns TEXT NOT NULL,
      status TEXT NOT NULL DEFAULT 'proposed'
    );
    CREATE TABLE IF NOT EXISTS rule_fires (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      rule_id TEXT NOT NULL,
      session_id TEXT NOT NULL,
      fired_at TEXT NOT NULL DEFAULT (datetime('now')),
      prevented INTEGER DEFAULT 0,
      FOREIGN KEY (rule_id) REFERENCES rules(id),
      FOREIGN KEY (session_id) REFERENCES sessions(id)
    );
    CREATE TABLE IF NOT EXISTS corrections (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      session_id TEXT NOT NULL,
      project_path TEXT NOT NULL,
      prompt_text TEXT NOT NULL,
      correction_type TEXT NOT NULL,
      confidence REAL NOT NULL DEFAULT 0.0,
      matched_text TEXT,
      captured_text TEXT,
      processed INTEGER DEFAULT 0,
      created_at TEXT NOT NULL DEFAULT (datetime('now'))
    );
    CREATE TABLE IF NOT EXISTS promotions (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      rule_text_hash TEXT NOT NULL,
      source_project TEXT NOT NULL,
      target_scope TEXT NOT NULL,
      promoted_at TEXT NOT NULL DEFAULT (datetime('now')),
      original_rule_id TEXT,
      UNIQUE(rule_text_hash, source_project)
    );
  `);

  return db;
}

// ── Query helpers ──

export function getProjects(): string[] {
  const d = getDb();
  const fromPatterns = d
    .prepare("SELECT DISTINCT project_path FROM patterns ORDER BY project_path")
    .all() as { project_path: string }[];
  const fromRules = d
    .prepare("SELECT DISTINCT project_path FROM rules ORDER BY project_path")
    .all() as { project_path: string }[];

  const set = new Set<string>();
  for (const r of fromPatterns) set.add(r.project_path);
  for (const r of fromRules) set.add(r.project_path);
  return Array.from(set).sort();
}

export function getOverviewStats(project?: string): OverviewStats {
  const d = getDb();
  const where = project ? "WHERE project_path = ?" : "";
  const params = project ? [project] : [];

  const totalPatterns =
    (
      d
        .prepare(`SELECT COUNT(*) as c FROM patterns ${where}`)
        .get(...params) as any
    )?.c ?? 0;

  const activeRules =
    (
      d
        .prepare(
          `SELECT COUNT(*) as c FROM rules WHERE status = 'active' ${project ? "AND project_path = ?" : ""}`
        )
        .get(...params) as any
    )?.c ?? 0;

  const totalFires =
    (
      d
        .prepare(
          `SELECT COALESCE(SUM(fire_count), 0) as c FROM rules ${where}`
        )
        .get(...params) as any
    )?.c ?? 0;

  const totalRules =
    (
      d
        .prepare(`SELECT COUNT(*) as c FROM rules ${where}`)
        .get(...params) as any
    )?.c ?? 0;

  const effectiveness =
    totalRules > 0 ? Math.round((activeRules / totalRules) * 100) : 0;

  return { totalPatterns, activeRules, effectiveness, totalFires };
}

export function getFeed(project?: string): FeedItem[] {
  const d = getDb();

  let patterns: Pattern[];
  if (project) {
    patterns = d
      .prepare(
        "SELECT * FROM patterns WHERE project_path = ? ORDER BY detected_at DESC LIMIT 200"
      )
      .all(project) as Pattern[];
  } else {
    patterns = d
      .prepare("SELECT * FROM patterns ORDER BY detected_at DESC LIMIT 200")
      .all() as Pattern[];
  }

  let rules: Rule[];
  if (project) {
    rules = d
      .prepare(
        "SELECT * FROM rules WHERE project_path = ? ORDER BY created_at DESC LIMIT 200"
      )
      .all(project) as Rule[];
  } else {
    rules = d
      .prepare("SELECT * FROM rules ORDER BY created_at DESC LIMIT 200")
      .all() as Rule[];
  }

  const patternItems: FeedItem[] = patterns.map((p) => {
    let text = `${p.detector_type} detected`;
    try {
      const ev = JSON.parse(p.evidence_json);
      text = ev.description || ev.summary || ev.message || text;
    } catch {
      /* ignore */
    }
    return {
      id: `p-${p.id}`,
      type: "pattern",
      detector_type: p.detector_type,
      project_path: p.project_path,
      text,
      timestamp: p.detected_at,
    };
  });

  const ruleItems: FeedItem[] = rules.map((r) => ({
    id: `r-${r.id}`,
    type: "rule",
    detector_type: "rule",
    project_path: r.project_path,
    text: r.text,
    timestamp: r.created_at,
    confidence: r.confidence,
    status: r.status,
    fire_count: r.fire_count,
  }));

  const all = [...patternItems, ...ruleItems];
  all.sort(
    (a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
  );
  return all.slice(0, 50);
}

export function getAllRules(project?: string): Rule[] {
  const d = getDb();
  if (project) {
    return d
      .prepare(
        "SELECT * FROM rules WHERE project_path = ? ORDER BY created_at DESC"
      )
      .all(project) as Rule[];
  }
  return d
    .prepare("SELECT * FROM rules ORDER BY created_at DESC")
    .all() as Rule[];
}

export function getLogEntries(project?: string): LogEntry[] {
  const d = getDb();

  const pWhere = project ? "WHERE project_path = ?" : "";
  const pParams = project ? [project] : [];

  const patterns = d
    .prepare(
      `SELECT * FROM patterns ${pWhere} ORDER BY detected_at DESC LIMIT 100`
    )
    .all(...pParams) as Pattern[];

  const rWhere = project ? "WHERE project_path = ?" : "";
  const rParams = project ? [project] : [];

  const rules = d
    .prepare(
      `SELECT * FROM rules ${rWhere} ORDER BY created_at DESC LIMIT 50`
    )
    .all(...rParams) as Rule[];

  const logs: LogEntry[] = [];

  for (const p of patterns) {
    let msg = `${p.detector_type} detected`;
    try {
      const ev = JSON.parse(p.evidence_json);
      msg = ev.description || ev.summary || ev.message || msg;
    } catch {
      /* ignore */
    }

    const levelMap: Record<string, string> = {
      revert_cycle: "ERROR",
      repeated_error: "ERROR",
      silent_fix: "WARN",
      user_correction: "INFO",
      long_bash: "WARN",
    };

    logs.push({
      timestamp: p.detected_at,
      level: levelMap[p.detector_type] || "INFO",
      component: "Detector",
      message: `[${p.detector_type}] ${msg}`,
    });
  }

  for (const r of rules) {
    logs.push({
      timestamp: r.created_at,
      level: r.status === "dead" ? "WARN" : r.status === "active" ? "INFO" : "DEBUG",
      component: "DB",
      message: `Rule ${r.status}: "${r.text.slice(0, 120)}" (confidence: ${(r.confidence * 100).toFixed(0)}%)`,
    });
  }

  logs.sort(
    (a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
  );
  return logs;
}
