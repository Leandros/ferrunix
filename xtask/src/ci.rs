//! Xtask CI "emulation".

use anyhow::Result;
use xshell::{cmd, Shell};

/// Run all tests, similar to the GitHub Actions in `ci.yml`.
pub(super) fn run() -> Result<()> {
    let sh = Shell::new()?;
    // sh.set_var("RUSTFLAGS", "-Dwarnings");
    // sh.set_var("CARGO_INCREMENTAL", "0");
    // sh.set_var("CARGO_TERM_COLOR", "always");

    cmd!(sh, "cargo test -p ferrunix --no-default-features").run()?;
    cmd!(sh, "cargo test -p ferrunix --no-default-features -F derive").run()?;
    cmd!(
        sh,
        "cargo test -p ferrunix --no-default-features -F derive,multithread"
    )
    .run()?;
    cmd!(sh, "cargo test -p ferrunix-core --no-default-features").run()?;
    cmd!(
        sh,
        "cargo test -p ferrunix-core --no-default-features -F multithread"
    )
    .run()?;
    cmd!(sh, "cargo test -p ferrunix-macros --no-default-features").run()?;
    cmd!(
        sh,
        "cargo test -p ferrunix-macros --no-default-features -F multithread"
    )
    .run()?;
    cmd!(sh, "cargo test -p ferrunix-macros --no-default-features -F multithread,development").run()?;
    cmd!(sh, "cargo test --all").run()?;
    cmd!(sh, "cargo clippy --tests --workspace").run()?;

    Ok(())
}
