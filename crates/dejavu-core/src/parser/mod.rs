use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub struct SessionMessage {
    #[serde(rename = "type")]
    pub msg_type: Option<String>,
    pub role: Option<String>,
    pub content: Option<serde_json::Value>,
    pub tool_name: Option<String>,
    pub tool_input: Option<serde_json::Value>,
    pub tool_result: Option<serde_json::Value>,
    pub timestamp: Option<String>,
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
    pub result: Option<serde_json::Value>,
    pub index: usize,
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
    let content = std::fs::read_to_string(path)?;
    let mut messages = Vec::new();
    let mut tool_calls = Vec::new();
    let mut errors = Vec::new();
    let mut file_edits = Vec::new();

    let session_id = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let project_path = extract_project_path(path);

    for (idx, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Ok(msg) = serde_json::from_str::<SessionMessage>(line) {
            if let Some(ref tool_name) = msg.tool_name {
                let tc = ToolCall {
                    name: tool_name.clone(),
                    input: msg.tool_input.clone().unwrap_or(serde_json::Value::Null),
                    result: msg.tool_result.clone(),
                    index: idx,
                };

                if is_file_edit(tool_name) {
                    if let Some(edit) = extract_file_edit(&tc) {
                        file_edits.push(edit);
                    }
                }

                if let Some(err) = extract_error(&tc) {
                    errors.push(ErrorEvent {
                        message: err,
                        tool_name: tool_name.clone(),
                        index: idx,
                    });
                }

                tool_calls.push(tc);
            }
            messages.push(msg);
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
    // Claude Code stores sessions under ~/.claude/projects/{encoded_project_path}/
    session_path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .map(|encoded| {
            // Decode: `-` separated path segments, common encoding
            encoded.replace("-", "/")
        })
        .unwrap_or_else(|| "unknown".to_string())
}

fn is_file_edit(tool_name: &str) -> bool {
    matches!(tool_name, "Edit" | "Write" | "NotebookEdit")
}

fn extract_file_edit(tc: &ToolCall) -> Option<FileEdit> {
    let input = &tc.input;
    let file_path = input.get("file_path")?.as_str()?.to_string();
    let old_content = input.get("old_string").and_then(|v| v.as_str()).map(String::from);
    let new_content = input.get("new_string").and_then(|v| v.as_str()).map(String::from);

    Some(FileEdit {
        file_path,
        old_content,
        new_content,
        tool_name: tc.name.clone(),
        index: tc.index,
    })
}

fn extract_error(tc: &ToolCall) -> Option<String> {
    let result = tc.result.as_ref()?;

    // Check for error indicators in tool results
    if let Some(text) = result.as_str() {
        if text.contains("Error:") || text.contains("error:") || text.contains("FAILED") {
            return Some(text.lines().find(|l| {
                l.contains("Error:") || l.contains("error:") || l.contains("FAILED")
            })?.to_string());
        }
    }

    if let Some(obj) = result.as_object() {
        if let Some(err) = obj.get("error") {
            return Some(err.to_string());
        }
    }

    None
}
