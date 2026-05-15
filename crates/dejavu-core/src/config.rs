use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for claude-dejavu, loaded from `~/.config/claude-dejavu/config.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DejavuConfig {
    /// Minimum number of edits to consider a revert cycle.
    pub revert_cycle_min_edits: usize,

    /// Minimum number of sessions to flag a repeated error.
    pub repeated_error_min_sessions: usize,

    /// Multiplier over median duration to flag a long bash command.
    pub long_bash_threshold_multiplier: f64,

    /// Minimum confidence to promote a detection into a rule.
    pub confidence_threshold: f64,

    /// Days without a fire before a rule is considered dead.
    pub dead_rule_days: i64,

    /// Port for the web dashboard.
    pub dashboard_port: u16,

    /// Paths to exclude from scanning.
    pub excluded_paths: Vec<String>,
}

impl Default for DejavuConfig {
    fn default() -> Self {
        Self {
            revert_cycle_min_edits: 3,
            repeated_error_min_sessions: 2,
            long_bash_threshold_multiplier: 2.5,
            confidence_threshold: 0.5,
            dead_rule_days: 14,
            dashboard_port: 7777,
            excluded_paths: Vec::new(),
        }
    }
}

impl DejavuConfig {
    /// Returns the path to the config file: `~/.config/claude-dejavu/config.toml`.
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("could not find config directory"))?
            .join("claude-dejavu");
        Ok(config_dir.join("config.toml"))
    }

    /// Load configuration from disk, falling back to defaults if the file does not exist.
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let config: DejavuConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }
}
