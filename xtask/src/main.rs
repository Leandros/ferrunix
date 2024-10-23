//! Automation for the workspace.

use clap::{Parser, Subcommand};

mod ci;
mod docs;
mod publish;

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
    CI(ci::CiArgs),
    /// Publish all packages in the workspace to crates.io.
    Publish(publish::PublishArgs),
    /// Compile and package the documentation.
    Docs(docs::DocsArgs),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        CliCommands::CI(ref args) => ci::run(args)?,
        CliCommands::Publish(ref args) => publish::run(args)?,
        CliCommands::Docs(ref args) => docs::run(args)?,
    }

    Ok(())
}
