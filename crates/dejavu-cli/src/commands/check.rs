use anyhow::Result;
use std::path::PathBuf;

pub fn run(path: Option<String>, quiet: bool) -> Result<()> {
    let project_path = path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    let engine = dejavu_core::DejavuEngine::new()?;
    let conn = dejavu_core::db::open(&engine.db_path)?;

    let rules = dejavu_core::db::get_active_rules(&conn, &project_path.to_string_lossy())?;
    let proposed = rules.iter().filter(|r| r.status == "proposed").count();

    if quiet {
        let output = serde_json::json!({
            "pending_count": proposed,
            "total_rules": rules.len(),
        });
        println!("{}", serde_json::to_string(&output)?);
    } else {
        if proposed > 0 {
            println!("dejavu: {} new patterns detected. Run `/dejavu` to review.", proposed);
        }
    }

    Ok(())
}
