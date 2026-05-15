use anyhow::{Context, Result};
use colored::Colorize;
use serde_json::{Map, Value};
use std::path::PathBuf;

fn claude_settings_path() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("could not find home directory"))?;
    Ok(home.join(".claude").join("settings.local.json"))
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
    println!("{}", "Uninstalling claude-dejavu hooks...".bold());

    let settings_path = claude_settings_path()?;

    if !settings_path.exists() {
        println!(
            "  {} No settings.local.json found, nothing to remove.",
            "ℹ".blue()
        );
        return Ok(());
    }

    let content =
        std::fs::read_to_string(&settings_path).context("failed to read settings.local.json")?;

    let mut settings: Map<String, Value> =
        serde_json::from_str(&content).context("failed to parse settings.local.json")?;

    let mut removed = 0;

    if let Some(hooks) = settings.get_mut("hooks")
        && let Some(hooks_map) = hooks.as_object_mut()
    {
        let event_names: Vec<String> = hooks_map.keys().cloned().collect();
        for event_name in event_names {
            if let Some(entries) = hooks_map.get_mut(&event_name)
                && let Some(arr) = entries.as_array_mut()
            {
                let before = arr.len();
                arr.retain(|entry| !is_dejavu_hook(entry));
                removed += before - arr.len();

                // Clean up empty arrays
                if arr.is_empty() {
                    hooks_map.remove(&event_name);
                }
            }
        }

        // Clean up empty hooks object
        if hooks_map.is_empty() {
            settings.remove("hooks");
        }
    }

    // Write back
    let output = serde_json::to_string_pretty(&Value::Object(settings))?;
    std::fs::write(&settings_path, output).context("failed to write settings.local.json")?;

    if removed > 0 {
        println!(
            "  {} Removed {removed} dejavu hook(s) from {}",
            "✓".green(),
            settings_path.display()
        );
    } else {
        println!(
            "  {} No dejavu hooks found in {}",
            "ℹ".blue(),
            settings_path.display()
        );
    }

    println!();
    println!(
        "{}",
        "claude-dejavu hooks have been removed.".green().bold()
    );
    println!(
        "{}",
        "The database and learned rules are preserved. Re-run `claude-dejavu install` to re-enable."
            .dimmed()
    );

    Ok(())
}
