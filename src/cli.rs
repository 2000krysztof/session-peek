use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "sessionPeek", about = "Run and peek into tagged dev processes")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run a tagged command, tee'ing output to the terminal and a structured log
    Run {
        #[arg(short = 't', long)]
        tag: String,
        #[arg(last = true)]
        command: Vec<String>,
    },
    /// Query the structured log for a tag
    Log {
        tag: String,
        #[arg(long)]
        tail: Option<usize>,
        #[arg(long, conflicts_with = "stderr_only")]
        stdout_only: bool,
        #[arg(long)]
        stderr_only: bool,
        #[arg(long)]
        grep: Option<String>,
        #[arg(long)]
        since: Option<String>,
    },
}
