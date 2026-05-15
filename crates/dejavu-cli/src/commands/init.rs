use anyhow::Result;
use colored::Colorize;

pub fn run() -> Result<()> {
    let cwd = std::env::current_dir()?;

    println!("{}", "🔮 Initializing claude-dejavu...".bold());

    // Create .dejavu directory
    let dejavu_dir = cwd.join(".dejavu");
    std::fs::create_dir_all(&dejavu_dir)?;

    // Initialize database
    let engine = dejavu_core::DejavuEngine::new()?;
    let _conn = dejavu_core::db::open(&engine.db_path)?;

    // Check if CLAUDE.md exists
    let claude_md = cwd.join("CLAUDE.md");
    if !claude_md.exists() {
        println!(
            "  {} No CLAUDE.md found. One will be created when rules are applied.",
            "ℹ".blue()
        );
    } else {
        println!("  {} CLAUDE.md found.", "✓".green());
    }

    println!(
        "  {} Database initialized at {:?}",
        "✓".green(),
        engine.db_path
    );
    println!("  {} .dejavu/ directory created", "✓".green());

    println!();
    println!(
        "{}",
        "dejavu is ready. Use your project normally with Claude Code.".dimmed()
    );
    println!(
        "{}",
        "Run `claude-dejavu scan` after a few sessions to detect patterns.".dimmed()
    );

    Ok(())
}
