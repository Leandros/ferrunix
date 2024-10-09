//! Utilities for auto-registration of types into the global registry.
//!
//! The global registry is available via [`Registry::global`].

use crate::{types::OnceCell, Registry};

/// The global, `'static` default [`Registry`]. It's constructed and accessible via
/// [`Registry::global`].
pub(crate) static DEFAULT_REGISTRY: OnceCell<Registry> = OnceCell::new();

/// All auto-registration functions need to use this type for registration.
///
/// This is, usually, used by the derive macro, and not manually.
#[non_exhaustive]
#[allow(missing_debug_implementations)]
pub struct RegistrationFunc(pub fn(&Registry));

// Create a new inventory for the auto-registration.
inventory::collect!(RegistrationFunc);

/// Use `autoregister` to register a new [`RegistrationFunc`].
pub use inventory::submit as autoregister;
