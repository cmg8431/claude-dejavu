use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

pub fn run(path: Option<String>) -> Result<()> {
    let project_path = path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let engine = dejavu_core::DejavuEngine::new()?;
    let conn = dejavu_core::db::open(&engine.db_path)?;

    let rules = dejavu_core::db::get_active_rules(&conn, &project_path.to_string_lossy())?;

    if rules.is_empty() {
        println!("  {} No rules for this project yet.", "ℹ".blue());
        println!(
            "  {}",
            "Run `claude-dejavu scan` to detect patterns.".dimmed()
        );
        return Ok(());
    }

    println!(
        "{} {} learned rules:\n",
        "📋".to_string().bold(),
        rules.len()
    );

    for rule in &rules {
        let status_badge = match rule.status.as_str() {
            "active" => "active".green(),
            "proposed" => "proposed".yellow(),
            "dead" => "dead".dimmed(),
            _ => rule.status.as_str().normal(),
        };

        let grade = rule_quality_grade(rule.confidence, rule.fire_count);
        let grade_badge = match grade {
            'A' => " A ".on_green().white().bold(),
            'B' => " B ".on_cyan().white().bold(),
            'C' => " C ".on_yellow().black().bold(),
            _ => " D ".on_red().white().bold(),
        };

        println!(
            "  {} {} [{}] (confidence: {:.2}, fires: {})",
            grade_badge,
            rule.id.bold(),
            status_badge,
            rule.confidence,
            rule.fire_count,
        );
        println!("    {}", rule.text);

        if let Some(ref last_fired) = rule.last_fired {
            println!("    Last fired: {}", last_fired.dimmed());
        }

        println!();
    }

    Ok(())
}

/// Compute a letter grade for a rule based on confidence and fire count.
///   A: confidence >= 0.8 AND fire_count >= 3
///   B: confidence >= 0.6 AND fire_count >= 1
///   C: confidence >= 0.5
///   D: confidence < 0.5
fn rule_quality_grade(confidence: f64, fire_count: i64) -> char {
    if confidence >= 0.8 && fire_count >= 3 {
        'A'
    } else if confidence >= 0.6 && fire_count >= 1 {
        'B'
    } else if confidence >= 0.5 {
        'C'
    } else {
        'D'
    }
}
