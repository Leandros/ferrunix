#![allow(clippy::exit)]
//! Xtask CI "emulation".

use anyhow::Result;
use xshell::{cmd, Shell};

/// How tests are run.
#[derive(Debug, Clone, clap::ValueEnum)]
pub(super) enum TestRunner {
    /// Use the default cargo test runner.
    Cargo,
    /// Use nextest, requires installing `cargo-nextest`.
    Nextest,
}

/// Arguments for `xtask ci`.
#[derive(Debug, Clone, clap::Args)]
pub(super) struct CiArgs {
    /// Which test runner to use
    #[arg(short, long)]
    testrunner: Option<TestRunner>,

    /// Whether to skip extended tests, e.g., `clippy`, `cargo outdated`, and
    /// `cargo-semver-checks`. Useful in CI, when these run as a separate task.
    #[arg(long, default_value_t = false)]
    no_extended: bool,
}

/// Run all tests, similar to the GitHub Actions in `ci.yml`.
pub(super) fn run(args: &CiArgs) -> Result<()> {
    let sh = Shell::new()?;
    // sh.set_var("RUSTFLAGS", "-Dwarnings");
    // sh.set_var("CARGO_INCREMENTAL", "0");
    // sh.set_var("CARGO_TERM_COLOR", "always");

    let has_cargo_outdated = if args.no_extended {
        false
    } else if let Err(err) = cmd!(sh, "cargo outdated --version").output() {
        eprintln!("failed to find `cargo-outdated`: {err}");
        eprintln!(
            "try installing it with: cargo install --locked cargo-outdated"
        );
        false
    } else {
        true
    };

    let has_cargo_semver = if args.no_extended {
        false
    } else if let Err(err) = cmd!(sh, "cargo semver-checks --version").output()
    {
        eprintln!("failed to find `cargo-semver-checks`: {err}");
        eprintln!(
            "try installing it with: cargo install --locked \
             cargo-semver-checks"
        );
        false
    } else {
        true
    };

    let test_matrix = [
        ("ferrunix", ""),
        ("ferrunix", "derive"),
        ("ferrunix", "multithread"),
        ("ferrunix", "tokio"),
        ("ferrunix", "derive,multithread"),
        ("ferrunix", "derive,tokio"),
        ("ferrunix-core", ""),
        ("ferrunix-core", "multithread"),
        ("ferrunix-core", "tokio"),
        ("ferrunix-macros", ""),
        ("ferrunix-macros", "multithread"),
        ("ferrunix-macros", "development"),
        ("ferrunix-macros", "development,multithread"),
    ];

    let testrunner: &[&str] = match args.testrunner {
        Some(TestRunner::Nextest) => &["nextest", "run"],
        None | Some(TestRunner::Cargo) => &["test"],
    };
    for (proj, features) in test_matrix {
        if features.is_empty() {
            cmd!(sh, "cargo {testrunner...} -p {proj} --no-default-features")
                .run()?;
            continue;
        }

        cmd!(
            sh,
            "cargo {testrunner...} -p {proj} --no-default-features -F {features}"
        )
        .run()?;
    }

    // cmd!(sh, "cargo test --all").run()?;

    if !args.no_extended && cmd!(sh, "cargo clippy --version").output().is_ok()
    {
        cmd!(sh, "cargo clippy --tests --workspace").run()?;
    }

    if !args.no_extended && has_cargo_outdated {
        cmd!(sh, "cargo outdated --workspace --exit-code 1").run()?;
    }

    if !args.no_extended && has_cargo_semver {
        cmd!(sh, "cargo semver-checks").run()?;
    }

    Ok(())
}
