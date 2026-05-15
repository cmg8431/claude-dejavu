pub mod db;
pub mod detector;
pub mod parser;
pub mod rule;

use anyhow::Result;
use std::path::Path;

pub use detector::{Detection, DetectorType};
pub use parser::ParsedSession;

pub struct DejavuEngine {
    pub db_path: std::path::PathBuf,
}

impl DejavuEngine {
    pub fn new() -> Result<Self> {
        let db_path = db::default_db_path()?;
        Ok(Self { db_path })
    }

    pub fn with_db_path(db_path: std::path::PathBuf) -> Self {
        Self { db_path }
    }

    pub fn scan(&self, project_path: &Path) -> Result<Vec<Detection>> {
        let claude_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("no home dir"))?
            .join(".claude");

        let session_files = parser::find_session_files(&claude_dir)?;

        let project_encoded = encode_project_path(project_path);

        let mut sessions = Vec::new();
        for file in &session_files {
            match parser::parse_session(file) {
                Ok(session) => {
                    if is_same_project(&session.project_path, &project_encoded, project_path) {
                        sessions.push(session);
                    }
                }
                Err(_) => continue,
            }
        }

        let detections = detector::run_all_detectors(&sessions);

        // Store patterns in DB
        let conn = db::open(&self.db_path)?;
        for detection in &detections {
            let evidence_json = serde_json::to_string(&detection.evidence)?;
            let session_id = detection
                .evidence
                .sessions
                .first()
                .map(|s| s.as_str())
                .unwrap_or("unknown");

            db::insert_pattern(
                &conn,
                detection.detector_type.as_str(),
                &project_path.to_string_lossy(),
                session_id,
                &evidence_json,
                None,
            )?;
        }

        Ok(detections)
    }

    pub fn generate_rules(
        &self,
        project_path: &Path,
        detections: &[Detection],
    ) -> Result<Vec<(String, Detection)>> {
        let conn = db::open(&self.db_path)?;
        let existing_rules = db::get_active_rules(&conn, &project_path.to_string_lossy())?;
        let next_id = existing_rules.len() + 1;

        let rules: Vec<(String, Detection)> = detections
            .iter()
            .filter(|d| d.confidence >= 0.5)
            .enumerate()
            .map(|(i, d)| {
                let id = format!("r-{:03}", next_id + i);
                (id, d.clone())
            })
            .collect();

        for (id, detection) in &rules {
            let source_patterns = serde_json::to_string(&detection.evidence.sessions)?;
            db::insert_rule(
                &conn,
                id,
                &project_path.to_string_lossy(),
                "project",
                None,
                &detection.suggested_rule,
                detection.confidence,
                &source_patterns,
            )?;
        }

        Ok(rules)
    }

    pub fn apply_rules(
        &self,
        project_path: &Path,
        rules: &[(String, Detection)],
    ) -> Result<String> {
        rule::patch_claude_md(project_path, rules)
    }

    /// Track that a rule "fired" — prevented a repeated mistake.
    pub fn record_fire(&self, rule_id: &str, session_id: &str, prevented: bool) -> Result<()> {
        let conn = db::open(&self.db_path)?;
        db::record_rule_fire(&conn, rule_id, session_id, prevented)?;
        Ok(())
    }

    /// Get rules that haven't fired in `days` and suggest removal.
    pub fn find_dead_rules(&self, project_path: &Path, days: i64) -> Result<Vec<db::Rule>> {
        let conn = db::open(&self.db_path)?;
        let rules = db::get_dead_rules(&conn, &project_path.to_string_lossy(), days)?;
        Ok(rules)
    }

    /// Mark dead rules and suggest cleanup.
    pub fn cleanup_dead_rules(&self, project_path: &Path, days: i64) -> Result<Vec<db::Rule>> {
        let conn = db::open(&self.db_path)?;
        let dead = db::get_dead_rules(&conn, &project_path.to_string_lossy(), days)?;
        for rule in &dead {
            db::update_rule_status(&conn, &rule.id, "dead")?;
        }
        Ok(dead)
    }
}

/// Encode a project path the same way Claude Code does for directory names.
fn encode_project_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace('/', "-")
        .trim_start_matches('-')
        .to_string()
}

fn is_same_project(session_project: &str, encoded: &str, target: &Path) -> bool {
    let target_str = target.to_string_lossy();

    // Direct match on encoded directory name
    if session_project == encoded {
        return true;
    }

    // Partial match
    let normalized_session = session_project.replace('\\', "/");
    let normalized_target = target_str.replace('\\', "/");

    normalized_session.contains(&*normalized_target)
        || normalized_target.contains(&normalized_session)
}
