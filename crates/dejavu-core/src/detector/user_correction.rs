use super::{Detection, DetectorType, Evidence};
use crate::parser::ParsedSession;
use regex::Regex;
use std::collections::HashMap;

/// Detector ④: User Correction (from claude-reflect patterns)
///
/// Signal: User explicitly corrects Claude's behavior via text patterns.
/// "no, use X not Y", "don't use X", "actually, ...", "use X instead"
///
/// Multi-language: English, Korean, Japanese
pub fn detect(sessions: &[ParsedSession]) -> Vec<Detection> {
    let patterns = CorrectionPatterns::new();
    let mut corrections: HashMap<String, Vec<CorrectionInstance>> = HashMap::new();

    for session in sessions {
        for msg in &session.messages {
            let Some(ref role) = msg.role else { continue };
            if role != "user" {
                continue;
            }

            let text = extract_text_content(msg);
            if text.is_empty() {
                continue;
            }

            if let Some(correction) = patterns.detect_correction(&text) {
                let key = correction.normalized.clone();
                corrections
                    .entry(key)
                    .or_default()
                    .push(CorrectionInstance {
                        session_id: session.id.clone(),
                        raw_text: text.clone(),
                        confidence: correction.confidence,
                        correction_type: correction.correction_type,
                    });
            }
        }
    }

    let mut detections = Vec::new();

    for (normalized, instances) in &corrections {
        let session_ids: Vec<String> = instances
            .iter()
            .map(|i| i.session_id.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let max_confidence = instances
            .iter()
            .map(|i| i.confidence)
            .fold(0.0_f64, f64::max);

        // Boost confidence if same correction appears across sessions
        let cross_session_boost = if session_ids.len() > 1 {
            0.1 * (session_ids.len() as f64 - 1.0).min(0.3)
        } else {
            0.0
        };

        let confidence = (max_confidence + cross_session_boost).min(1.0);

        let correction_type = instances
            .first()
            .map(|i| i.correction_type)
            .unwrap_or(CorrectionType::Explicit);

        let suggested_rule = generate_rule_text(normalized, instances, correction_type);

        detections.push(Detection {
            detector_type: DetectorType::UserCorrection,
            evidence: Evidence {
                sessions: session_ids,
                file_paths: vec![],
                occurrences: instances.len(),
                details: serde_json::json!({
                    "normalized": normalized,
                    "correction_type": format!("{:?}", correction_type),
                    "sample_raw": instances.first().map(|i| &i.raw_text),
                }),
            },
            confidence,
            suggested_rule,
        });
    }

    detections
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
enum CorrectionType {
    Explicit,   // "remember: ...", "rule: ..."
    Negative,   // "don't use X", "stop doing X"
    Redirect,   // "use X instead of Y", "no, use X not Y"
    Preference, // "actually, ...", "I prefer ..."
    Approval,   // "perfect!", "that's great" (positive reinforcement)
}

struct CorrectionInstance {
    session_id: String,
    raw_text: String,
    confidence: f64,
    correction_type: CorrectionType,
}

struct CorrectionResult {
    normalized: String,
    confidence: f64,
    correction_type: CorrectionType,
}

struct CorrectionPatterns {
    explicit: Vec<Regex>,
    negative: Vec<Regex>,
    redirect: Vec<Regex>,
    preference: Vec<Regex>,
}

impl CorrectionPatterns {
    fn new() -> Self {
        Self {
            explicit: vec![
                // English
                Regex::new(r"(?i)remember:\s+(.+)").unwrap(),
                Regex::new(r"(?i)rule:\s+(.+)").unwrap(),
                Regex::new(r"(?i)always\s+(?:do|use|check|run)\s+(.+)").unwrap(),
                Regex::new(r"(?i)never\s+(?:do|use|run|delete)\s+(.+)").unwrap(),
                // Korean
                Regex::new(r"(?i)기억해[:\s]+(.+)").unwrap(),
                Regex::new(r"(?i)항상\s+(.+)(?:해|하세요|해야)").unwrap(),
                Regex::new(r"(?i)절대\s+(.+)\s*(?:하지|하면)\s*(?:마|안)").unwrap(),
                // Japanese
                Regex::new(r"覚えて[:\s]+(.+)").unwrap(),
                Regex::new(r"必ず\s*(.+)(?:して|する|すること)").unwrap(),
            ],
            negative: vec![
                Regex::new(r"(?i)don'?t\s+use\s+(.+)").unwrap(),
                Regex::new(r"(?i)stop\s+using\s+(.+)").unwrap(),
                Regex::new(r"(?i)stop\s+doing\s+(.+)").unwrap(),
                Regex::new(r"(?i)don'?t\s+do\s+(.+)").unwrap(),
                // Korean
                Regex::new(r"(?i)(.+)\s*쓰지\s*마").unwrap(),
                Regex::new(r"(?i)(.+)\s*하지\s*마").unwrap(),
                Regex::new(r"(?i)(.+)\s*사용하지\s*마").unwrap(),
            ],
            redirect: vec![
                Regex::new(r"(?i)no,?\s+use\s+(.+?)\s+not\s+(.+)").unwrap(),
                Regex::new(r"(?i)use\s+(.+?)\s+instead\s+of\s+(.+)").unwrap(),
                Regex::new(r"(?i)use\s+(.+?)\s+instead").unwrap(),
                Regex::new(r"(?i)instead\s+of\s+(.+?),?\s+(?:you\s+should|use|do)\s+(.+)").unwrap(),
                // Korean
                Regex::new(r"(?i)(.+)\s*말고\s+(.+)\s*(?:써|사용|쓰세요)").unwrap(),
                Regex::new(r"(?i)(.+)\s*대신\s+(.+)\s*(?:써|사용|쓰세요)").unwrap(),
            ],
            preference: vec![
                Regex::new(r"(?i)actually,?\s+(.+)").unwrap(),
                Regex::new(r"(?i)i\s+prefer\s+(.+)").unwrap(),
                Regex::new(r"(?i)i\s+want\s+(.+)").unwrap(),
                // Korean
                Regex::new(r"(?i)사실은?\s+(.+)").unwrap(),
                Regex::new(r"(?i)(.+)(?:이|가)\s+(?:나아|좋아|낫)").unwrap(),
            ],
        }
    }

    fn detect_correction(&self, text: &str) -> Option<CorrectionResult> {
        // Check in order of confidence: explicit > negative > redirect > preference

        for re in &self.explicit {
            if let Some(caps) = re.captures(text) {
                let matched = caps.get(1).map(|m| m.as_str()).unwrap_or(text);
                return Some(CorrectionResult {
                    normalized: normalize_correction(matched),
                    confidence: 0.90,
                    correction_type: CorrectionType::Explicit,
                });
            }
        }

        for re in &self.redirect {
            if let Some(caps) = re.captures(text) {
                let matched = caps.get(0).map(|m| m.as_str()).unwrap_or(text);
                return Some(CorrectionResult {
                    normalized: normalize_correction(matched),
                    confidence: 0.80,
                    correction_type: CorrectionType::Redirect,
                });
            }
        }

        for re in &self.negative {
            if let Some(caps) = re.captures(text) {
                let matched = caps.get(0).map(|m| m.as_str()).unwrap_or(text);
                return Some(CorrectionResult {
                    normalized: normalize_correction(matched),
                    confidence: 0.75,
                    correction_type: CorrectionType::Negative,
                });
            }
        }

        for re in &self.preference {
            if let Some(caps) = re.captures(text) {
                let matched = caps.get(1).map(|m| m.as_str()).unwrap_or(text);
                return Some(CorrectionResult {
                    normalized: normalize_correction(matched),
                    confidence: 0.60,
                    correction_type: CorrectionType::Preference,
                });
            }
        }

        None
    }
}

fn extract_text_content(msg: &crate::parser::SessionMessage) -> String {
    match &msg.content {
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(serde_json::Value::Array(arr)) => arr
            .iter()
            .filter_map(|v| {
                if let Some(obj) = v.as_object()
                    && obj.get("type").and_then(|t| t.as_str()) == Some("text")
                {
                    return obj.get("text").and_then(|t| t.as_str()).map(String::from);
                }
                None
            })
            .collect::<Vec<_>>()
            .join(" "),
        _ => String::new(),
    }
}

fn normalize_correction(text: &str) -> String {
    text.trim()
        .trim_end_matches(&['.', '!', '?', ','][..])
        .to_lowercase()
}

fn generate_rule_text(
    normalized: &str,
    instances: &[CorrectionInstance],
    correction_type: CorrectionType,
) -> String {
    let raw = instances
        .first()
        .map(|i| i.raw_text.as_str())
        .unwrap_or(normalized);

    // Clean up the raw text for use as a rule
    let cleaned = raw
        .trim()
        .trim_start_matches(|c: char| !c.is_alphanumeric() && c != '(' && c != '"');

    match correction_type {
        CorrectionType::Explicit => {
            // Already explicit — use as-is
            format!(
                "{} (User explicitly stated this {} time(s).)",
                capitalize(cleaned),
                instances.len()
            )
        }
        CorrectionType::Negative | CorrectionType::Redirect => {
            format!(
                "{} (Corrected {} time(s) across {} session(s).)",
                capitalize(cleaned),
                instances.len(),
                instances
                    .iter()
                    .map(|i| &i.session_id)
                    .collect::<std::collections::HashSet<_>>()
                    .len()
            )
        }
        CorrectionType::Preference => {
            format!(
                "User preference: {} (Stated {} time(s).)",
                cleaned,
                instances.len()
            )
        }
        CorrectionType::Approval => {
            format!(
                "Keep doing: {} (Approved {} time(s).)",
                cleaned,
                instances.len()
            )
        }
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}
