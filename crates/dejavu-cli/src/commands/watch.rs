use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

pub fn run(path: Option<String>) -> Result<()> {
    let project_path = path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    println!("{}", "👁  dejavu watch mode".bold());
    println!(
        "  Watching session logs for: {}",
        project_path.display().to_string().cyan()
    );
    println!("  Press Ctrl+C to stop.\n");

    let engine = dejavu_core::DejavuEngine::new()?;

    let claude_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("no home dir"))?
        .join(".claude")
        .join("projects");

    if !claude_dir.exists() {
        println!(
            "  {} ~/.claude/projects/ not found. Start a Claude Code session first.",
            "⚠".yellow()
        );
        return Ok(());
    }

    let mut last_scan_count = 0;
    let interval = std::time::Duration::from_secs(30);

    loop {
        // Count current session files
        let files = dejavu_core::parser::find_session_files(
            &dirs::home_dir()
                .ok_or_else(|| anyhow::anyhow!("could not find home directory"))?
                .join(".claude"),
        )?;

        let current_count = files.len();

        if current_count != last_scan_count {
            let now = chrono::Local::now().format("%H:%M:%S");
            println!(
                "  [{}] {} session files found, scanning...",
                now.to_string().dimmed(),
                current_count
            );

            match engine.scan(&project_path) {
                Ok(detections) => {
                    if !detections.is_empty() {
                        println!(
                            "  [{}] {} {} new patterns detected!",
                            now.to_string().dimmed(),
                            "✓".green(),
                            detections.len(),
                        );

                        let rules = engine.generate_rules(&project_path, &detections)?;
                        if !rules.is_empty() {
                            let content = engine.apply_rules(&project_path, &rules)?;
                            dejavu_core::rule::write_claude_md(&project_path, &content)?;
                            println!(
                                "  [{}] {} {} rules written to CLAUDE.md",
                                now.to_string().dimmed(),
                                "✓".green(),
                                rules.len(),
                            );
                        }
                    }
                }
                Err(e) => {
                    eprintln!("  [{}] scan error: {}", now, e);
                }
            }

            last_scan_count = current_count;
        }

        std::thread::sleep(interval);
    }
}
