use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

pub fn run(path: Option<String>, auto: bool) -> Result<()> {
    let project_path = path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    if !auto {
        println!("{}", "🔍 Scanning session logs...".bold());
    }

    let engine = dejavu_core::DejavuEngine::new()?;
    let detections = engine.scan(&project_path)?;

    if detections.is_empty() {
        if !auto {
            println!("  {} No antipatterns detected yet.", "ℹ".blue());
            println!(
                "  {}",
                "Keep using Claude Code — dejavu will find patterns over time."
                    .dimmed()
            );
        }
        return Ok(());
    }

    if !auto {
        println!(
            "  {} {} patterns detected.\n",
            "✓".green(),
            detections.len()
        );
    }

    let rules = engine.generate_rules(&project_path, &detections)?;

    if rules.is_empty() {
        if !auto {
            println!(
                "  {} No rules with sufficient confidence yet.",
                "ℹ".blue()
            );
        }
        return Ok(());
    }

    if !auto {
        for (id, detection) in &rules {
            let detector_label = match detection.detector_type {
                dejavu_core::DetectorType::RevertCycle => "revert cycle".yellow(),
                dejavu_core::DetectorType::RepeatedError => "repeated error".red(),
                dejavu_core::DetectorType::SilentFix => "silent fix".magenta(),
            dejavu_core::DetectorType::UserCorrection => "user correction".cyan(),
            };

            println!(
                "┌─ Proposed rule {} ─ [{}] ──────────────────────────",
                id, detector_label
            );
            println!("│ \"{}\"", detection.suggested_rule);
            println!(
                "│ Evidence: {} sessions, {} occurrences",
                detection.evidence.sessions.len(),
                detection.evidence.occurrences,
            );
            println!("│ Confidence: {:.2}", detection.confidence);
            println!("│ Apply? [y/n/edit]");
            println!("└──────────────────────────────────────────────────");
            println!();
        }
    }

    let content = engine.apply_rules(&project_path, &rules)?;
    dejavu_core::rule::write_claude_md(&project_path, &content)?;

    if !auto {
        println!(
            "  {} {} rules written to CLAUDE.md",
            "✓".green(),
            rules.len()
        );
    } else {
        eprintln!("dejavu: {} rules applied to CLAUDE.md", rules.len());
    }

    Ok(())
}
