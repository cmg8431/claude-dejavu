import Database from "better-sqlite3";
import path from "path";
import os from "os";
import fs from "fs";

let db: Database.Database | null = null;

function getDbPath(): string {
  const platform = process.platform;
  let dataDir: string;

  if (platform === "darwin") {
    dataDir = path.join(os.homedir(), "Library", "Application Support", "claude-dejavu");
  } else if (platform === "win32") {
    dataDir = path.join(process.env.APPDATA || path.join(os.homedir(), "AppData", "Roaming"), "claude-dejavu");
  } else {
    dataDir = path.join(
      process.env.XDG_DATA_HOME || path.join(os.homedir(), ".local", "share"),
      "claude-dejavu"
    );
  }

  return path.join(dataDir, "dejavu.db");
}

export function getDb(): Database.Database {
  if (db) return db;

  const dbPath = getDbPath();

  if (!fs.existsSync(dbPath)) {
    // Create directory and empty DB if it doesn't exist
    fs.mkdirSync(path.dirname(dbPath), { recursive: true });
  }

  db = new Database(dbPath, { readonly: false });
  db.pragma("journal_mode = WAL");
  db.pragma("foreign_keys = ON");

  // Ensure tables exist
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

export interface RuleFire {
  id: number;
  rule_id: string;
  session_id: string;
  fired_at: string;
  prevented: number;
}

export function getOverviewStats() {
  const db = getDb();

  const totalPatterns = (db.prepare("SELECT COUNT(*) as count FROM patterns").get() as any)?.count ?? 0;
  const activeRules = (db.prepare("SELECT COUNT(*) as count FROM rules WHERE status = 'active'").get() as any)?.count ?? 0;
  const totalRules = (db.prepare("SELECT COUNT(*) as count FROM rules").get() as any)?.count ?? 0;
  const totalFireCount = (db.prepare("SELECT COALESCE(SUM(fire_count), 0) as count FROM rules").get() as any)?.count ?? 0;
  const totalRuleFires = (db.prepare("SELECT COUNT(*) as count FROM rule_fires").get() as any)?.count ?? 0;
  const preventedFires = (db.prepare("SELECT COUNT(*) as count FROM rule_fires WHERE prevented = 1").get() as any)?.count ?? 0;

  const effectiveness = totalRuleFires > 0 ? Math.round((preventedFires / totalRuleFires) * 100) : 0;

  return {
    totalPatterns,
    activeRules,
    totalRules,
    totalFireCount,
    effectiveness,
  };
}

export function getAllRules(status?: string): Rule[] {
  const db = getDb();
  if (status && status !== "all") {
    return db.prepare("SELECT * FROM rules WHERE status = ? ORDER BY created_at DESC").all(status) as Rule[];
  }
  return db.prepare("SELECT * FROM rules ORDER BY created_at DESC").all() as Rule[];
}

export function getTimelineEvents() {
  const db = getDb();

  const ruleCreations = db.prepare(
    "SELECT id, text, status, created_at as timestamp, 'rule_created' as event_type, confidence, detector_type FROM rules LEFT JOIN (SELECT DISTINCT cluster_id, detector_type FROM patterns) p ON 1=0 ORDER BY created_at DESC"
  ).all() as any[];

  // Get actual rule creation events
  const rules = db.prepare(
    "SELECT id, text, status, created_at as timestamp, 'rule_created' as event_type, confidence FROM rules ORDER BY created_at DESC"
  ).all() as any[];

  // Get fire events
  const fires = db.prepare(
    `SELECT rf.id, r.text, r.id as rule_id, rf.fired_at as timestamp, 'rule_fired' as event_type, rf.prevented
     FROM rule_fires rf JOIN rules r ON rf.rule_id = r.id
     ORDER BY rf.fired_at DESC`
  ).all() as any[];

  // Get dead rules
  const deadRules = db.prepare(
    "SELECT id, text, status, created_at as timestamp, 'rule_died' as event_type, confidence FROM rules WHERE status = 'dead' ORDER BY created_at DESC"
  ).all() as any[];

  // Get pattern detections
  const patterns = db.prepare(
    "SELECT id, detector_type, detected_at as timestamp, 'pattern_detected' as event_type, project_path FROM patterns ORDER BY detected_at DESC LIMIT 50"
  ).all() as any[];

  const events = [...rules, ...fires, ...deadRules, ...patterns];
  events.sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime());

  return events;
}

export function getDetectorStats() {
  const db = getDb();

  const detectorTypes = ["revert_cycle", "repeated_error", "silent_fix", "user_correction"];

  return detectorTypes.map((type) => {
    const patternCount = (db.prepare("SELECT COUNT(*) as count FROM patterns WHERE detector_type = ?").get(type) as any)?.count ?? 0;

    const recentDetections = db.prepare(
      "SELECT id, detector_type, project_path, detected_at, evidence_json FROM patterns WHERE detector_type = ? ORDER BY detected_at DESC LIMIT 5"
    ).all(type) as Pattern[];

    const ruleCount = (db.prepare(
      `SELECT COUNT(DISTINCT r.id) as count FROM rules r
       WHERE EXISTS (
         SELECT 1 FROM patterns p
         WHERE p.detector_type = ?
         AND p.cluster_id IS NOT NULL
       )`
    ).get(type) as any)?.count ?? 0;

    return {
      type,
      patternCount,
      ruleCount,
      recentDetections,
    };
  });
}
