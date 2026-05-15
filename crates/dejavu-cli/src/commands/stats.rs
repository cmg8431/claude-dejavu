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
    let pattern_count = dejavu_core::db::get_pattern_count(&conn, &project_path.to_string_lossy())?;

    println!("{}\n", "📊 claude-dejavu stats".bold());

    println!("  Patterns detected:  {}", pattern_count.to_string().cyan());
    println!("  Rules learned:      {}", rules.len().to_string().cyan());

    let active = rules.iter().filter(|r| r.status == "active").count();
    let proposed = rules.iter().filter(|r| r.status == "proposed").count();
    let dead = rules.iter().filter(|r| r.status == "dead").count();

    println!("    Active:           {}", active.to_string().green());
    println!("    Proposed:         {}", proposed.to_string().yellow());
    println!("    Dead:             {}", dead.to_string().dimmed());

    let total_fires: i64 = rules.iter().map(|r| r.fire_count).sum();
    println!("  Total fires:        {}", total_fires.to_string().cyan());

    let effective = rules
        .iter()
        .filter(|r| r.fire_count > 0 && r.status == "active")
        .count();
    if !rules.is_empty() {
        let effectiveness = (effective as f64 / rules.len() as f64) * 100.0;
        println!("  Effectiveness:      {:.0}%", effectiveness);
    }

    // Suggest dead rule cleanup
    let stale_rules: Vec<_> = rules
        .iter()
        .filter(|r| r.fire_count == 0 && r.status == "active")
        .collect();

    if !stale_rules.is_empty() {
        println!();
        println!(
            "  {} {} rules have never fired — consider removing.",
            "⚠".yellow(),
            stale_rules.len()
        );
    }

    Ok(())
}
