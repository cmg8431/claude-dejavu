use super::{Detection, DetectorType, Evidence};
use crate::parser::ParsedSession;
use std::collections::HashMap;

/// Detector ①: Edit → Revert → Re-edit (Revert Cycle)
///
/// Signal: Same file edited, then content reverted (partially or fully)
/// back toward a previous version within the same session or across sessions.
///
/// Detection: Track file edit history per file. If old_content of edit N+k
/// is similar to new_content of edit N, that's a revert.
pub fn detect(sessions: &[ParsedSession]) -> Vec<Detection> {
    let mut file_edit_history: HashMap<String, Vec<EditSnapshot>> = HashMap::new();

    for session in sessions {
        for edit in &session.file_edits {
            file_edit_history
                .entry(edit.file_path.clone())
                .or_default()
                .push(EditSnapshot {
                    session_id: session.id.clone(),
                    old_content: edit.old_content.clone(),
                    new_content: edit.new_content.clone(),
                    _index: edit.index,
                });
        }
    }

    let mut detections = Vec::new();

    for (file_path, edits) in &file_edit_history {
        if edits.len() < 3 {
            continue;
        }

        let cycles = find_revert_cycles(edits);

        if cycles.is_empty() {
            continue;
        }

        let session_ids: Vec<String> = cycles
            .iter()
            .flat_map(|c| c.session_ids.iter().cloned())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let confidence = (cycles.len() as f64 * 0.3).min(1.0);

        let suggested_rule = format!(
            "In `{}`, check existing logic before editing. Past sessions show {} revert cycles in this file.",
            file_path,
            cycles.len(),
        );

        detections.push(Detection {
            detector_type: DetectorType::RevertCycle,
            evidence: Evidence {
                sessions: session_ids,
                file_paths: vec![file_path.clone()],
                occurrences: cycles.len(),
                details: serde_json::json!({
                    "cycles": cycles.len(),
                    "file": file_path,
                }),
            },
            confidence,
            suggested_rule,
        });
    }

    detections
}

struct EditSnapshot {
    session_id: String,
    old_content: Option<String>,
    new_content: Option<String>,
    _index: usize,
}

struct RevertCycle {
    session_ids: Vec<String>,
}

fn find_revert_cycles(edits: &[EditSnapshot]) -> Vec<RevertCycle> {
    let mut cycles = Vec::new();

    for i in 0..edits.len() {
        for j in (i + 2)..edits.len() {
            // Check if edit j reverts back toward edit i's state
            let Some(ref content_after_i) = edits[i].new_content else {
                continue;
            };
            let Some(ref content_before_j) = edits[j].old_content else {
                continue;
            };

            let similarity = text_similarity(content_after_i, content_before_j);

            // If later edit's old_content is similar to earlier edit's new_content,
            // it means the changes were partially reverted in between
            if similarity > 0.7 {
                let mut session_ids = vec![edits[i].session_id.clone()];
                if edits[j].session_id != edits[i].session_id {
                    session_ids.push(edits[j].session_id.clone());
                }
                cycles.push(RevertCycle { session_ids });
            }
        }
    }

    cycles
}

fn text_similarity(a: &str, b: &str) -> f64 {
    if a == b {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }

    let a_lines: Vec<&str> = a.lines().collect();
    let b_lines: Vec<&str> = b.lines().collect();

    let common = a_lines.iter().filter(|l| b_lines.contains(l)).count();
    let total = a_lines.len().max(b_lines.len());

    if total == 0 {
        return 0.0;
    }

    common as f64 / total as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{FileEdit, ParsedSession};

    fn make_session(id: &str, file_edits: Vec<FileEdit>) -> ParsedSession {
        ParsedSession {
            id: id.to_string(),
            project_path: "/test/project".to_string(),
            messages: vec![],
            tool_calls: vec![],
            errors: vec![],
            file_edits,
        }
    }

    fn make_edit(file: &str, old: Option<&str>, new: Option<&str>, index: usize) -> FileEdit {
        FileEdit {
            file_path: file.to_string(),
            old_content: old.map(String::from),
            new_content: new.map(String::from),
            tool_name: "Edit".to_string(),
            index,
        }
    }

    #[test]
    fn three_edits_same_file_with_revert_detected() {
        // Edit 0: write "version A"
        // Edit 1: change to something else (old="version A", new="version B")
        // Edit 2: revert back (old="version A", new="version C")
        //   -> edit 2's old_content matches edit 0's new_content = revert cycle
        let edits = vec![
            make_edit(
                "src/main.rs",
                None,
                Some("fn main() {\n    println!(\"hello\");\n}"),
                0,
            ),
            make_edit(
                "src/main.rs",
                Some("fn main() {\n    println!(\"hello\");\n}"),
                Some("fn main() {\n    println!(\"world\");\n}"),
                1,
            ),
            make_edit(
                "src/main.rs",
                Some("fn main() {\n    println!(\"hello\");\n}"),
                Some("fn main() {\n    println!(\"fixed\");\n}"),
                2,
            ),
        ];
        let sessions = vec![make_session("s1", edits)];
        let detections = detect(&sessions);
        assert!(
            !detections.is_empty(),
            "should detect revert cycle on same file"
        );
        assert!(
            detections[0]
                .evidence
                .file_paths
                .contains(&"src/main.rs".to_string())
        );
    }

    #[test]
    fn different_files_no_detection() {
        let edits = vec![
            make_edit("src/a.rs", None, Some("content a"), 0),
            make_edit("src/b.rs", None, Some("content b"), 1),
            make_edit("src/c.rs", None, Some("content c"), 2),
        ];
        let sessions = vec![make_session("s1", edits)];
        let detections = detect(&sessions);
        assert!(
            detections.is_empty(),
            "edits on different files should not trigger revert cycle"
        );
    }

    #[test]
    fn fewer_than_three_edits_no_detection() {
        let edits = vec![
            make_edit("src/main.rs", None, Some("content"), 0),
            make_edit("src/main.rs", Some("content"), Some("changed"), 1),
        ];
        let sessions = vec![make_session("s1", edits)];
        let detections = detect(&sessions);
        assert!(
            detections.is_empty(),
            "fewer than 3 edits should not trigger detection"
        );
    }

    #[test]
    fn text_similarity_identical() {
        assert_eq!(text_similarity("hello", "hello"), 1.0);
    }

    #[test]
    fn text_similarity_empty() {
        assert_eq!(text_similarity("", "hello"), 0.0);
        assert_eq!(text_similarity("hello", ""), 0.0);
    }

    #[test]
    fn text_similarity_partial() {
        let a = "line1\nline2\nline3";
        let b = "line1\nline2\nline4";
        let sim = text_similarity(a, b);
        assert!(
            sim > 0.5 && sim < 1.0,
            "partial overlap should be between 0.5 and 1.0, got {}",
            sim
        );
    }
}
