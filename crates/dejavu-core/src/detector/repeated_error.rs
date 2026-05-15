use super::{Detection, DetectorType, Evidence};
use crate::parser::ParsedSession;
use std::collections::HashMap;

/// Detector ②: Repeated Error → Fix
///
/// Signal: Same error message (or similar) appears N times across sessions,
/// always followed by the same fix pattern.
///
/// Detection: Regex-cluster Bash tool error lines, count occurrences.
pub fn detect(sessions: &[ParsedSession]) -> Vec<Detection> {
    let mut error_clusters: HashMap<String, Vec<ErrorInstance>> = HashMap::new();

    for session in sessions {
        for error in &session.errors {
            let normalized = normalize_error(&error.message);
            error_clusters
                .entry(normalized)
                .or_default()
                .push(ErrorInstance {
                    session_id: session.id.clone(),
                    raw_message: error.message.clone(),
                    tool_name: error.tool_name.clone(),
                });
        }
    }

    let mut detections = Vec::new();

    for (normalized_error, instances) in &error_clusters {
        if instances.len() < 2 {
            continue;
        }

        let session_ids: Vec<String> = instances
            .iter()
            .map(|i| i.session_id.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        if session_ids.len() < 2 {
            continue;
        }

        let confidence = calculate_confidence(instances.len(), session_ids.len());

        let suggested_rule = format!(
            "When encountering `{}`, check the common fix pattern observed across {} sessions.",
            truncate(normalized_error, 80),
            session_ids.len(),
        );

        detections.push(Detection {
            detector_type: DetectorType::RepeatedError,
            evidence: Evidence {
                sessions: session_ids,
                file_paths: vec![],
                occurrences: instances.len(),
                details: serde_json::json!({
                    "normalized_error": normalized_error,
                    "sample_raw": instances.first().map(|i| &i.raw_message),
                    "tool": instances.first().map(|i| &i.tool_name),
                }),
            },
            confidence,
            suggested_rule,
        });
    }

    detections
}

struct ErrorInstance {
    session_id: String,
    raw_message: String,
    tool_name: String,
}

fn normalize_error(msg: &str) -> String {
    let msg = msg.trim();

    // Strip file paths, line numbers, timestamps
    let re_path = regex::Regex::new(r"(/[\w./\-]+)").unwrap();
    let re_line = regex::Regex::new(r":\d+:\d+").unwrap();
    let re_hex = regex::Regex::new(r"0x[0-9a-fA-F]+").unwrap();

    let normalized = re_path.replace_all(msg, "<PATH>");
    let normalized = re_line.replace_all(&normalized, ":<LINE>");
    let normalized = re_hex.replace_all(&normalized, "<HEX>");

    normalized.to_lowercase()
}

fn calculate_confidence(total_occurrences: usize, unique_sessions: usize) -> f64 {
    let base = (unique_sessions as f64 / 2.0).min(1.0);
    let frequency_boost = ((total_occurrences as f64).ln() / 3.0).min(0.3);
    (base + frequency_boost).min(1.0)
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        let mut end = max;
        while end > 0 && !s.is_char_boundary(end) {
            end -= 1;
        }
        &s[..end]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{ErrorEvent, ParsedSession};

    fn make_session(id: &str, errors: Vec<ErrorEvent>) -> ParsedSession {
        ParsedSession {
            id: id.to_string(),
            project_path: "/test/project".to_string(),
            messages: vec![],
            tool_calls: vec![],
            errors,
            file_edits: vec![],
        }
    }

    fn make_error(msg: &str) -> ErrorEvent {
        ErrorEvent {
            message: msg.to_string(),
            tool_name: "Bash".to_string(),
            index: 0,
        }
    }

    #[test]
    fn two_sessions_same_error_detected() {
        let sessions = vec![
            make_session("s1", vec![make_error("error: cannot find module 'foo'")]),
            make_session("s2", vec![make_error("error: cannot find module 'foo'")]),
        ];
        let detections = detect(&sessions);
        assert!(!detections.is_empty(), "should detect repeated error");
        assert!(detections[0].confidence > 0.0);
        assert_eq!(detections[0].evidence.occurrences, 2);
    }

    #[test]
    fn single_session_error_not_detected() {
        let sessions = vec![make_session(
            "s1",
            vec![make_error("error: something went wrong")],
        )];
        let detections = detect(&sessions);
        assert!(
            detections.is_empty(),
            "single session error should not trigger detection"
        );
    }

    #[test]
    fn same_session_duplicate_errors_not_detected() {
        // Two errors in the SAME session should not trigger (needs 2+ unique sessions)
        let sessions = vec![make_session(
            "s1",
            vec![
                make_error("error: not found"),
                make_error("error: not found"),
            ],
        )];
        let detections = detect(&sessions);
        assert!(
            detections.is_empty(),
            "duplicate errors in same session should not trigger detection"
        );
    }

    #[test]
    fn error_normalization_strips_paths_and_line_numbers() {
        let normalized = normalize_error("error at /home/user/project/src/main.rs:42:10");
        assert!(
            !normalized.contains("/home/user"),
            "paths should be stripped"
        );
        assert!(normalized.contains("<path>"), "paths replaced with <PATH>");
        assert!(
            !normalized.contains(":42:10"),
            "line numbers should be stripped"
        );
    }

    #[test]
    fn error_normalization_strips_hex_addresses() {
        let normalized = normalize_error("segfault at 0xDEADBEEF");
        assert!(
            !normalized.contains("0xDEADBEEF"),
            "hex addresses should be stripped"
        );
        assert!(normalized.contains("<hex>"));
    }

    #[test]
    fn errors_with_different_paths_cluster_together() {
        let sessions = vec![
            make_session(
                "s1",
                vec![make_error(
                    "error: file not found /home/alice/project/foo.rs",
                )],
            ),
            make_session(
                "s2",
                vec![make_error("error: file not found /home/bob/project/foo.rs")],
            ),
        ];
        let detections = detect(&sessions);
        assert!(
            !detections.is_empty(),
            "errors differing only in path should cluster"
        );
    }
}
