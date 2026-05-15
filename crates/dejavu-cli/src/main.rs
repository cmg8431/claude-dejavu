mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "claude-dejavu", version, about = "Claude remembers its mistakes.")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize dejavu for the current project
    Init,
    /// Scan session logs and detect antipatterns
    Scan {
        /// Project path (defaults to current directory)
        #[arg(short, long)]
        path: Option<String>,
        /// Auto-apply rules without prompting
        #[arg(long)]
        auto: bool,
    },
    /// List learned rules for this project
    List {
        /// Project path (defaults to current directory)
        #[arg(short, long)]
        path: Option<String>,
    },
    /// Show rule effectiveness statistics
    Stats {
        /// Project path (defaults to current directory)
        #[arg(short, long)]
        path: Option<String>,
    },
    /// Output active rules as JSON for hook injection
    Inject {
        /// Project path
        #[arg(short, long)]
        path: Option<String>,
        /// Output format (json or text)
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// Check for pending patterns (used by SessionStart hook)
    Check {
        /// Project path
        #[arg(short, long)]
        path: Option<String>,
        /// Quiet JSON output
        #[arg(long)]
        quiet: bool,
    },
    /// Ingest tool usage buffers from hooks
    Ingest {
        /// Directory containing .jsonl buffer files
        #[arg(long)]
        buffer_dir: String,
        /// Project path
        #[arg(short, long)]
        path: Option<String>,
    },
    /// Watch session logs in real-time (daemon mode)
    Watch {
        /// Project path
        #[arg(short, long)]
        path: Option<String>,
    },
    /// Find and remove dead rules (no fires in N days)
    Cleanup {
        /// Project path
        #[arg(short, long)]
        path: Option<String>,
        /// Days without fire to consider dead (default: 14)
        #[arg(long, default_value = "14")]
        days: i64,
        /// Actually remove dead rules
        #[arg(long)]
        apply: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => commands::init::run(),
        Commands::Scan { path, auto } => commands::scan::run(path, auto),
        Commands::List { path } => commands::list::run(path),
        Commands::Stats { path } => commands::stats::run(path),
        Commands::Inject { path, format } => commands::inject::run(path, format),
        Commands::Check { path, quiet } => commands::check::run(path, quiet),
        Commands::Ingest { buffer_dir, path } => commands::ingest::run(buffer_dir, path),
        Commands::Watch { path } => commands::watch::run(path),
        Commands::Cleanup { path, days, apply } => {
            if apply {
                commands::cleanup::run_apply(path, days)
            } else {
                commands::cleanup::run(path, days)
            }
        }
    }
}
