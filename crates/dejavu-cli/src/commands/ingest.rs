use anyhow::Result;
use std::path::PathBuf;

pub fn run(buffer_dir: String, path: Option<String>) -> Result<()> {
    let project_path = path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    let buffer_path = PathBuf::from(&buffer_dir);
    if !buffer_path.exists() {
        return Ok(());
    }

    let engine = dejavu_core::DejavuEngine::new()?;
    let conn = dejavu_core::db::open(&engine.db_path)?;

    let mut total_events = 0;

    for entry in std::fs::read_dir(&buffer_path)? {
        let entry = entry?;
        let file_path = entry.path();

        if file_path.extension().is_some_and(|ext| ext == "jsonl") {
            let content = std::fs::read_to_string(&file_path)?;

            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                if let Ok(event) = serde_json::from_str::<serde_json::Value>(line) {
                    let session_id = event
                        .get("session_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");

                    let tool_name = event
                        .get("tool_name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");

                    // Store as a pattern evidence event
                    let evidence = serde_json::json!({
                        "tool_name": tool_name,
                        "tool_input": event.get("tool_input"),
                        "tool_response": event.get("tool_response"),
                        "timestamp": event.get("timestamp"),
                    });

                    dejavu_core::db::insert_pattern(
                        &conn,
                        "tool_event",
                        &project_path.to_string_lossy(),
                        session_id,
                        &evidence.to_string(),
                        None,
                    )?;

                    total_events += 1;
                }
            }
        }
    }

    if total_events > 0 {
        eprintln!("dejavu: ingested {} tool events", total_events);
    }

    Ok(())
}
