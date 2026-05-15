use anyhow::Result;
use std::process::Command;

/// Analyze a batch of edit diffs using Claude AI to extract specific rules.
/// Uses `claude -p` (print mode) with Haiku for cost efficiency.
pub fn analyze_diffs(file_path: &str, diffs: &[(String, String)]) -> Result<Option<String>> {
    if diffs.is_empty() {
        return Ok(None);
    }

    let sample_diffs: Vec<String> = diffs
        .iter()
        .take(10)
        .map(|(old, new)| format!("OLD: {}\nNEW: {}", truncate(old, 100), truncate(new, 100)))
        .collect();

    let diff_text = sample_diffs.join("\n---\n");

    let prompt = format!(
        r#"Analyze these code edit diffs from file `{}`. These are corrections a user made after an AI edited this file.

{}

What specific patterns do you see? Generate 1-3 concise, actionable rules for the AI to follow when editing this file.

Respond ONLY with a JSON array of strings. No markdown, no code fences. Example:
["Use borderRadius: 8 not 12", "fontSize should be 18px for titles"]

If no clear pattern, respond: []"#,
        shorten_path(file_path),
        diff_text
    );

    let output = Command::new("claude")
        .args(["-p", "--model", "haiku", "--output-format", "json"])
        .arg(&prompt)
        .output();

    let output = match output {
        Ok(o) => o,
        Err(_) => return Ok(None),
    };

    if !output.status.success() {
        return Ok(None);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stdout = stdout.trim();

    // Parse response — claude --output-format json wraps in {"result": "..."}
    let raw_result = if let Ok(obj) = serde_json::from_str::<serde_json::Value>(stdout) {
        // Extract "result" field (claude CLI json format)
        if let Some(result_str) = obj.get("result").and_then(|r| r.as_str()) {
            result_str.to_string()
        } else {
            stdout.to_string()
        }
    } else {
        stdout.to_string()
    };

    // Strip markdown code fences if present
    let clean = raw_result
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    // Parse as JSON array of strings
    let rules: Vec<String> = match serde_json::from_str::<Vec<String>>(clean) {
        Ok(arr) => arr,
        Err(_) => return Ok(None),
    };

    if rules.is_empty() {
        return Ok(None);
    }

    let short_path = shorten_path(file_path);
    let rule_text = format!("`{}`: {}", short_path, rules.join(". "));

    Ok(Some(rule_text))
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

fn shorten_path(path: &str) -> String {
    for prefix in ["/src/", "/app/", "/pages/", "/components/"] {
        if let Some(pos) = path.find(prefix) {
            return path[pos + 1..].to_string();
        }
    }
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() > 3 {
        parts[parts.len() - 3..].join("/")
    } else {
        path.to_string()
    }
}
