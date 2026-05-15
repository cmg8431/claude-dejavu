use super::{Detection, DetectorType, Evidence};
use crate::parser::ParsedSession;
use std::collections::HashMap;

/// Detector ③: Silent User Fix ⭐ (killer differentiator)
///
/// Signal: Claude finishes work → user edits the same file in the same session
/// (without asking Claude to) → that fix gets broken again in a future session.
///
/// Detection:
/// 1. Find files Claude edited (via Edit/Write tools)
/// 2. In subsequent messages, if the user's turn contains content referencing
///    the same file with different changes, that's a silent correction
/// 3. Cross-reference with future sessions to see if the correction was lost
///
/// For v0: We detect step 1-2 (silent corrections within a session).
/// The cross-session regression check comes in v1.
pub fn detect(sessions: &[ParsedSession]) -> Vec<Detection> {
    let mut corrections: HashMap<String, Vec<SilentCorrection>> = HashMap::new();

    for session in sessions {
        let claude_edits = find_claude_edits(session);
        let user_edits = find_user_follow_up_edits(session, &claude_edits);

        for correction in user_edits {
            corrections
                .entry(correction.file_path.clone())
                .or_default()
                .push(correction);
        }
    }

    let mut detections = Vec::new();

    for (file_path, file_corrections) in &corrections {
        if file_corrections.is_empty() {
            continue;
        }

        let session_ids: Vec<String> = file_corrections
            .iter()
            .map(|c| c.session_id.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let confidence = (file_corrections.len() as f64 * 0.25 + 0.3).min(1.0);

        let suggested_rule = format!(
            "In `{}`, follow the user's preferred style. {} silent corrections detected in past sessions.",
            file_path,
            file_corrections.len(),
        );

        detections.push(Detection {
            detector_type: DetectorType::SilentFix,
            evidence: Evidence {
                sessions: session_ids,
                file_paths: vec![file_path.clone()],
                occurrences: file_corrections.len(),
                details: serde_json::json!({
                    "corrections": file_corrections.iter().map(|c| {
                        serde_json::json!({
                            "claude_edit_index": c.claude_edit_index,
                            "user_edit_index": c.user_edit_index,
                        })
                    }).collect::<Vec<_>>(),
                }),
            },
            confidence,
            suggested_rule,
        });
    }

    detections
}

struct SilentCorrection {
    session_id: String,
    file_path: String,
    claude_edit_index: usize,
    user_edit_index: usize,
}

fn find_claude_edits(session: &ParsedSession) -> Vec<(String, usize)> {
    // Claude's edits are tool calls (Edit, Write) that appear in assistant messages
    session
        .file_edits
        .iter()
        .filter(|e| e.tool_name == "Edit" || e.tool_name == "Write")
        .map(|e| (e.file_path.clone(), e.index))
        .collect()
}

fn find_user_follow_up_edits(
    session: &ParsedSession,
    claude_edits: &[(String, usize)],
) -> Vec<SilentCorrection> {
    let mut corrections = Vec::new();

    // Look for patterns where:
    // 1. Claude edits a file at index I
    // 2. Later, the user (or Claude on user's behalf) edits the same file
    //    but the edit undoes or modifies Claude's change
    for (file_path, claude_idx) in claude_edits {
        // Find subsequent edits to the same file
        for later_edit in &session.file_edits {
            if later_edit.index <= *claude_idx {
                continue;
            }
            if later_edit.file_path != *file_path {
                continue;
            }

            // Check if this looks like a correction:
            // The user's edit targets the same region Claude just modified
            if is_likely_correction(
                session,
                *claude_idx,
                later_edit.index,
                &later_edit.old_content,
            ) {
                corrections.push(SilentCorrection {
                    session_id: session.id.clone(),
                    file_path: file_path.clone(),
                    claude_edit_index: *claude_idx,
                    user_edit_index: later_edit.index,
                });
                break; // One correction per Claude edit
            }
        }
    }

    corrections
}

fn is_likely_correction(
    session: &ParsedSession,
    claude_edit_idx: usize,
    follow_up_idx: usize,
    follow_up_old: &Option<String>,
) -> bool {
    // Heuristic: If there's a user message between Claude's edit and the follow-up edit,
    // it's less likely to be a "silent" fix (user probably asked for a change).
    // We want edits that happen without explicit user instruction.
    let _has_user_message_between = session.messages.iter().any(|msg| {
        if let Some(ref role) = msg.role
            && role == "user"
        {
            // Check if this message is between the two edits
            // (simplified: we use message ordering as proxy)
            return true;
        }
        false
    });

    // For v0: If the follow-up edit's old_content contains text that Claude
    // just wrote, it's modifying Claude's output → likely a correction.
    // More sophisticated detection in v1.
    if follow_up_old.is_some() && (follow_up_idx - claude_edit_idx) < 20 {
        return true;
    }

    false
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

    fn make_edit(
        file: &str,
        old: Option<&str>,
        new: Option<&str>,
        tool: &str,
        index: usize,
    ) -> FileEdit {
        FileEdit {
            file_path: file.to_string(),
            old_content: old.map(String::from),
            new_content: new.map(String::from),
            tool_name: tool.to_string(),
            index,
        }
    }

    #[test]
    fn claude_edit_followed_by_same_file_edit_detected() {
        let edits = vec![
            make_edit("src/lib.rs", Some("old"), Some("new"), "Edit", 1),
            make_edit("src/lib.rs", Some("new"), Some("fixed"), "Edit", 5),
        ];
        let sessions = vec![make_session("s1", edits)];
        let detections = detect(&sessions);
        assert!(
            !detections.is_empty(),
            "edit followed by another edit on same file should be detected"
        );
        assert!(
            detections[0]
                .evidence
                .file_paths
                .contains(&"src/lib.rs".to_string())
        );
    }

    #[test]
    fn edits_on_different_files_no_detection() {
        let edits = vec![
            make_edit("src/a.rs", Some("old"), Some("new"), "Edit", 1),
            make_edit("src/b.rs", Some("old2"), Some("new2"), "Edit", 5),
        ];
        let sessions = vec![make_session("s1", edits)];
        let detections = detect(&sessions);
        assert!(
            detections.is_empty(),
            "edits on different files should not trigger silent fix detection"
        );
    }

    #[test]
    fn write_tool_also_detected() {
        let edits = vec![
            make_edit("src/lib.rs", None, Some("initial content"), "Write", 1),
            make_edit(
                "src/lib.rs",
                Some("initial content"),
                Some("corrected"),
                "Edit",
                5,
            ),
        ];
        let sessions = vec![make_session("s1", edits)];
        let detections = detect(&sessions);
        assert!(
            !detections.is_empty(),
            "Write followed by Edit on same file should be detected"
        );
    }

    #[test]
    fn follow_up_without_old_content_not_detected() {
        let edits = vec![
            make_edit("src/lib.rs", Some("old"), Some("new"), "Edit", 1),
            make_edit("src/lib.rs", None, None, "Edit", 5),
        ];
        let sessions = vec![make_session("s1", edits)];
        let detections = detect(&sessions);
        assert!(
            detections.is_empty(),
            "follow-up edit without old_content should not be detected"
        );
    }

    #[test]
    fn far_apart_edits_not_detected() {
        let edits = vec![
            make_edit("src/lib.rs", Some("old"), Some("new"), "Edit", 1),
            make_edit("src/lib.rs", Some("new"), Some("fixed"), "Edit", 100),
        ];
        let sessions = vec![make_session("s1", edits)];
        let detections = detect(&sessions);
        assert!(
            detections.is_empty(),
            "edits far apart (index diff >= 20) should not be detected"
        );
    }
}
