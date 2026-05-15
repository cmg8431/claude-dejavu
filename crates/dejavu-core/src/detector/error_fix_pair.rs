use super::{Detection, DetectorType, Evidence};
use crate::parser::ParsedSession;
use std::collections::HashMap;

/// Detector ⑦: Error → Fix Pairing
///
/// When a Bash error is followed by a successful Edit/Bash that fixes it,
/// record the pair. If the same error type recurs, suggest the known fix.
pub fn detect(sessions: &[ParsedSession]) -> Vec<Detection> {
    let mut fix_pairs: HashMap<String, Vec<FixPair>> = HashMap::new();

    for session in sessions {
        let pairs = extract_error_fix_pairs(session);
        for pair in pairs {
            let key = pair.error_normalized.clone();
            fix_pairs.entry(key).or_default().push(pair);
        }
    }

    let mut detections = Vec::new();

    for (error_key, pairs) in &fix_pairs {
        if pairs.is_empty() {
            continue;
        }

        let session_ids: Vec<String> = pairs
            .iter()
            .map(|p| p.session_id.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        // Find the most common fix action
        let best_fix = find_best_fix(pairs);
        let Some(fix_desc) = best_fix else {
            continue;
        };

        let confidence = calculate_confidence(pairs.len(), session_ids.len());

        let suggested_rule = format!("When `{}` fails: {}", truncate(error_key, 50), fix_desc,);

        detections.push(Detection {
            detector_type: DetectorType::ErrorFixPair,
            evidence: Evidence {
                sessions: session_ids,
                file_paths: vec![],
                occurrences: pairs.len(),
                details: serde_json::json!({
                    "error": error_key,
                    "fix": fix_desc,
                    "times_seen": pairs.len(),
                }),
            },
            confidence,
            suggested_rule,
        });
    }

    detections
}

struct FixPair {
    session_id: String,
    error_normalized: String,
    _error_raw: String,
    _fix_tool: String,
    fix_description: String,
}

fn extract_error_fix_pairs(session: &ParsedSession) -> Vec<FixPair> {
    let mut pairs = Vec::new();

    for (i, tc) in session.tool_calls.iter().enumerate() {
        // Find error results
        let Some(ref result) = tc.result else {
            continue;
        };
        if !result.is_error && !has_error_in_content(&result.content) {
            continue;
        }

        // Skip noise
        if is_noise(&result.content) {
            continue;
        }

        let error_normalized = normalize_error(&result.content);
        if error_normalized.is_empty() {
            continue;
        }

        // Look ahead for the fix (next 1-5 tool calls)
        for j in (i + 1)..std::cmp::min(i + 6, session.tool_calls.len()) {
            let fix_tc = &session.tool_calls[j];

            // Fix is a successful tool call (no error result)
            let is_fix = match &fix_tc.result {
                None => true, // No result = assumed success (Edit/Write)
                Some(r) => !r.is_error && !has_error_in_content(&r.content),
            };

            if !is_fix {
                continue;
            }

            let fix_desc = describe_fix(fix_tc);
            if fix_desc.is_empty() {
                continue;
            }

            pairs.push(FixPair {
                session_id: session.id.clone(),
                error_normalized: error_normalized.clone(),
                _error_raw: result.content.clone(),
                _fix_tool: fix_tc.name.clone(),
                fix_description: fix_desc,
            });

            break; // Only pair with the first fix
        }
    }

    pairs
}

fn describe_fix(tc: &crate::parser::ToolCall) -> String {
    match tc.name.as_str() {
        "Edit" => {
            let file = tc
                .input
                .get("file_path")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            let short_file = file.split('/').next_back().unwrap_or(file);
            format!("edit `{}`.", short_file)
        }
        "Write" => {
            let file = tc
                .input
                .get("file_path")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            let short_file = file.split('/').next_back().unwrap_or(file);
            format!("create/rewrite `{}`.", short_file)
        }
        "Bash" => {
            let cmd = tc
                .input
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            // Shorten long commands
            let short_cmd = if cmd.len() > 60 {
                format!("{}...", &cmd[..57])
            } else {
                cmd.to_string()
            };
            format!("run `{}`.", short_cmd)
        }
        "Read" => {
            let file = tc
                .input
                .get("file_path")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            let short_file = file.split('/').next_back().unwrap_or(file);
            format!("check `{}` first.", short_file)
        }
        _ => String::new(),
    }
}

fn normalize_error(msg: &str) -> String {
    // Extract the meaningful part of the error
    let lines: Vec<&str> = msg.lines().collect();

    // Find the first actual error line
    let re = regex::Regex::new(r"(/[\w./\-]+)").unwrap();
    for line in &lines {
        let line = line.trim();
        if line.is_empty() || line.starts_with("Exit code") {
            continue;
        }
        if line.contains("error:") || line.contains("Error:") || line.contains("FAILED") {
            let normalized = re.replace_all(line, "<path>");
            return normalized.trim().to_string();
        }
    }

    // Fallback: first non-empty line after "Exit code"
    let meaningful = lines
        .iter()
        .find(|l| !l.trim().is_empty() && !l.starts_with("Exit code"));

    if let Some(line) = meaningful {
        return re.replace_all(line.trim(), "<path>").to_string();
    }

    String::new()
}

fn has_error_in_content(content: &str) -> bool {
    content.contains("error:") || content.contains("Error:") || content.contains("FAILED")
}

fn is_noise(content: &str) -> bool {
    let lower = content.to_lowercase();
    lower.contains("tool_use_error")
        || lower.contains("cancelled:")
        || lower.contains("the user doesn't want")
        || lower.contains("tool use was rejected")
}

fn find_best_fix(pairs: &[FixPair]) -> Option<String> {
    if pairs.is_empty() {
        return None;
    }

    // Count fix descriptions
    let mut fix_counts: HashMap<String, usize> = HashMap::new();
    for pair in pairs {
        *fix_counts.entry(pair.fix_description.clone()).or_default() += 1;
    }

    fix_counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(desc, _)| desc)
}

fn calculate_confidence(total: usize, unique_sessions: usize) -> f64 {
    let base = if unique_sessions >= 2 { 0.8 } else { 0.6 };
    let freq_boost = ((total as f64).ln() / 4.0).min(0.2);
    (base + freq_boost).min(1.0)
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        // Find char boundary
        let mut end = max;
        while end > 0 && !s.is_char_boundary(end) {
            end -= 1;
        }
        &s[..end]
    }
}
