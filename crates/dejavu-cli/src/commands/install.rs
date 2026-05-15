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

/// Check if a hook entry was installed by dejavu (by looking at the command field).
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

    // Step 1: Initialize the database (same as init)
    let engine = dejavu_core::DejavuEngine::new()?;
    let _conn = dejavu_core::db::open(&engine.db_path)?;
    println!("  {} Database ready at {:?}", "✓".green(), engine.db_path);

    // Step 2: Register hooks in ~/.claude/settings.local.json
    let settings_path = claude_settings_path()?;

    // Ensure parent directory exists
    if let Some(parent) = settings_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Read existing settings or start with empty object
    let mut settings: Map<String, Value> = if settings_path.exists() {
        let content = std::fs::read_to_string(&settings_path)
            .context("failed to read settings.local.json")?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Map::new()
    };

    // Merge hooks
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

        // Remove any existing dejavu hooks to avoid duplicates
        existing_arr.retain(|entry| !is_dejavu_hook(entry));

        // Append dejavu hooks
        for entry in dejavu_arr {
            existing_arr.push(entry.clone());
        }
    }

    // Write back
    let output = serde_json::to_string_pretty(&Value::Object(settings))?;
    std::fs::write(&settings_path, output).context("failed to write settings.local.json")?;

    println!(
        "  {} Hooks registered in {}",
        "✓".green(),
        settings_path.display()
    );

    println!();
    println!("{}", "claude-dejavu is installed and ready.".green().bold());
    println!(
        "{}",
        "Claude Code will now automatically learn from your mistakes.".dimmed()
    );

    Ok(())
}
