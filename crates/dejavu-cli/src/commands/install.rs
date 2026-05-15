use anyhow::{Context, Result};
use colored::Colorize;
use serde_json::{Map, Value, json};
use std::path::PathBuf;

fn claude_settings_path() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("could not find home directory"))?;
    Ok(home.join(".claude").join("settings.local.json"))
}

fn dejavu_hooks() -> Value {
    json!({
        "SessionStart": [
            {
                "matcher": "startup|clear|compact",
                "hooks": [
                    {
                        "type": "command",
                        "command": "claude-dejavu inject --format text --path \"$PWD\"",
                        "timeout": 15
                    }
                ]
            }
        ],
        "Stop": [
            {
                "hooks": [
                    {
                        "type": "command",
                        "command": "claude-dejavu scan --auto --path \"$PWD\"",
                        "timeout": 30
                    }
                ]
            }
        ]
    })
}

fn is_dejavu_hook(entry: &Value) -> bool {
    if let Some(hooks_arr) = entry.get("hooks").and_then(|h| h.as_array()) {
        for hook in hooks_arr {
            if let Some(cmd) = hook.get("command").and_then(|c| c.as_str())
                && cmd.contains("claude-dejavu")
            {
                return true;
            }
        }
    }
    false
}

pub fn run() -> Result<()> {
    println!("{}", "Installing claude-dejavu...".bold());

    // Step 1: Initialize DB
    let engine = dejavu_core::DejavuEngine::new()?;
    let _conn = dejavu_core::db::open(&engine.db_path)?;
    println!("  {} Database ready at {:?}", "✓".green(), engine.db_path);

    // Step 2: Register hooks (safe merge — never overwrites non-dejavu hooks)
    register_hooks()?;

    // Step 3: Bootstrap — scan existing sessions for initial rules
    println!("\n{}", "Scanning existing session history...".bold());
    bootstrap(&engine)?;

    println!();
    println!("{}", "claude-dejavu is installed and ready.".green().bold());
    println!(
        "{}",
        "Claude Code will now automatically learn from your mistakes.".dimmed()
    );

    Ok(())
}

fn register_hooks() -> Result<()> {
    let settings_path = claude_settings_path()?;

    if let Some(parent) = settings_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Read existing settings — never overwrite, only merge
    let mut settings: Map<String, Value> = if settings_path.exists() {
        let content = std::fs::read_to_string(&settings_path)
            .context("failed to read settings.local.json")?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Map::new()
    };

    let dejavu = dejavu_hooks();
    let dejavu_hooks_map = dejavu
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("dejavu hooks template is not an object"))?;

    let hooks = settings
        .entry("hooks")
        .or_insert_with(|| Value::Object(Map::new()));

    let hooks_map = hooks
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("hooks field in settings is not an object"))?;

    for (event_name, dejavu_entries) in dejavu_hooks_map {
        let dejavu_arr = dejavu_entries
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("{event_name} dejavu entries is not an array"))?;

        let existing = hooks_map
            .entry(event_name)
            .or_insert_with(|| Value::Array(Vec::new()));

        let existing_arr = existing
            .as_array_mut()
            .ok_or_else(|| anyhow::anyhow!("{event_name} hooks is not an array"))?;

        // Only remove dejavu's own hooks — preserve everything else
        existing_arr.retain(|entry| !is_dejavu_hook(entry));

        for entry in dejavu_arr {
            existing_arr.push(entry.clone());
        }
    }

    let output = serde_json::to_string_pretty(&Value::Object(settings))?;
    std::fs::write(&settings_path, output).context("failed to write settings.local.json")?;

    println!(
        "  {} Hooks registered in {}",
        "✓".green(),
        settings_path.display()
    );

    Ok(())
}

fn bootstrap(engine: &dejavu_core::DejavuEngine) -> Result<()> {
    let results = engine.scan_all()?;

    if results.is_empty() {
        println!(
            "  {} No patterns found yet — they'll appear as you use Claude Code.",
            "ℹ".blue()
        );
        return Ok(());
    }

    let mut total_patterns = 0;
    let mut total_rules = 0;

    for (project, detections) in &results {
        let high_confidence: Vec<_> = detections
            .iter()
            .filter(|d| d.confidence >= engine.config.confidence_threshold)
            .collect();

        if high_confidence.is_empty() {
            continue;
        }

        total_patterns += detections.len();

        println!(
            "\n  📂 {} — {} patterns found",
            project.dimmed(),
            detections.len(),
        );

        for detection in &high_confidence {
            let label = match detection.detector_type {
                dejavu_core::DetectorType::RevertCycle => "revert".yellow(),
                dejavu_core::DetectorType::RepeatedError => "error".red(),
                dejavu_core::DetectorType::SilentFix => "silent".magenta(),
                dejavu_core::DetectorType::UserCorrection => "correction".cyan(),
                dejavu_core::DetectorType::LongBash => "bash".blue(),
            };

            println!(
                "    [{}] {} (confidence: {:.0}%)",
                label,
                detection.suggested_rule,
                detection.confidence * 100.0,
            );
            total_rules += 1;
        }
    }

    if total_rules > 0 {
        println!(
            "\n  {} Found {} patterns across {} projects → {} rules proposed.",
            "✓".green(),
            total_patterns,
            results.len(),
            total_rules,
        );
        println!(
            "  {}",
            "Run `claude-dejavu scan` in a project to apply rules to CLAUDE.md.".dimmed()
        );
    }

    Ok(())
}
