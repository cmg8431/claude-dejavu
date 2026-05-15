use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A single line from a Claude Code session .jsonl file.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct RawEntry {
    #[serde(rename = "type")]
    entry_type: Option<String>,
    uuid: Option<String>,
    parent_uuid: Option<String>,
    timestamp: Option<String>,
    session_id: Option<String>,
    cwd: Option<String>,
    message: Option<RawMessage>,
    /// For tool result entries
    tool_use_result: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RawMessage {
    role: Option<String>,
    content: Option<serde_json::Value>,
    model: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SessionMessage {
    pub entry_type: String,
    pub role: Option<String>,
    pub content: Option<serde_json::Value>,
    pub timestamp: Option<String>,
    pub uuid: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ParsedSession {
    pub id: String,
    pub project_path: String,
    pub messages: Vec<SessionMessage>,
    pub tool_calls: Vec<ToolCall>,
    pub errors: Vec<ErrorEvent>,
    pub file_edits: Vec<FileEdit>,
}

#[derive(Debug, Clone)]
pub struct ToolCall {
    pub name: String,
    pub input: serde_json::Value,
    pub result: Option<ToolResult>,
    pub index: usize,
    pub tool_use_id: String,
}

#[derive(Debug, Clone)]
pub struct ToolResult {
    pub content: String,
    pub is_error: bool,
}

#[derive(Debug, Clone)]
pub struct ErrorEvent {
    pub message: String,
    pub tool_name: String,
    pub index: usize,
}

#[derive(Debug, Clone)]
pub struct FileEdit {
    pub file_path: String,
    pub old_content: Option<String>,
    pub new_content: Option<String>,
    pub tool_name: String,
    pub index: usize,
}

/// Check if a file path matches any excluded pattern.
///
/// Supported glob forms:
/// - `**/segment/**` — matches if `segment` appears as a path component
/// - `*.ext` — matches if the path ends with `.ext`
/// - Exact string — matches if the path contains the pattern literally
pub fn should_skip_file(path: &str, excluded: &[String]) -> bool {
    for pattern in excluded {
        if pattern.starts_with("**/") && pattern.ends_with("/**") {
            // e.g. "**/node_modules/**" → check if the segment appears in the path
            let segment = &pattern[3..pattern.len() - 3];
            if path.contains(&format!("/{segment}/")) || path.contains(&format!("{segment}/")) {
                return true;
            }
        } else if pattern.starts_with("*.") {
            // e.g. "*.env" → check extension
            let suffix = &pattern[1..]; // ".env"
            if path.ends_with(suffix) {
                return true;
            }
        } else if path.contains(pattern.as_str()) {
            return true;
        }
    }
    false
}

/// Returns true if user message content contains a `<private>` tag.
fn content_has_private_tag(content: &Option<serde_json::Value>) -> bool {
    match content {
        Some(serde_json::Value::String(s)) => s.contains("<private>"),
        Some(serde_json::Value::Array(arr)) => arr.iter().any(|item| {
            item.get("text")
                .and_then(|t| t.as_str())
                .is_some_and(|s| s.contains("<private>"))
        }),
        _ => false,
    }
}

pub fn find_session_files(claude_dir: &Path) -> Result<Vec<PathBuf>> {
    let projects_dir = claude_dir.join("projects");
    if !projects_dir.exists() {
        return Ok(vec![]);
    }

    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(&projects_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "jsonl") {
            files.push(path.to_path_buf());
        }
    }

    files.sort_by(|a, b| {
        b.metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
            .cmp(
                &a.metadata()
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH),
            )
    });

    Ok(files)
}

pub fn parse_session(path: &Path) -> Result<ParsedSession> {
    parse_session_with_excludes(path, &[])
}

pub fn parse_session_with_excludes(path: &Path, excluded: &[String]) -> Result<ParsedSession> {
    let content = std::fs::read_to_string(path)?;

    let session_id = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let project_path = extract_project_path(path);

    let mut messages = Vec::new();
    let mut tool_calls = Vec::new();
    let mut errors = Vec::new();
    let mut file_edits = Vec::new();

    // Map tool_use_id → index in tool_calls for linking results
    let mut tool_use_map: HashMap<String, usize> = HashMap::new();

    for (idx, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let entry: RawEntry = match serde_json::from_str(line) {
            Ok(e) => e,
            Err(_) => continue,
        };

        let entry_type = entry.entry_type.as_deref().unwrap_or("");

        match entry_type {
            "assistant" => {
                let msg = entry.message.as_ref();
                let content_val = msg.and_then(|m| m.content.clone());

                messages.push(SessionMessage {
                    entry_type: "assistant".to_string(),
                    role: Some("assistant".to_string()),
                    content: content_val.clone(),
                    timestamp: entry.timestamp.clone(),
                    uuid: entry.uuid.clone(),
                });

                // Extract tool_use blocks from content array
                if let Some(serde_json::Value::Array(arr)) = &content_val {
                    for item in arr {
                        if item.get("type").and_then(|t| t.as_str()) == Some("tool_use") {
                            let tool_name = item
                                .get("name")
                                .and_then(|n| n.as_str())
                                .unwrap_or("")
                                .to_string();
                            let tool_input = item
                                .get("input")
                                .cloned()
                                .unwrap_or(serde_json::Value::Null);
                            let tool_use_id = item
                                .get("id")
                                .and_then(|id| id.as_str())
                                .unwrap_or("")
                                .to_string();

                            let tc_idx = tool_calls.len();

                            // Extract file edits, skipping excluded paths
                            if is_file_edit(&tool_name)
                                && let Some(edit) = extract_file_edit(&tool_name, &tool_input, idx)
                                && !should_skip_file(&edit.file_path, excluded)
                            {
                                file_edits.push(edit);
                            }

                            tool_calls.push(ToolCall {
                                name: tool_name,
                                input: tool_input,
                                result: None,
                                index: idx,
                                tool_use_id: tool_use_id.clone(),
                            });

                            if !tool_use_id.is_empty() {
                                tool_use_map.insert(tool_use_id, tc_idx);
                            }
                        }
                    }
                }
            }

            "user" => {
                let msg = entry.message.as_ref();
                let content_val = msg.and_then(|m| m.content.clone());

                // Skip messages containing <private> tags
                if content_has_private_tag(&content_val) {
                    continue;
                }

                // Check if this is a tool_result response
                if let Some(serde_json::Value::Array(arr)) = &content_val {
                    for item in arr {
                        if item.get("type").and_then(|t| t.as_str()) == Some("tool_result") {
                            let tool_use_id = item
                                .get("tool_use_id")
                                .and_then(|id| id.as_str())
                                .unwrap_or("");
                            let is_error = item
                                .get("is_error")
                                .and_then(|e| e.as_bool())
                                .unwrap_or(false);

                            let result_content = match item.get("content") {
                                Some(serde_json::Value::String(s)) => s.clone(),
                                Some(serde_json::Value::Array(arr)) => arr
                                    .iter()
                                    .filter_map(|v| {
                                        v.get("text").and_then(|t| t.as_str()).map(String::from)
                                    })
                                    .collect::<Vec<_>>()
                                    .join("\n"),
                                _ => String::new(),
                            };

                            // Link result to tool call
                            if let Some(&tc_idx) = tool_use_map.get(tool_use_id) {
                                let result = ToolResult {
                                    content: result_content.clone(),
                                    is_error,
                                };
                                tool_calls[tc_idx].result = Some(result);

                                // Extract errors (skip noise)
                                if (is_error || has_error_indicators(&result_content))
                                    && !is_noise_error(&result_content)
                                {
                                    let tool_name = tool_calls[tc_idx].name.clone();
                                    let error_msg = extract_error_line(&result_content);
                                    if !error_msg.is_empty() && !is_noise_error(&error_msg) {
                                        errors.push(ErrorEvent {
                                            message: error_msg,
                                            tool_name,
                                            index: idx,
                                        });
                                    }
                                }
                            }

                            continue;
                        }
                    }
                }

                // Regular user message (not tool result)
                messages.push(SessionMessage {
                    entry_type: "user".to_string(),
                    role: Some("user".to_string()),
                    content: content_val,
                    timestamp: entry.timestamp.clone(),
                    uuid: entry.uuid.clone(),
                });
            }

            _ => {}
        }
    }

    Ok(ParsedSession {
        id: session_id,
        project_path,
        messages,
        tool_calls,
        errors,
        file_edits,
    })
}

fn extract_project_path(session_path: &Path) -> String {
    // Claude Code: ~/.claude/projects/{encoded_project_path}/{session_id}.jsonl
    session_path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string()
}

fn is_file_edit(tool_name: &str) -> bool {
    matches!(tool_name, "Edit" | "Write" | "NotebookEdit")
}

fn extract_file_edit(tool_name: &str, input: &serde_json::Value, index: usize) -> Option<FileEdit> {
    let file_path = input.get("file_path")?.as_str()?.to_string();

    let (old_content, new_content) = match tool_name {
        "Edit" => (
            input
                .get("old_string")
                .and_then(|v| v.as_str())
                .map(String::from),
            input
                .get("new_string")
                .and_then(|v| v.as_str())
                .map(String::from),
        ),
        "Write" => (
            None,
            input
                .get("content")
                .and_then(|v| v.as_str())
                .map(String::from),
        ),
        _ => (None, None),
    };

    Some(FileEdit {
        file_path,
        old_content,
        new_content,
        tool_name: tool_name.to_string(),
        index,
    })
}

/// Filter out noise errors that aren't actionable patterns
fn is_noise_error(text: &str) -> bool {
    let lower = text.to_lowercase();

    // Tool use rejections (user cancelled)
    if lower.contains("tool_use_error") || lower.contains("cancelled: parallel tool call") {
        return true;
    }
    // User rejected tool use
    if lower.contains("the user doesn't want to proceed") || lower.contains("tool use was rejected")
    {
        return true;
    }
    // Code snippets misidentified as errors (line numbers at start)
    if text.starts_with(|c: char| c.is_ascii_digit()) && text.contains('\t') {
        return true;
    }
    // Diff output
    if text.starts_with("diff --git") || text.starts_with("---") || text.starts_with("+++") {
        return true;
    }
    // Import statements
    if text.starts_with("import ") || text.starts_with("from ") {
        return true;
    }

    false
}

fn has_error_indicators(text: &str) -> bool {
    text.contains("Error:")
        || text.contains("error:")
        || text.contains("FAILED")
        || text.contains("error[")
        || text.contains("Exit code")
}

fn extract_error_line(text: &str) -> String {
    for line in text.lines() {
        let line = line.trim();
        if line.contains("Error:")
            || line.contains("error:")
            || line.contains("FAILED")
            || line.contains("error[")
        {
            return line.to_string();
        }
    }
    // If no specific error line found, take first non-empty line
    text.lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_jsonl_line(entry_type: &str, role: &str, content: serde_json::Value) -> String {
        serde_json::json!({
            "type": entry_type,
            "uuid": "test-uuid-123",
            "parentUuid": null,
            "timestamp": "2025-01-01T00:00:00Z",
            "sessionId": "test-session",
            "cwd": "/test/project",
            "message": {
                "role": role,
                "content": content
            }
        })
        .to_string()
    }

    #[test]
    fn parse_assistant_with_tool_use() {
        let content = serde_json::json!([
            {
                "type": "text",
                "text": "Let me edit that file."
            },
            {
                "type": "tool_use",
                "id": "tu_123",
                "name": "Edit",
                "input": {
                    "file_path": "/test/src/main.rs",
                    "old_string": "old code",
                    "new_string": "new code"
                }
            }
        ]);

        let line = make_jsonl_line("assistant", "assistant", content);

        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), &line).unwrap();

        let session = parse_session(tmp.path()).unwrap();
        assert_eq!(session.tool_calls.len(), 1);
        assert_eq!(session.tool_calls[0].name, "Edit");
        assert_eq!(session.tool_calls[0].tool_use_id, "tu_123");
        assert_eq!(session.file_edits.len(), 1);
        assert_eq!(session.file_edits[0].file_path, "/test/src/main.rs");
    }

    #[test]
    fn parse_user_with_tool_result() {
        let assistant_line = make_jsonl_line(
            "assistant",
            "assistant",
            serde_json::json!([
                {
                    "type": "tool_use",
                    "id": "tu_456",
                    "name": "Bash",
                    "input": {"command": "cargo build"}
                }
            ]),
        );

        let user_line = make_jsonl_line(
            "user",
            "user",
            serde_json::json!([
                {
                    "type": "tool_result",
                    "tool_use_id": "tu_456",
                    "is_error": true,
                    "content": "error: cannot find crate `foo`"
                }
            ]),
        );

        let jsonl = format!("{}\n{}", assistant_line, user_line);

        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), &jsonl).unwrap();

        let session = parse_session(tmp.path()).unwrap();
        assert_eq!(session.tool_calls.len(), 1);
        assert!(session.tool_calls[0].result.is_some());
        let result = session.tool_calls[0].result.as_ref().unwrap();
        assert!(result.is_error);
        assert!(result.content.contains("cannot find crate"));
        assert!(!session.errors.is_empty());
    }

    #[test]
    fn extract_file_edit_from_edit_tool() {
        let input = serde_json::json!({
            "file_path": "/test/src/lib.rs",
            "old_string": "fn old() {}",
            "new_string": "fn new() {}"
        });

        let edit = extract_file_edit("Edit", &input, 0).unwrap();
        assert_eq!(edit.file_path, "/test/src/lib.rs");
        assert_eq!(edit.old_content.as_deref(), Some("fn old() {}"));
        assert_eq!(edit.new_content.as_deref(), Some("fn new() {}"));
        assert_eq!(edit.tool_name, "Edit");
    }

    #[test]
    fn extract_file_edit_from_write_tool() {
        let input = serde_json::json!({
            "file_path": "/test/new_file.rs",
            "content": "fn main() {}"
        });

        let edit = extract_file_edit("Write", &input, 0).unwrap();
        assert_eq!(edit.file_path, "/test/new_file.rs");
        assert!(edit.old_content.is_none());
        assert_eq!(edit.new_content.as_deref(), Some("fn main() {}"));
    }

    #[test]
    fn is_file_edit_recognizes_edit_tools() {
        assert!(is_file_edit("Edit"));
        assert!(is_file_edit("Write"));
        assert!(is_file_edit("NotebookEdit"));
        assert!(!is_file_edit("Bash"));
        assert!(!is_file_edit("Read"));
    }

    #[test]
    fn has_error_indicators_works() {
        assert!(has_error_indicators("Error: something failed"));
        assert!(has_error_indicators("error: bad input"));
        assert!(has_error_indicators("test FAILED"));
        assert!(has_error_indicators("error[E0308]: mismatched types"));
        assert!(has_error_indicators("Exit code 1"));
        assert!(!has_error_indicators("everything is fine"));
    }

    #[test]
    fn extract_error_line_finds_error() {
        let text = "Compiling foo v0.1.0\nerror: cannot find crate\nnote: see docs";
        let line = extract_error_line(text);
        assert_eq!(line, "error: cannot find crate");
    }

    #[test]
    fn parse_empty_file() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), "").unwrap();

        let session = parse_session(tmp.path()).unwrap();
        assert!(session.messages.is_empty());
        assert!(session.tool_calls.is_empty());
        assert!(session.errors.is_empty());
        assert!(session.file_edits.is_empty());
    }
}
