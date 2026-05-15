pub mod error_fix_pair;
pub mod long_bash;
pub mod project_context;
pub mod repeated_error;
pub mod revert_cycle;
pub mod silent_fix;
pub mod smart_analyzer;
pub mod user_correction;

use crate::parser::ParsedSession;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Detection {
    pub detector_type: DetectorType,
    pub evidence: Evidence,
    pub confidence: f64,
    pub suggested_rule: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetectorType {
    RevertCycle,
    RepeatedError,
    SilentFix,
    UserCorrection,
    LongBash,
    ProjectContext,
    ErrorFixPair,
}

impl DetectorType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RevertCycle => "revert_cycle",
            Self::RepeatedError => "repeated_error",
            Self::SilentFix => "silent_fix",
            Self::UserCorrection => "user_correction",
            Self::LongBash => "long_bash",
            Self::ProjectContext => "project_context",
            Self::ErrorFixPair => "error_fix_pair",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub sessions: Vec<String>,
    pub file_paths: Vec<String>,
    pub occurrences: usize,
    pub details: serde_json::Value,
}

/// Run session-based detectors (need parsed session data).
pub fn run_all_detectors(sessions: &[ParsedSession]) -> Vec<Detection> {
    let mut detections = Vec::new();

    detections.extend(error_fix_pair::detect(sessions));
    detections.extend(user_correction::detect(sessions));
    detections.extend(repeated_error::detect(sessions));
    detections.extend(revert_cycle::detect(sessions));
    detections.extend(silent_fix::detect(sessions));
    detections.extend(long_bash::detect(sessions));

    detections.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    detections
}
