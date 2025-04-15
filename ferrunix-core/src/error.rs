//! All errors that might happen.
#![allow(clippy::module_name_repetitions)]

use thiserror::Error;

use crate::cycle_detection::ValidationError;
use crate::types::BoxErr;

/// Errors happening during resolving of types.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ResolveError {
    /// Some of the required dependencies are missing.
    #[error("missing dependencies")]
    DependenciesMissing,

    /// Validation failed. Check the underlying error.
    #[error("validation failed: {0}")]
    Validation(#[from] ValidationError),

    /// Type is missing, possibly never registered.
    #[error("type missing: {typename}")]
    TypeMissing {
        /// Name of the type that was attempted to be resolved.
        typename: &'static str,
    },

    /// The constructor returned an error.
    #[error("error from constructor: {0}")]
    Ctor(BoxErr),

    /// An error in the implementation happened.
    #[error("implementation error: {0}")]
    Impl(#[from] ImplErrors),
}

/// Implementation errors. These errors represent internal errors that are usually
/// the fault of the people implementing `ferrunix`, not the users.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ImplErrors {
    /// The type returned by the constructor is different from the type that we cast it to.
    #[error("type mismatch")]
    TypeMismatch,
}

impl ResolveError {
    /// Returns true if the underlying error was produced by a failure in one of the constructors
    /// registered.
    pub fn is_ctor_err(&self) -> bool {
        matches!(self, Self::Ctor(_))
    }

    /// Returns the underlying error that was produced by the failure in one of the registered
    /// constructors, or `None` if it's another error.
    pub fn ctor_err(&self) -> Option<&BoxErr> {
        match self {
            Self::Ctor(error) => Some(error),
            _ => None,
        }
    }
}
