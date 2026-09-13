use std::fs;
use std::time::SystemTime;

use anyhow::{Context, Result};
use regex::Regex;

use crate::config;
use crate::log_format::{format_line, parse_line, Stream};
use crate::root;

#[allow(clippy::too_many_arguments)]
pub fn cmd_log(
    tag: &str,
    tail: Option<usize>,
    stdout_only: bool,
    stderr_only: bool,
    grep: Option<&str>,
    since: Option<&str>,
) -> Result<()> {
    let cwd = std::env::current_dir().context("failed to read current directory")?;
    let project_root = root::discover_root(&cwd);
    let log_dir = config::resolve_log_dir(&project_root)?;
    let log_path = log_dir.join(format!("{tag}.log"));

    let contents = fs::read_to_string(&log_path).with_context(|| {
        format!(
            "no log found for tag '{tag}' at {} (has it been started with `sessionPeek run -t {tag} -- ...`?)",
            log_path.display()
        )
    })?;

    let regex = grep
        .map(Regex::new)
        .transpose()
        .context("invalid --grep pattern")?;

    let since_cutoff = since
        .map(|s| -> Result<SystemTime> {
            let dur = humantime::parse_duration(s).context("invalid --since duration")?;
            Ok(SystemTime::now()
                .checked_sub(dur)
                .unwrap_or(SystemTime::UNIX_EPOCH))
        })
        .transpose()?;

    let mut lines: Vec<_> = contents.lines().filter_map(parse_line).collect();

    if stdout_only {
        lines.retain(|l| l.stream == Stream::Stdout);
    } else if stderr_only {
        lines.retain(|l| l.stream == Stream::Stderr);
    }

    if let Some(re) = &regex {
        lines.retain(|l| re.is_match(&l.content));
    }

    if let Some(cutoff) = since_cutoff {
        lines.retain(|l| l.timestamp >= cutoff);
    }

    if let Some(n) = tail {
        let start = lines.len().saturating_sub(n);
        lines = lines.split_off(start);
    }

    for line in &lines {
        println!("{}", format_line(line));
    }

    Ok(())
}
