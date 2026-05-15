use anyhow::Result;
use std::path::PathBuf;

pub fn run(path: Option<String>, format: String) -> Result<()> {
    let project_path = path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

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

        // Compute effectiveness: percentage of active rules that have fired at least once
        let active_rules: Vec<_> = rules.iter().filter(|r| r.status == "active").collect();
        let effective = active_rules.iter().filter(|r| r.fire_count > 0).count();
        let effectiveness = if active_rules.is_empty() {
            0.0
        } else {
            (effective as f64 / active_rules.len() as f64) * 100.0
        };

        let proposed = rules.iter().filter(|r| r.status == "proposed").count();

        let mut summary = format!(
            "dejavu: {} rules active | {} patterns detected | {:.0}% effectiveness",
            active_rules.len(),
            pattern_count,
            effectiveness,
        );

        if proposed > 0 {
            summary.push_str(&format!(" | {} new proposals — run /dejavu", proposed,));
        }

        println!("{}\n", summary);

        for rule in &rules {
            println!("- {}", rule.text);
        }
    }

    Ok(())
}
