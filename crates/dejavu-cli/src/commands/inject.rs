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
                })
            })
            .collect();

        let output = serde_json::json!({
            "rules": rules_json,
            "pattern_count": pattern_count,
        });

        println!("{}", serde_json::to_string(&output)?);
    } else {
        // claude-mem style status message for SessionStart injection
        let active_rules: Vec<_> = rules.iter().filter(|r| r.status == "active").collect();
        let proposed = rules.iter().filter(|r| r.status == "proposed").count();
        let effective = active_rules.iter().filter(|r| r.fire_count > 0).count();
        let total_fires: i64 = rules.iter().map(|r| r.fire_count).sum();

        println!("# claude-dejavu status\n");

        if rules.is_empty() && pattern_count == 0 {
            // First time — no data yet
            println!("This project has no learned rules yet. The current session");
            println!("will be analyzed; subsequent sessions will benefit from");
            println!("detected antipatterns written to CLAUDE.md.\n");
            println!("Rule injection starts after the first `claude-dejavu scan`.\n");
            println!("`/dejavu` is available to review detected patterns.");
            println!("Otherwise learning happens passively as you work.\n");
            println!("Dashboard: http://localhost:7777");
            println!("How it works: `/how-it-works`\n");
            println!("This message disappears once the first rule is applied.");
        } else if active_rules.is_empty() && proposed > 0 {
            // Patterns found but no rules approved yet
            println!(
                "{} patterns detected, {} proposed rules awaiting review.\n",
                pattern_count, proposed
            );
            println!("Run `/dejavu` to review and approve proposed rules.\n");
            println!("Dashboard: http://localhost:7777");
        } else {
            // Active rules exist — show status + rules
            let effectiveness = if active_rules.is_empty() {
                0.0
            } else {
                (effective as f64 / active_rules.len() as f64) * 100.0
            };

            println!(
                "{} rules active | {} patterns detected | {:.0}% effectiveness | {} fires\n",
                active_rules.len(),
                pattern_count,
                effectiveness,
                total_fires,
            );

            if proposed > 0 {
                println!(
                    "{} new proposals awaiting review — run `/dejavu`\n",
                    proposed
                );
            }

            println!("## Active Rules\n");
            for rule in &active_rules {
                println!("- {}", rule.text);
            }

            println!("\nDashboard: http://localhost:7777");
        }
    }

    Ok(())
}
