use std::fs::File;
use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::{Duration, SystemTime};

use anyhow::{bail, Context, Result};

use crate::ansi;
use crate::config;
use crate::line_assembler::LineAssembler;
use crate::log_format::{format_line, LogLine, Stream};
use crate::root;

const IDLE_FLUSH: Duration = Duration::from_millis(200);

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

    let (tx, rx) = mpsc::channel::<(Stream, Vec<u8>)>();

    let tx_out = tx.clone();
    let stdout_thread = thread::spawn(move || read_chunks(stdout, Stream::Stdout, tx_out));

    let tx_err = tx.clone();
    let stderr_thread = thread::spawn(move || read_chunks(stderr, Stream::Stderr, tx_err));

    drop(tx);

    let mut stdout_asm = LineAssembler::default();
    let mut stderr_asm = LineAssembler::default();

    loop {
        match rx.recv_timeout(IDLE_FLUSH) {
            Ok((stream, chunk)) => {
                let asm = match stream {
                    Stream::Stdout => &mut stdout_asm,
                    Stream::Stderr => &mut stderr_asm,
                };
                asm.feed(&chunk, |line| emit_line(stream, line, &mut log_file));
            }
            Err(RecvTimeoutError::Timeout) => {
                if let Some(line) = stdout_asm.flush_pending() {
                    emit_line(Stream::Stdout, &line, &mut log_file);
                }
                if let Some(line) = stderr_asm.flush_pending() {
                    emit_line(Stream::Stderr, &line, &mut log_file);
                }
            }
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }

    if let Some(line) = stdout_asm.flush_pending() {
        emit_line(Stream::Stdout, &line, &mut log_file);
    }
    if let Some(line) = stderr_asm.flush_pending() {
        emit_line(Stream::Stderr, &line, &mut log_file);
    }

    let _ = stdout_thread.join();
    let _ = stderr_thread.join();

    let status = child.wait().context("failed to wait for child process")?;
    Ok(status.code().unwrap_or(1))
}

fn read_chunks(mut source: impl Read, stream: Stream, tx: mpsc::Sender<(Stream, Vec<u8>)>) {
    let mut buf = [0u8; 8192];
    loop {
        match source.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                if tx.send((stream, buf[..n].to_vec())).is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}

fn emit_line(stream: Stream, content: &[u8], log_file: &mut File) {
    let text = String::from_utf8_lossy(content);

    match stream {
        Stream::Stdout => {
            let _ = writeln!(io::stdout(), "{text}");
        }
        Stream::Stderr => {
            let _ = writeln!(io::stderr(), "{text}");
        }
    }

    let line = LogLine {
        timestamp: SystemTime::now(),
        stream,
        content: ansi::strip(&text).into_owned(),
    };
    let _ = writeln!(log_file, "{}", format_line(&line));
    let _ = log_file.flush();
}
