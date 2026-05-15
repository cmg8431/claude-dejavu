use super::{Detection, DetectorType, Evidence};
use crate::parser::ParsedSession;
use std::collections::HashMap;

/// Detector ⑤: Long Bash Session (stuck pattern)
///
/// Signal: A task has N× more Bash calls than the session average.
/// Claude is stuck in a debugging loop. When it finally escapes,
/// the solution is worth recording.
///
/// Detection: Count Bash tool calls per session. If a session has
/// significantly more Bash calls than the average, flag it.
pub fn detect(sessions: &[ParsedSession]) -> Vec<Detection> {
    if sessions.len() < 3 {
        return vec![];
    }

    let mut session_bash_counts: Vec<(String, usize)> = Vec::new();

    for session in sessions {
        let bash_count = session
            .tool_calls
            .iter()
            .filter(|tc| tc.name == "Bash")
            .count();
        session_bash_counts.push((session.id.clone(), bash_count));
    }

    let total: usize = session_bash_counts.iter().map(|(_, c)| c).sum();
    let avg = total as f64 / session_bash_counts.len() as f64;

    if avg < 3.0 {
        return vec![];
    }

    let threshold = avg * 2.5; // 2.5x the average

    let mut detections = Vec::new();

    // Find error patterns in high-bash sessions
    let mut error_clusters: HashMap<String, Vec<String>> = HashMap::new();

    for session in sessions {
        let bash_count = session
            .tool_calls
            .iter()
            .filter(|tc| tc.name == "Bash")
            .count();

        if (bash_count as f64) < threshold {
            continue;
        }

        // This was a "stuck" session — extract the errors that kept repeating
        for error in &session.errors {
            let key = normalize_error_key(&error.message);
            error_clusters
                .entry(key)
                .or_default()
                .push(session.id.clone());
        }

        // If no specific error but high bash count, record the pattern
        if session.errors.is_empty() {
            let confidence = ((bash_count as f64 / avg) * 0.2).min(0.8);

            detections.push(Detection {
                detector_type: DetectorType::LongBash,
                evidence: Evidence {
                    sessions: vec![session.id.clone()],
                    file_paths: vec![],
                    occurrences: bash_count,
                    details: serde_json::json!({
                        "bash_count": bash_count,
                        "session_average": avg.round(),
                        "ratio": format!("{:.1}x", bash_count as f64 / avg),
                    }),
                },
                confidence,
                suggested_rule: format!(
                    "Session had {:.1}x more Bash calls than average ({} vs {:.0}). Consider checking approach before extensive debugging.",
                    bash_count as f64 / avg,
                    bash_count,
                    avg,
                ),
            });
        }
    }

    // Cluster repeated errors from stuck sessions
    for (error_key, session_ids) in &error_clusters {
        let unique_sessions: Vec<String> = session_ids
            .iter()
            .cloned()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        if unique_sessions.len() < 2 {
            continue;
        }

        let confidence = (unique_sessions.len() as f64 * 0.3).min(0.9);

        detections.push(Detection {
            detector_type: DetectorType::LongBash,
            evidence: Evidence {
                sessions: unique_sessions.clone(),
                file_paths: vec![],
                occurrences: session_ids.len(),
                details: serde_json::json!({
                    "error_pattern": error_key,
                    "stuck_sessions": unique_sessions.len(),
                }),
            },
            confidence,
            suggested_rule: format!(
                "Recurring debugging pattern: `{}`. Check common fix before retrying. (Caused stuck sessions {} times.)",
                truncate(error_key, 60),
                unique_sessions.len(),
            ),
        });
    }

    detections
}

fn normalize_error_key(msg: &str) -> String {
    let msg = msg.trim();
    // Strip paths and line numbers for clustering
    let re_path = regex::Regex::new(r"(/[\w./\-]+)").unwrap();
    let re_line = regex::Regex::new(r":\d+:\d+").unwrap();
    let normalized = re_path.replace_all(msg, "<PATH>");
    let normalized = re_line.replace_all(&normalized, "");
    normalized.to_lowercase()
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max { s } else { &s[..max] }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{ParsedSession, ToolCall, ToolResult};

    fn make_session_with_bash_calls(id: &str, bash_count: usize) -> ParsedSession {
        let tool_calls = (0..bash_count)
            .map(|i| ToolCall {
                name: "Bash".to_string(),
                input: serde_json::json!({"command": "echo test"}),
                result: Some(ToolResult {
                    content: "test".to_string(),
                    is_error: false,
                }),
                index: i,
                tool_use_id: format!("tool_{}", i),
            })
            .collect();

        ParsedSession {
            id: id.to_string(),
            project_path: "/test/project".to_string(),
            messages: vec![],
            tool_calls,
            errors: vec![],
            file_edits: vec![],
        }
    }

    #[test]
    fn session_with_3x_average_bash_calls_detected() {
        // 4 sessions: 3 normal (10 calls each), 1 outlier (80 calls)
        // avg = (10+10+10+80)/4 = 27.5, threshold = 27.5 * 2.5 = 68.75
        // The 80-call session exceeds the threshold
        let sessions = vec![
            make_session_with_bash_calls("s1", 10),
            make_session_with_bash_calls("s2", 10),
            make_session_with_bash_calls("s3", 10),
            make_session_with_bash_calls("s_outlier", 80),
        ];
        let detections = detect(&sessions);
        assert!(
            !detections.is_empty(),
            "session with 3x average bash calls should be detected"
        );
        assert!(
            detections
                .iter()
                .any(|d| d.evidence.sessions.contains(&"s_outlier".to_string()))
        );
    }

    #[test]
    fn normal_sessions_no_detection() {
        let sessions = vec![
            make_session_with_bash_calls("s1", 10),
            make_session_with_bash_calls("s2", 12),
            make_session_with_bash_calls("s3", 8),
            make_session_with_bash_calls("s4", 11),
        ];
        let detections = detect(&sessions);
        assert!(
            detections.is_empty(),
            "normal sessions should not trigger detection"
        );
    }

    #[test]
    fn fewer_than_3_sessions_no_detection() {
        let sessions = vec![
            make_session_with_bash_calls("s1", 100),
            make_session_with_bash_calls("s2", 5),
        ];
        let detections = detect(&sessions);
        assert!(
            detections.is_empty(),
            "fewer than 3 sessions should not trigger detection"
        );
    }

    #[test]
    fn very_low_average_no_detection() {
        // avg < 3.0 should return early
        let sessions = vec![
            make_session_with_bash_calls("s1", 1),
            make_session_with_bash_calls("s2", 2),
            make_session_with_bash_calls("s3", 1),
        ];
        let detections = detect(&sessions);
        assert!(
            detections.is_empty(),
            "very low bash average should not trigger detection"
        );
    }
}
