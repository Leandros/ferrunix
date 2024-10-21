#![allow(clippy::exit)]
//! Xtask CI "emulation".

use anyhow::Result;
use xshell::{cmd, Shell};

/// Run all tests, similar to the GitHub Actions in `ci.yml`.
pub(super) fn run() -> Result<()> {
    let sh = Shell::new()?;
    // sh.set_var("RUSTFLAGS", "-Dwarnings");
    // sh.set_var("CARGO_INCREMENTAL", "0");
    // sh.set_var("CARGO_TERM_COLOR", "always");

    let has_cargo_outdated =
        if let Err(err) = cmd!(sh, "cargo outdated --version").output() {
            eprintln!("failed to find `cargo-outdated`: {err}");
            eprintln!(
                "try installing it with: cargo install --locked cargo-outdated"
            );
            false
        } else {
            true
        };

    let has_cargo_semver =
        if let Err(err) = cmd!(sh, "cargo semver-checks --version").output() {
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

    let testrunner = &["nextest", "run"];
    // let testrunner = &["test"];
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
    cmd!(sh, "cargo clippy --tests --workspace").run()?;

    if has_cargo_outdated {
        cmd!(sh, "cargo outdated --workspace --exit-code 1").run()?;
    }

    if has_cargo_semver {
        cmd!(sh, "cargo semver-checks").run()?;
    }

    Ok(())
}
