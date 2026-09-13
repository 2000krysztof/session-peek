use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    pub log_dir: Option<PathBuf>,
}

fn read_config(path: &Path) -> Result<Option<Config>> {
    match std::fs::read_to_string(path) {
        Ok(contents) => {
            let cfg: Config = serde_json::from_str(&contents)
                .with_context(|| format!("failed to parse config at {}", path.display()))?;
            Ok(Some(cfg))
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => {
            Err(err).with_context(|| format!("failed to read config at {}", path.display()))
        }
    }
}

/// Highest priority first: per-project `.sessionpeek.json` at the discovered
/// root, then the global `~/.config/sessionPeek/config.json`, then defaults.
pub fn load(root: &Path) -> Result<Config> {
    if let Some(cfg) = read_config(&root.join(crate::root::PROJECT_CONFIG_FILE))? {
        return Ok(cfg);
    }
    if let Some(config_dir) = dirs::config_dir() {
        if let Some(cfg) = read_config(&config_dir.join("sessionPeek").join("config.json"))? {
            return Ok(cfg);
        }
    }
    Ok(Config::default())
}

pub fn resolve_log_dir(root: &Path) -> Result<PathBuf> {
    let cfg = load(root)?;
    Ok(cfg
        .log_dir
        .unwrap_or_else(|| root.join(".cache").join("sessionpeek")))
}
