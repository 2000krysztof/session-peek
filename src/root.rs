use std::path::{Path, PathBuf};

/// Presence of this file both pins the project root explicitly and doubles
/// as the per-project config file (see `config.rs`).
pub const PROJECT_CONFIG_FILE: &str = ".sessionpeek.json";

const MARKERS: &[&str] = &[
    ".git",
    ".hg",
    ".svn",
    "Cargo.toml",
    "package.json",
    "pyproject.toml",
    "go.mod",
    "pom.xml",
    "build.gradle",
    "Gemfile",
    "composer.json",
    "Makefile",
];

/// Walks upward from `start` looking for a project root marker. Checks the
/// explicit `.sessionpeek.json` pin first, then a broader fallback marker
/// list. Falls back to `start` itself if nothing is found, so a one-off
/// script with no VCS or manifest still works.
pub fn discover_root(start: &Path) -> PathBuf {
    let mut current = start;
    loop {
        if current.join(PROJECT_CONFIG_FILE).is_file() {
            return current.to_path_buf();
        }
        if MARKERS.iter().any(|marker| current.join(marker).exists()) {
            return current.to_path_buf();
        }
        match current.parent() {
            Some(parent) => current = parent,
            None => return start.to_path_buf(),
        }
    }
}
