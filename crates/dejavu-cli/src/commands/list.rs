use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

pub fn run(path: Option<String>) -> Result<()> {
    let project_path = path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap());

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

    println!("{} {} learned rules:\n", "📋".to_string().bold(), rules.len());

    for rule in &rules {
        let status_badge = match rule.status.as_str() {
            "active" => "active".green(),
            "proposed" => "proposed".yellow(),
            "dead" => "dead".dimmed(),
            _ => rule.status.as_str().normal(),
        };

        println!(
            "  {} [{}] (confidence: {:.2}, fires: {})",
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
