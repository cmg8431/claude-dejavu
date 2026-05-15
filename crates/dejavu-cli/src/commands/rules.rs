use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

pub fn add(path: Option<String>, text: String, scope: String) -> Result<()> {
    let project_path = path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let engine = dejavu_core::DejavuEngine::new()?;
    let conn = dejavu_core::db::open(&engine.db_path)?;

    // Generate next rule ID
    let existing = dejavu_core::db::get_active_rules(&conn, &project_path.to_string_lossy())?;
    let next_num = existing.len() + 1;
    let rule_id = format!("r-{:03}", next_num);

    dejavu_core::db::insert_rule(
        &conn,
        &rule_id,
        &project_path.to_string_lossy(),
        &scope,
        None,
        &text,
        1.0, // manual rules get max confidence
        "[]",
    )?;

    // Activate immediately (manual rules don't need proposal stage)
    dejavu_core::db::update_rule_status(&conn, &rule_id, "active")?;

    // Rebuild CLAUDE.md
    rebuild_claude_md(&project_path)?;

    println!("  {} Added {} — \"{}\"", "✓".green(), rule_id.bold(), text);

    Ok(())
}

pub fn edit(path: Option<String>, rule_id: String, new_text: String) -> Result<()> {
    let project_path = path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let engine = dejavu_core::DejavuEngine::new()?;
    let conn = dejavu_core::db::open(&engine.db_path)?;

    dejavu_core::db::update_rule_text(&conn, &rule_id, &new_text)?;

    // Rebuild CLAUDE.md
    rebuild_claude_md(&project_path)?;

    println!(
        "  {} Updated {} — \"{}\"",
        "✓".green(),
        rule_id.bold(),
        new_text
    );

    Ok(())
}

pub fn remove(path: Option<String>, rule_id: String) -> Result<()> {
    let project_path = path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let engine = dejavu_core::DejavuEngine::new()?;
    let conn = dejavu_core::db::open(&engine.db_path)?;

    dejavu_core::db::update_rule_status(&conn, &rule_id, "rejected")?;

    // Rebuild CLAUDE.md
    rebuild_claude_md(&project_path)?;

    println!("  {} Removed {}", "✓".green(), rule_id.bold());

    Ok(())
}

pub fn feedback(path: Option<String>) -> Result<()> {
    let project_path = path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let engine = dejavu_core::DejavuEngine::new()?;
    let conn = dejavu_core::db::open(&engine.db_path)?;

    let rules = dejavu_core::db::get_active_rules(&conn, &project_path.to_string_lossy())?;
    let pattern_count = dejavu_core::db::get_pattern_count(&conn, &project_path.to_string_lossy())?;

    if rules.is_empty() {
        println!("  {} No rules yet.", "ℹ".blue());
        return Ok(());
    }

    println!("{}\n", "📋 Rule Feedback Report".bold());

    let mut effective = 0;
    let mut dead = 0;

    for rule in &rules {
        let grade = match (rule.confidence >= 0.8, rule.fire_count >= 3) {
            (true, true) => "A".green(),
            (true, false) | (false, true) => "B".cyan(),
            _ if rule.fire_count > 0 => "C".yellow(),
            _ => "D".red(),
        };

        let status_icon = if rule.fire_count > 0 {
            effective += 1;
            "🔥"
        } else {
            dead += 1;
            "💤"
        };

        println!(
            "  {} [{}] {} — fires: {}, confidence: {:.0}%",
            status_icon,
            grade,
            rule.id.bold(),
            rule.fire_count,
            rule.confidence * 100.0,
        );
        println!("    {}", rule.text.dimmed());
        if let Some(ref last) = rule.last_fired {
            println!("    Last fired: {}", last.dimmed());
        }
        println!();
    }

    let effectiveness = if rules.is_empty() {
        0.0
    } else {
        (effective as f64 / rules.len() as f64) * 100.0
    };

    println!(
        "  {} patterns | {} rules | {:.0}% effective | {} effective | {} dormant",
        pattern_count.to_string().cyan(),
        rules.len().to_string().cyan(),
        effectiveness,
        effective.to_string().green(),
        dead.to_string().yellow(),
    );

    if dead > 0 {
        println!(
            "\n  {} {} rules have never fired — consider removing with `claude-dejavu rules remove <id>`",
            "💡".to_string().yellow(),
            dead,
        );
    }

    Ok(())
}

fn rebuild_claude_md(project_path: &std::path::Path) -> Result<()> {
    let engine = dejavu_core::DejavuEngine::new()?;
    let conn = dejavu_core::db::open(&engine.db_path)?;
    let active_rules = dejavu_core::db::get_active_rules(&conn, &project_path.to_string_lossy())?;

    let rules_for_patch: Vec<(String, dejavu_core::Detection)> = active_rules
        .iter()
        .map(|r| {
            (
                r.id.clone(),
                dejavu_core::Detection {
                    detector_type: dejavu_core::DetectorType::RepeatedError,
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

    let content = dejavu_core::rule::patch_claude_md(project_path, &rules_for_patch)?;
    dejavu_core::rule::write_claude_md(project_path, &content)?;

    Ok(())
}
