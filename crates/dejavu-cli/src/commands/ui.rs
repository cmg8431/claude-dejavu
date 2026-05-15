use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

pub fn run(port: u16, background: bool) -> Result<()> {
    let data_dir = dejavu_core::db::default_data_dir()?;
    let pid_file = data_dir.join("dashboard.pid");

    // Check if already running
    if let Some(pid) = read_pid(&pid_file)
        && is_process_alive(pid)
    {
        println!(
            "  {} Dashboard already running at {} (PID {})",
            "✓".green(),
            format!("http://localhost:{}", port).cyan(),
            pid,
        );
        return Ok(());
    }

    // Clean stale PID
    let _ = std::fs::remove_file(&pid_file);

    let dashboard_dir = find_dashboard_dir();

    let Some(dashboard_dir) = dashboard_dir else {
        eprintln!("  {} Dashboard not found. Install from source:", "✗".red());
        eprintln!("    cd ~/dev/proj/claude-dejavu/packages/dashboard");
        eprintln!("    npm install && npm run build");
        return Ok(());
    };

    ensure_node_modules(&dashboard_dir)?;

    let url = format!("http://localhost:{}", port);

    if background {
        let child = std::process::Command::new("npm")
            .args(["run", "dev"])
            .current_dir(&dashboard_dir)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?;

        std::fs::write(&pid_file, child.id().to_string())?;
        println!(
            "  {} Dashboard started at {} (PID {})",
            "✓".green(),
            url.cyan(),
            child.id(),
        );
        open_browser(&url);
    } else {
        println!(
            "  {} Starting dashboard at {}",
            "🔮".to_string().bold(),
            url.cyan(),
        );
        println!("  Press Ctrl+C to stop.\n");

        open_browser(&url);

        let status = std::process::Command::new("npm")
            .args(["run", "dev"])
            .current_dir(&dashboard_dir)
            .status()?;

        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

pub fn stop() -> Result<()> {
    let data_dir = dejavu_core::db::default_data_dir()?;
    let pid_file = data_dir.join("dashboard.pid");

    if let Some(pid) = read_pid(&pid_file) {
        if is_process_alive(pid) {
            let _ = std::process::Command::new("kill")
                .arg(pid.to_string())
                .status();
            println!("  {} Dashboard stopped (PID {})", "✓".green(), pid);
        }
        let _ = std::fs::remove_file(&pid_file);
    } else {
        println!("  {} Dashboard not running.", "ℹ".blue());
    }

    Ok(())
}

fn find_dashboard_dir() -> Option<PathBuf> {
    let candidates = [
        std::env::current_exe().ok().and_then(|p| {
            p.parent()
                .map(|p| p.join("../../packages/dashboard").to_path_buf())
        }),
        dirs::home_dir().map(|h| h.join("dev/proj/claude-dejavu/packages/dashboard")),
        std::env::current_dir()
            .ok()
            .map(|p| p.join("packages/dashboard")),
    ];

    candidates
        .into_iter()
        .flatten()
        .find(|candidate| candidate.join("package.json").exists())
}

fn ensure_node_modules(dashboard_dir: &std::path::Path) -> Result<()> {
    if dashboard_dir.join("node_modules").exists() {
        return Ok(());
    }

    println!(
        "  {} {}",
        "⟳".yellow(),
        "Installing dashboard dependencies...".bold()
    );

    let status = std::process::Command::new("npm")
        .args(["install"])
        .current_dir(dashboard_dir)
        .status()?;

    if !status.success() {
        anyhow::bail!(
            "npm install failed with exit code {}",
            status.code().unwrap_or(1)
        );
    }

    println!("  {} Dependencies installed.", "✓".green());
    Ok(())
}

fn read_pid(path: &PathBuf) -> Option<u32> {
    std::fs::read_to_string(path).ok()?.trim().parse().ok()
}

fn open_browser(url: &str) {
    let cmd = if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };

    let _ = std::process::Command::new(cmd)
        .arg(url)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}

fn is_process_alive(pid: u32) -> bool {
    std::process::Command::new("kill")
        .args(["-0", &pid.to_string()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}
