use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::SystemTime;

use anyhow::{bail, Context, Result};

use crate::config;
use crate::log_format::{format_line, LogLine, Stream};
use crate::root;

pub fn cmd_run(tag: &str, command: &[String]) -> Result<i32> {
    let Some((program, args)) = command.split_first() else {
        bail!("no command given to run (usage: sessionPeek run -t <tag> -- <command...>)");
    };

    let cwd = std::env::current_dir().context("failed to read current directory")?;
    let project_root = root::discover_root(&cwd);
    let log_dir = config::resolve_log_dir(&project_root)?;
    std::fs::create_dir_all(&log_dir)
        .with_context(|| format!("failed to create log directory {}", log_dir.display()))?;
    let log_path = log_dir.join(format!("{tag}.log"));

    let mut log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .with_context(|| format!("failed to open log file {}", log_path.display()))?;

    let mut child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to spawn command: {program}"))?;

    let stdout = child.stdout.take().expect("child spawned with piped stdout");
    let stderr = child.stderr.take().expect("child spawned with piped stderr");

    let (tx, rx) = mpsc::channel::<(Stream, String)>();

    let tx_out = tx.clone();
    let stdout_thread = thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(|l| l.ok()) {
            if tx_out.send((Stream::Stdout, line)).is_err() {
                break;
            }
        }
    });

    let tx_err = tx.clone();
    let stderr_thread = thread::spawn(move || {
        for line in BufReader::new(stderr).lines().map_while(|l| l.ok()) {
            if tx_err.send((Stream::Stderr, line)).is_err() {
                break;
            }
        }
    });

    drop(tx);

    for (stream, content) in rx {
        match stream {
            Stream::Stdout => {
                let _ = writeln!(io::stdout(), "{content}");
            }
            Stream::Stderr => {
                let _ = writeln!(io::stderr(), "{content}");
            }
        }
        let line = LogLine {
            timestamp: SystemTime::now(),
            stream,
            content,
        };
        let _ = writeln!(log_file, "{}", format_line(&line));
        let _ = log_file.flush();
    }

    let _ = stdout_thread.join();
    let _ = stderr_thread.join();

    let status = child.wait().context("failed to wait for child process")?;
    Ok(status.code().unwrap_or(1))
}
