//! Publish all packages to `crates.io`.

use anyhow::Result;
use xshell::{cmd, Shell};

/// Arguments for `xtask publish ...`.
#[derive(Debug, Default, clap::Args)]
pub struct PublishArgs {
    /// Perform all checks without uploading.
    #[arg(short = 'n', long)]
    dry_run: bool,

    /// Whether to skip the `publish` tasks from cargo.
    #[arg(long)]
    no_publish: bool,
}

/// Invoked when `xtask publish` is called.
pub(super) fn run(args: &PublishArgs) -> Result<()> {
    let sh = Shell::new()?;
    let is_dry_run = args.dry_run.then_some("--dry-run");

    let out = cmd!(sh, "git cliff -c cliff.toml --bump --latest").output()?;
    let mut changelog = String::from_utf8(out.stdout)?;
    let existing_changelog = sh.read_file("./CHANGELOG.md")?;
    changelog.push_str("\n\n");
    changelog.push_str(&existing_changelog);
    sh.write_file("./CHANGELOG.md", changelog)?;

    cmd!(sh, "git add ./CHANGELOG.md").run()?;
    cmd!(sh, "git commit -m 'chore: update changelog'").run()?;

    if !args.no_publish {
        cmd!(sh, "cargo publish -p ferrunix-core {is_dry_run...}").run()?;
        cmd!(sh, "cargo publish -p ferrunix-macros {is_dry_run...}").run()?;
        cmd!(sh, "cargo publish -p ferrunix {is_dry_run...}").run()?;
    }

    Ok(())
}
