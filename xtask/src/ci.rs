#![allow(clippy::exit)]
//! Xtask CI "emulation".

use std::iter;

use anyhow::Result;
use itertools::Itertools;
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

    /// Which features to **not** test.
    #[arg(short, long)]
    filter: Option<Vec<String>>,

    /// Whether to skip extended tests, e.g., `clippy`, `cargo outdated`, and
    /// `cargo-semver-checks`. Useful in CI, when these run as a separate task.
    #[arg(long, default_value_t = false)]
    no_extended: bool,
}

/// Run all tests, similar to the GitHub Actions in `ci.yml`.
#[allow(clippy::too_many_lines)]
pub(super) fn run(args: &CiArgs) -> Result<()> {
    let mut errors: Vec<xshell::Error> = Vec::new();
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

    let combinations_ferrunix =
        feature_combinations(&["derive", "multithread", "tokio", "tracing"]);
    let combinations_ferrunix_core =
        feature_combinations(&["multithread", "tokio", "tracing"]);
    let combinations_ferrunix_macros =
        feature_combinations(&["multithread", "development"]);
    let test_matrix = {
        let ferrunix = iter::repeat("ferrunix").zip(combinations_ferrunix);
        let ferrunix_core =
            iter::repeat("ferrunix-core").zip(combinations_ferrunix_core);
        let ferrunix_macros =
            iter::repeat("ferrunix-macros").zip(combinations_ferrunix_macros);

        let filter = args.filter.clone().unwrap_or_default();
        ferrunix
            .chain(ferrunix_core)
            .chain(ferrunix_macros)
            .filter(|(_, features)| {
                if filter.is_empty() {
                    true
                } else {
                    filter.iter().any(|filtered_feature| {
                        !features.contains(filtered_feature)
                    })
                }
            })
            .collect_vec()
    };

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
            if let Err(err) = res {
                errors.push(err);
            }
            continue;
        }

        let res = cmd!(
            sh,
            "cargo {testrunner...} -p {proj} --no-default-features -F {features}"
        )
        .run();
        if let Err(err) = res {
            errors.push(err);
        }
    }

    {
        cmd!(
            sh,
            "cargo clean -p ferrunix -p ferrunix-core -p ferrunix-macros"
        )
        .run()?;
        let res = cmd!(sh, "cargo {testrunner...} -p doc-tests").run();
        if let Err(err) = res {
            errors.push(err);
        }
    }

    if !args.no_extended && cmd!(sh, "cargo clippy --version").output().is_ok()
    {
        let res = cmd!(sh, "cargo clippy --tests --workspace").run();
        if let Err(err) = res {
            errors.push(err);
        }
    }

    if !args.no_extended && has_cargo_outdated {
        let res = cmd!(sh, "cargo outdated --workspace").run();
        if let Err(err) = res {
            errors.push(err);
        }
    }

    if !args.no_extended && has_cargo_semver {
        let res = cmd!(sh, "cargo semver-checks").run();
        if let Err(err) = res {
            errors.push(err);
        }
    }

    if !errors.is_empty() {
        return Err(anyhow::anyhow!("not all checks passed: {errors:#?}"));
    }

    Ok(())
}

/// Generate all possible combinations of `features`.
fn feature_combinations(features: &[&str]) -> Vec<String> {
    let len = features.len();
    let base_iter = features.iter().combinations(len);

    let mut chains: Vec<Box<dyn Iterator<Item = Vec<&&str>>>> =
        Vec::with_capacity(len);
    for num_combinations in (1..len).rev() {
        let iter = base_iter
            .clone()
            .chain(features.iter().combinations(num_combinations));
        chains.push(Box::new(iter));
    }

    let mut ret = chains
        .into_iter()
        .map(Itertools::collect_vec)
        .flat_map(|outer| {
            outer
                .into_iter()
                .map(|el| {
                    let x = el.iter().join(",");
                    x
                })
                .collect_vec()
        })
        .unique()
        .collect::<Vec<_>>();

    ret.push(String::new());

    ret
}
