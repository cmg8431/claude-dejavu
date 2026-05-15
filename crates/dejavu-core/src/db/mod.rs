use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;

const SCHEMA: &str = include_str!("schema.sql");

pub fn open(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    conn.execute_batch(SCHEMA)?;
    Ok(conn)
}

pub fn default_db_path() -> Result<std::path::PathBuf> {
    let data_dir = dirs::data_dir()
        .ok_or_else(|| anyhow::anyhow!("could not find data directory"))?
        .join("claude-dejavu");
    std::fs::create_dir_all(&data_dir)?;
    Ok(data_dir.join("dejavu.db"))
}

pub fn insert_pattern(
    conn: &Connection,
    detector_type: &str,
    project_path: &str,
    session_id: &str,
    evidence_json: &str,
    cluster_id: Option<&str>,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO patterns (detector_type, project_path, session_id, evidence_json, cluster_id)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![detector_type, project_path, session_id, evidence_json, cluster_id],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn insert_rule(
    conn: &Connection,
    id: &str,
    project_path: &str,
    scope: &str,
    scope_target: Option<&str>,
    text: &str,
    confidence: f64,
    source_patterns: &str,
) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO rules (id, project_path, scope, scope_target, text, confidence, source_patterns)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![id, project_path, scope, scope_target, text, confidence, source_patterns],
    )?;
    Ok(())
}

pub fn get_active_rules(conn: &Connection, project_path: &str) -> Result<Vec<Rule>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_path, scope, scope_target, text, confidence, created_at, last_fired, fire_count, status
         FROM rules WHERE project_path = ?1 AND status IN ('active', 'proposed')
         ORDER BY confidence DESC",
    )?;
    let rows = stmt.query_map([project_path], |row| {
        Ok(Rule {
            id: row.get(0)?,
            project_path: row.get(1)?,
            scope: row.get(2)?,
            scope_target: row.get(3)?,
            text: row.get(4)?,
            confidence: row.get(5)?,
            created_at: row.get(6)?,
            last_fired: row.get(7)?,
            fire_count: row.get(8)?,
            status: row.get(9)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn get_pattern_count(conn: &Connection, project_path: &str) -> Result<i64> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM patterns WHERE project_path = ?1",
        [project_path],
        |row| row.get(0),
    )?;
    Ok(count)
}

pub fn insert_correction(
    conn: &Connection,
    session_id: &str,
    project_path: &str,
    prompt_text: &str,
    correction_type: &str,
    confidence: f64,
    matched_text: Option<&str>,
    captured_text: Option<&str>,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO corrections (session_id, project_path, prompt_text, correction_type, confidence, matched_text, captured_text)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![session_id, project_path, prompt_text, correction_type, confidence, matched_text, captured_text],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_unprocessed_corrections(conn: &Connection, project_path: &str) -> Result<Vec<Correction>> {
    let mut stmt = conn.prepare(
        "SELECT id, session_id, project_path, prompt_text, correction_type, confidence, matched_text, captured_text, created_at
         FROM corrections WHERE project_path = ?1 AND processed = 0
         ORDER BY created_at ASC",
    )?;
    let rows = stmt.query_map([project_path], |row| {
        Ok(Correction {
            id: row.get(0)?,
            session_id: row.get(1)?,
            project_path: row.get(2)?,
            prompt_text: row.get(3)?,
            correction_type: row.get(4)?,
            confidence: row.get(5)?,
            matched_text: row.get(6)?,
            captured_text: row.get(7)?,
            created_at: row.get(8)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn mark_corrections_processed(conn: &Connection, ids: &[i64]) -> Result<()> {
    for id in ids {
        conn.execute("UPDATE corrections SET processed = 1 WHERE id = ?1", [id])?;
    }
    Ok(())
}

/// Check if a rule text (normalized) appears across multiple projects → eligible for promotion
pub fn check_cross_project_pattern(conn: &Connection, rule_text_hash: &str) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT project_path FROM rules
         WHERE id IN (SELECT original_rule_id FROM promotions WHERE rule_text_hash = ?1)
         UNION
         SELECT DISTINCT source_project FROM promotions WHERE rule_text_hash = ?1",
    )?;
    let rows = stmt.query_map([rule_text_hash], |row| row.get::<_, String>(0))?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn insert_promotion(
    conn: &Connection,
    rule_text_hash: &str,
    source_project: &str,
    target_scope: &str,
    original_rule_id: Option<&str>,
) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO promotions (rule_text_hash, source_project, target_scope, original_rule_id)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![rule_text_hash, source_project, target_scope, original_rule_id],
    )?;
    Ok(())
}

pub fn get_global_rules(conn: &Connection) -> Result<Vec<Rule>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_path, scope, scope_target, text, confidence, created_at, last_fired, fire_count, status
         FROM rules WHERE scope = 'global' AND status IN ('active', 'proposed')
         ORDER BY confidence DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Rule {
            id: row.get(0)?,
            project_path: row.get(1)?,
            scope: row.get(2)?,
            scope_target: row.get(3)?,
            text: row.get(4)?,
            confidence: row.get(5)?,
            created_at: row.get(6)?,
            last_fired: row.get(7)?,
            fire_count: row.get(8)?,
            status: row.get(9)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn record_rule_fire(
    conn: &Connection,
    rule_id: &str,
    session_id: &str,
    prevented: bool,
) -> Result<()> {
    conn.execute(
        "INSERT INTO rule_fires (rule_id, session_id, prevented) VALUES (?1, ?2, ?3)",
        rusqlite::params![rule_id, session_id, if prevented { 1 } else { 0 }],
    )?;
    conn.execute(
        "UPDATE rules SET fire_count = fire_count + 1, last_fired = datetime('now') WHERE id = ?1",
        [rule_id],
    )?;
    Ok(())
}

pub fn get_dead_rules(conn: &Connection, project_path: &str, days: i64) -> Result<Vec<Rule>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_path, scope, scope_target, text, confidence, created_at, last_fired, fire_count, status
         FROM rules
         WHERE project_path = ?1
           AND status = 'active'
           AND fire_count = 0
           AND julianday('now') - julianday(created_at) > ?2
         ORDER BY created_at ASC",
    )?;
    let rows = stmt.query_map(rusqlite::params![project_path, days], |row| {
        Ok(Rule {
            id: row.get(0)?,
            project_path: row.get(1)?,
            scope: row.get(2)?,
            scope_target: row.get(3)?,
            text: row.get(4)?,
            confidence: row.get(5)?,
            created_at: row.get(6)?,
            last_fired: row.get(7)?,
            fire_count: row.get(8)?,
            status: row.get(9)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn update_rule_status(conn: &Connection, rule_id: &str, status: &str) -> Result<()> {
    conn.execute(
        "UPDATE rules SET status = ?1 WHERE id = ?2",
        rusqlite::params![status, rule_id],
    )?;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub id: String,
    pub project_path: String,
    pub scope: String,
    pub scope_target: Option<String>,
    pub text: String,
    pub confidence: f64,
    pub created_at: String,
    pub last_fired: Option<String>,
    pub fire_count: i64,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct Correction {
    pub id: i64,
    pub session_id: String,
    pub project_path: String,
    pub prompt_text: String,
    pub correction_type: String,
    pub confidence: f64,
    pub matched_text: Option<String>,
    pub captured_text: Option<String>,
    pub created_at: String,
}
