//! Build documentation.
use xshell::{cmd, Shell};

/// All arguments for the `xtask docs` command.
#[derive(Debug, clap::Args)]
pub(super) struct DocsArgs {}

/// Build the mdbook and docs.rs documentation, to be deployed somewhere.
pub(super) fn run(_args: &DocsArgs) -> anyhow::Result<()> {
    let sh = Shell::new()?;

    // Run tests for ferrunix; these validate all code in the book.
    cmd!(sh, "cargo test -p ferrunix").run()?;

    // Build the docs.
    let feature_matrix = ["default", "multithread", "tokio"];
    {
        for feature in feature_matrix {
            if feature == "default" {
                cmd!(sh, "cargo doc --target-dir target/doc-default --no-default-features").run()?;
                continue;
            }

            cmd!(sh, "cargo doc --target-dir target/doc-{feature} --no-default-features -F {feature}").run()?;
        }
    }

    // Build the book.
    {
        let _guard = sh.push_dir("book");
        cmd!(sh, "mdbook build .").run()?;
    }

    // Assemble.
    {
        cmd!(sh, "rm -rf target/tmp/deploy").run()?;
        cmd!(sh, "mkdir -p target/tmp/deploy").run()?;
        let _guard = sh.push_dir("target/tmp/deploy");

        for feature in feature_matrix {
            cmd!(sh, "cp -r ../../doc-{feature}/doc docs-{feature}").run()?;
        }

        cmd!(sh, "cp -r ../../../book/book/. .").run()?;
    }

    // Tar up.
    // {
    //     let _guard = sh.push_dir("target/tmp/deploy");
    //     cmd!(sh, "tar czf ../github-pages .").run()?;
    // }

    Ok(())
}
