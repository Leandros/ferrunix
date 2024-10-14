//! Automation for the workspace.

use clap::{Parser, Subcommand};

mod ci;

/// All xtask CLI arguments.
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
#[command(next_line_help = true)]
#[command(propagate_version = true)]
#[command(arg_required_else_help(true))]
struct Cli {
    /// Available sub-commands.
    #[command(subcommand)]
    command: CliCommands,
}

/// All available sub-commands.
#[derive(Debug, Subcommand)]
enum CliCommands {
    /// Build the workspace, similar to how the CI would build it.
    CI,
}

fn main() -> anyhow:: Result<()> {
    let cli = Cli::parse();

    match cli.command {
        CliCommands::CI => ci::run()?,
    }

    Ok(())
}
