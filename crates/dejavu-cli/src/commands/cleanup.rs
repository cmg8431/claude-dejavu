use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

pub fn run(path: Option<String>, days: i64) -> Result<()> {
    let project_path = path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    let engine = dejavu_core::DejavuEngine::new()?;
    let dead_rules = engine.find_dead_rules(&project_path, days)?;

    if dead_rules.is_empty() {
        println!(
            "  {} No dead rules found (all rules have fired within {} days).",
            "✓".green(),
            days
        );
        return Ok(());
    }

    println!(
        "{} {} dead rules (no fires in {} days):\n",
        "🧹".to_string().bold(),
        dead_rules.len(),
        days
    );

    for rule in &dead_rules {
        println!(
            "  {} [{}] {} (created {})",
            rule.id.bold(),
            "dead".dimmed(),
            rule.text,
            rule.created_at.dimmed(),
        );
    }

    println!();
    println!(
        "  Run with {} to mark these as dead and remove from CLAUDE.md.",
        "--apply".cyan()
    );

    Ok(())
}

pub fn run_apply(path: Option<String>, days: i64) -> Result<()> {
    let project_path = path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    let engine = dejavu_core::DejavuEngine::new()?;
    let dead_rules = engine.cleanup_dead_rules(&project_path, days)?;

    if dead_rules.is_empty() {
        println!("  {} No dead rules to clean up.", "✓".green());
        return Ok(());
    }

    // Re-apply remaining active rules to CLAUDE.md
    let conn = dejavu_core::db::open(&engine.db_path)?;
    let active_rules = dejavu_core::db::get_active_rules(&conn, &project_path.to_string_lossy())?;

    // Build detection list from active rules for patching
    let rules_for_patch: Vec<(String, dejavu_core::Detection)> = active_rules
        .iter()
        .map(|r| {
            (
                r.id.clone(),
                dejavu_core::Detection {
                    detector_type: dejavu_core::DetectorType::RepeatedError, // placeholder
                    evidence: dejavu_core::detector::Evidence {
                        sessions: vec![],
                        file_paths: vec![],
                        occurrences: r.fire_count as usize,
                        details: serde_json::json!({}),
                    },
                    confidence: r.confidence,
                    suggested_rule: r.text.clone(),
                },
            )
        })
        .collect();

    let content = dejavu_core::rule::patch_claude_md(&project_path, &rules_for_patch)?;
    dejavu_core::rule::write_claude_md(&project_path, &content)?;

    println!(
        "  {} Removed {} dead rules. {} active rules remain in CLAUDE.md.",
        "✓".green(),
        dead_rules.len(),
        rules_for_patch.len(),
    );

    Ok(())
}
