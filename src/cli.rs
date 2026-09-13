use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "sessionPeek",
    version,
    about = "Run and peek into tagged dev processes",
    long_about = "Run a tagged process (dev server, backend, db, ...) and query its \
                  structured log later, filtered, instead of reading raw terminal output."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run a tagged command, tee'ing output to the terminal and a structured log
    Run {
        /// Short name to identify this process later, e.g. "frontend" or "backend"
        #[arg(short = 't', long)]
        tag: String,
        /// The command to run, after `--`, e.g. `-- npm run dev`
        #[arg(last = true)]
        command: Vec<String>,
    },
    /// Query the structured log for a tag
    Log {
        /// Tag of the process whose log to read, as passed to `run -t`
        tag: String,
        /// Only show the last N matching lines
        #[arg(long)]
        tail: Option<usize>,
        /// Only show stdout lines (conflicts with --stderr-only)
        #[arg(long, conflicts_with = "stderr_only")]
        stdout_only: bool,
        /// Only show stderr lines
        #[arg(long)]
        stderr_only: bool,
        /// Only show lines whose content matches this regex
        #[arg(long)]
        grep: Option<String>,
        /// Only show lines newer than this, e.g. "30s", "10m", "2h"
        #[arg(long)]
        since: Option<String>,
    },
}
