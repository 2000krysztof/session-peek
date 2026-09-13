mod cli;
mod config;
mod log_format;
mod query;
mod root;
mod runner;

use clap::Parser;
use cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse();

    let outcome = match cli.command {
        Commands::Run { tag, command } => runner::cmd_run(&tag, &command).map(Some),
        Commands::Log {
            tag,
            tail,
            stdout_only,
            stderr_only,
            grep,
            since,
        } => query::cmd_log(
            &tag,
            tail,
            stdout_only,
            stderr_only,
            grep.as_deref(),
            since.as_deref(),
        )
        .map(|_| None),
    };

    match outcome {
        Ok(Some(code)) => std::process::exit(code),
        Ok(None) => {}
        Err(err) => {
            eprintln!("sessionPeek: error: {err:#}");
            std::process::exit(1);
        }
    }
}
