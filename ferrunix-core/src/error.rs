//! All errors that might happen.
#![allow(clippy::module_name_repetitions)]

use thiserror::Error;

/// Errors happening during resolving of lazy types.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ResolveError {
    /// The lock for the inner value couldn't be acquired.
    #[error("lock couldn't be acquired")]
    LockAcquire,
    /// Some of the required dependencies are missing.
    #[error("couldn't resolve dependencies")]
    DependenciesMissing,
}
