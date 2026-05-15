use anyhow::Result;
use std::path::PathBuf;

pub fn run(path: Option<String>, format: String) -> Result<()> {
    let project_path = path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    let engine = dejavu_core::DejavuEngine::new()?;
    let conn = dejavu_core::db::open(&engine.db_path)?;

    let rules = dejavu_core::db::get_active_rules(&conn, &project_path.to_string_lossy())?;
    let pattern_count = dejavu_core::db::get_pattern_count(&conn, &project_path.to_string_lossy())?;

    if format == "json" {
        let rules_json: Vec<serde_json::Value> = rules
            .iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.id,
                    "text": r.text,
                    "confidence": r.confidence,
                    "fire_count": r.fire_count,
                    "status": r.status,
                    "evidence": format!(
                        "Confidence: {:.0}%, fired {} times. Created {}.",
                        r.confidence * 100.0,
                        r.fire_count,
                        r.created_at,
                    ),
                })
            })
            .collect();

        let output = serde_json::json!({
            "rules": rules_json,
            "pattern_count": pattern_count,
        });

        println!("{}", serde_json::to_string(&output)?);
    } else {
        if rules.is_empty() {
            return Ok(());
        }

        println!("# claude-dejavu: {} learned rules\n", rules.len());
        for rule in &rules {
            println!("- {}", rule.text);
        }
    }

    Ok(())
}
