use anyhow::{Context, Result};

use crate::config;
use crate::root;

pub fn cmd_delete(tag: &str) -> Result<()> {
    let cwd = std::env::current_dir().context("failed to read current directory")?;
    let project_root = root::discover_root(&cwd);
    let log_dir = config::resolve_log_dir(&project_root)?;
    let log_path = log_dir.join(format!("{tag}.log"));

    std::fs::remove_file(&log_path).with_context(|| {
        format!(
            "no log found for tag '{tag}' at {}",
            log_path.display()
        )
    })?;

    println!("deleted log for '{tag}'");
    Ok(())
}
