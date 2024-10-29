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
#[allow(clippy::too_many_lines)]
pub(super) fn run(args: &CiArgs) -> Result<()> {
    let mut had_errors = false;
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
        ("ferrunix", "tracing"),
        ("ferrunix", "multithread,tracing"),
        ("ferrunix", "tokio,tracing"),
        ("ferrunix", "multithread,tokio,tracing"),
        ("ferrunix", "derive,multithread,tokio,tracing"),
        ("ferrunix-core", ""),
        ("ferrunix-core", "multithread"),
        ("ferrunix-core", "tokio"),
        ("ferrunix-core", "tracing"),
        ("ferrunix-macros", ""),
        ("ferrunix-macros", "multithread"),
        ("ferrunix-macros", "development"),
        ("ferrunix-macros", "development,multithread"),
        ("doc-tests", ""),
    ];

    let testrunner: &[&str] = match args.testrunner {
        Some(TestRunner::Nextest) => &["nextest", "run", "--profile", "ci"],
        None | Some(TestRunner::Cargo) => &["test"],
    };
    for (proj, features) in test_matrix {
        if features.is_empty() {
            let res = cmd!(
                sh,
                "cargo {testrunner...} -p {proj} --no-default-features"
            )
            .run();
            if res.is_err() {
                had_errors = true;
            }
            continue;
        }

        let res = cmd!(
            sh,
            "cargo {testrunner...} -p {proj} --no-default-features -F {features}"
        )
        .run();
        if res.is_err() {
            had_errors = true;
        }
    }

    if !args.no_extended && cmd!(sh, "cargo clippy --version").output().is_ok()
    {
        let res = cmd!(sh, "cargo clippy --tests --workspace").run();
        if res.is_err() {
            had_errors = true;
        }
    }

    if !args.no_extended && has_cargo_outdated {
        let res = cmd!(sh, "cargo outdated --workspace --exit-code 1").run();
        if res.is_err() {
            had_errors = true;
        }
    }

    if !args.no_extended && has_cargo_semver {
        let res = cmd!(sh, "cargo semver-checks").run();
        if res.is_err() {
            had_errors = true;
        }
    }

    if had_errors {
        return Err(anyhow::anyhow!("not all checks passed"));
    }

    Ok(())
}
