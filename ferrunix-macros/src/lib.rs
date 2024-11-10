//! Proc-macro crate for [`ferrunix`].
//!
//! See the [`derive_inject`] macro for documentation.
//!
//! [`ferrunix`]: https://crates.io/crates/ferrunix
#![allow(
    clippy::panic,
    clippy::min_ident_chars,
    clippy::module_name_repetitions,
    clippy::missing_docs_in_private_items
)]

use darling::FromDeriveInput;
use syn::{parse_macro_input, DeriveInput};

use self::attr::DeriveAttrInput;
use self::inject::derive_macro_impl;

mod attr;
mod inject;
mod utils;

/// `#[derive(Inject)]` proc-macro for [`ferrunix`].
///
/// The `Inject` derive macro supports the two following attributes:
///
/// - `#[provides]`: Customizing the object registration.
/// - `#[inject]`: Customizing how an injected member is created.
///
/// ```rust,ignore
/// #[derive(Inject)]
/// #[provides(PROPERTY...)]
/// struct Transient {
///     #[inject(PROPERTY...)]
///     field: UserType,
/// }
/// ```
///
/// ## `provides` Properties
///
/// - `transient [= "<TYPE-SIGNATURE>"]`
///     - The object is provided as a transient registered with `<TYPE-SIGNATURE>`
///       as key. If the signature is omitted, the concrete type is used as a key.
/// - `singleton [= "<TYPE-SIGNATURE>"]`
///     - The object is provided as a singleton registered with `<TYPE-SIGNATURE>`
///       as key. If the signature is omitted, the concrete type is used as a key.
/// - `ctor = <IDENTIFIER>`
///     - The object isn't constructed using member-wise construction, but it's
///       constructed using a custom constructor (e.g., `new`). The constructor
///       will be passed the members in order of declaration as parameters.
/// - `no_registration`
///     - The type isn't registered automatically and the generated
///       `Self::register(&ferrunix::Registry)` function needs to be called
///       manually to register the type.
///
/// ## `inject` Properties
///
/// - `default`
///     - Construct the field using the `Default` implementation.
/// - `ctor = "<RUST-CODE>"`
///     - Construct the field using the provided Rust code.
/// - `transient [= true]`
///     - Construct the field as a transient by retrieving it from the `Registry`.
/// - `singleton [= true]`
///     - Construct the field as a singleton by retrieving it from the `Registry`.
///
/// ```rust,ignore,no_run
/// # #![allow(unused)]
/// use ferrunix::Inject;
///
/// pub trait Logger {}
///
/// #[derive(Inject)]
/// #[provides(transient = "dyn Logger", no_registration)]
/// //                   ^^^^^^^^^^^^^^
/// // The explicit type can be omitted, if it matches the concrete type.
/// pub struct MyLogger {}
///
/// impl Logger for MyLogger {}
///
/// #[derive(Inject)]
/// #[provides(singleton, no_registration)]
/// pub struct MyConfig {
///     #[inject(default)]
///     // Use the `Default::default` impl for construction.
///     counter: u32,
///
///     #[inject(ctor = r#""log-prefix: ".to_owned()"#)]
///     //              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
///     // The constructor must be valid Rust code. For strings, two sets of quotes
///     // are required.
///     prefix: String,
///
///     #[inject(transient /* = true */)]
///     //                 ^^^^^^^^^^^^
///     // The long form with `= true` is optional.
///     logger: Box<dyn Logger>,
/// }
///
/// fn main() {
///     let registry = ferrunix::Registry::empty();
///     MyLogger::register(&registry);
///     MyConfig::register(&registry);
/// }
/// ```
///
/// [`ferrunix`]: https://crates.io/crates/ferrunix
#[proc_macro_derive(Inject, attributes(provides, inject))]
#[allow(clippy::missing_panics_doc)]
pub fn derive_inject(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    // eprintln!("input: {input:#?}");

    let attr_input =
        DeriveAttrInput::from_derive_input(&input).map_err(syn::Error::from);
    if let Err(err) = attr_input {
        return err.into_compile_error().into();
    }
    let attr_input = attr_input.expect("error is returned above");
    // eprintln!("attr_input: {attr_input:#?}");

    derive_macro_impl(&input, &attr_input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
