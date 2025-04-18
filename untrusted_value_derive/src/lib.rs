//! This crate contains the proc macro definitions for the [untrusted_value](https://docs.rs/untrusted_value/latest/untrusted_value/)
//! crate.
//!
//! All proc macros are reexported in the [untrusted_value](https://docs.rs/untrusted_value/latest/untrusted_value/) crate, so
//! you should properly use that crate instead.
//!
//! See also the main repo at [https://github.com/0xCCF4/UntrustedValue](https://github.com/0xCCF4/UntrustedValue).
#![warn(missing_docs)]

extern crate proc_macro;

use crate::require_tainting::TaintChecker;
use proc_macro::TokenStream;
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Data, Field, Fields};

/// This macro can be used to annotate struct that contains data that
/// might be untrusted.
///
/// The macro will generate a new struct that resembles
/// the original struct, but all fields will be wrapped in `untrusted_value::UntrustedValue`.
///
/// An instance of a struct like this:
/// ```rust
/// # use untrusted_value::derive::UntrustedVariant;
/// use untrusted_value::{IntoUntrustedVariant, SanitizeWith};
///
/// #[derive(UntrustedVariant)]
/// pub struct Example {
///    pub name: String,
/// }
/// ```
/// will create a new struct like this:
/// ```rust
/// # use untrusted_value::UntrustedValue;
/// pub struct ExampleUntrusted {
///   pub name: UntrustedValue<String>,
/// }
/// ```
///
/// The following traits are implemented for the new struct:
/// - `untrusted_value::IntoUntrustedVariant` to convert the struct into the untrusted variant of it
/// - `untrusted_value::SanitizeWith` to sanitize the untrusted variant using a provided sanitizer to its original form
///
/// This proc macro supports the following attributes:
/// - `#[untrusted_derive(...)]` to implement derive macros for the untrusted variant struct
///
/// # Example
/// Image the situation where a struct is read from a configuration file using Serde.
/// The struct will look like this
/// ```compile_fail
/// #[derive(Serialize, Deserialize)]
/// pub struct NetworkConfig {
///   pub port: u32,
///   pub listen_address: String,
/// }
/// ```
/// When the struct is read from the configuration file, it is values are untrusted.
/// ```rust
/// pub use untrusted_value::{IntoUntrustedVariant, SanitizeWith};
/// pub use untrusted_value::derive::UntrustedVariant;
///
/// #[derive(UntrustedVariant, Clone, Debug)] // safe version: is cloneable and debuggable
/// #[untrusted_derive(Clone)] // untrusted version: is cloneable, but not debuggable!
/// pub struct NetworkConfig {
///   pub port: u32,
///   pub listen_address: String,
/// }
///
/// fn sanitize_ip_address(address: String) -> Result<String, ()> {
///     // somehow sanitize the address
/// #   Ok(address)
/// }
///
/// fn sanitize_port(port: u32) -> Result<u32, ()> {
///     // somehow sanitize the port
/// #   Ok(port)
/// }
///
/// fn load_from_config() -> NetworkConfig {
/// //   ...
/// #    NetworkConfig {
/// #        port: 1111,
/// #        listen_address: "0.0.0.0".into(),
/// #    }
/// }
///
/// let user_data = load_from_config().to_untrusted_variant();
///
/// // user data cannot be used on accident, since it is contained inside UntrustedValues
///
/// let user_data_clean = user_data
///         .sanitize_with(|value| {
///             Ok::<NetworkConfig, ()>(NetworkConfig {
///                 port: value
///                     .port
///                     .sanitize_with(sanitize_port)?,
///                 listen_address: value
///                     .listen_address
///                     .sanitize_with(sanitize_ip_address)?
///             })
///         })
///         .expect("Sanitization failed");
/// ```
///
/// This macro can be combined with the `SanitizeValue` macro to automatically implement the `SanitizeValue` trait.
/// When there are no sub-structs that also implement the `SanitizeValue` trait: One may
/// use the `#[untrusted_derive(SanitizeValueEnd)]` attribute to implement a wrapper that maps the
/// call to `UntrustedValue<Struct>::sanitize_value()` to the call `StructUntrusted::sanitize_value()`.
///
/// For example:
/// ```rust
/// # use untrusted_value::{IntoUntrustedVariant, SanitizeValue};
/// # use untrusted_value::derive::UntrustedVariant;
/// #
/// #[derive(Debug, UntrustedVariant)]
/// #[untrusted_derive(Clone, SanitizeValue)]
/// pub struct GeneralConfig {
///     pub network: NetworkConfig,
///     // other fields
/// }
///
/// #[derive(Clone, Debug, UntrustedVariant)] // <-- Implements `NetworkConfigUntrusted`
/// #[untrusted_derive(Clone, SanitizeValueEnd)]
/// pub struct NetworkConfig {
///     pub port: u32,
///     pub listen_address: String,
/// }
///
/// // Implement the `SanitizeValue` trait for the tainted version of `NetworkConfig`/`NetworkConfigUntrusted`
/// // This is made possible by the `#[untrusted_derive(SanitizeValueEnd)]` attribute. Otherwise, you would have to
/// // implement the `SanitizeValue` on the `UntrustedValue<NetworkConfig>` type. Using the `SanitizeValueEnd` attribute
/// // this becomes more convenient/readable.
/// impl SanitizeValue<NetworkConfig> for NetworkConfigUntrusted {
///     # type Error = ();
///     #
///     fn sanitize_value(self) -> Result<NetworkConfig, Self::Error> {
///         Ok(NetworkConfig {
///             port: self.port.use_untrusted_value(),
///             listen_address: self.listen_address.use_untrusted_value(),
///         })
///     }
/// }
/// ```
///
/// # Panics
/// This macro will panic if the annotated struct is not valid Rust code.
#[proc_macro_derive(UntrustedVariant, attributes(untrusted_derive))]
pub fn untrusted_variant_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    untrusted_variant::impl_untrusted_variant_macro(&ast).into()
}

/// This macro can be used to annotate structs and automatically implement
/// the `untrusted_value::SanitizeValue` trait.
///
/// The implementation implements the `SanitizeValue` trait on `UntrustedValue<AnnotatedType>`.
/// Struct member types are required to implement the `SanitizeValue(MemberType)` trait.
///
/// When using the `derive_harden_sanitize` feature first all sanitizer functions
/// are called. Then the (first) error (if any) is propagated.
/// If the flag is not present, the sanitizers are called sequentially and the first
/// error is propagated directly.
///
/// # Panics
/// This macro will panic if the annotated struct is not valid Rust code.
#[proc_macro_derive(SanitizeValue)]
pub fn sanitize_value_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    sanitize_value::impl_sanitize_value_macro(&ast).into()
}

/// This macro can be used to annotate functions to automatically wrap the
/// function arguments as `UntrustedValue<ArgType>`.
///
/// A function with the following signature:
/// ```rust
/// # use untrusted_value::derive::untrusted_inputs;
/// #
/// #[untrusted_inputs]
/// fn index(name: &str) -> () {
///     // some logic
/// #   ()
/// }
/// ```
/// Will be converted into:
/// ```rust
/// # use untrusted_value::UntrustedValue;
/// # use untrusted_value::derive::untrusted_inputs;
/// #
/// fn index(name: &str) -> () {
///     let name = UntrustedValue::from(name);
///     // some logic
/// #   ()
/// }
/// ```
///
/// This prevents the logic from using the tainted function inputs without proper sanitization.
///
/// This macro should be put at any binary entry points. Like when using a webserver the functions
/// handling a specific web request.
///
/// Note that: This macro will generate a compile error if a function argument is marked
/// as mutable. Since an `UntrustedValue` can not be mutable.
#[proc_macro_attribute]
pub fn untrusted_inputs(_attr: TokenStream, item: TokenStream) -> TokenStream {
    untrusted_inputs::impl_untrusted_inputs_macro(item.into()).into()
}

/// This macro can be used to annotate functions to automatically wrap the
/// function output as `UntrustedValue<ReturnType>`.
///
/// A function with the following signature:
/// ```rust
/// # use untrusted_value::derive::untrusted_output;
/// #
/// #[untrusted_output]
/// fn index() -> String {
///    "Hello World".to_string()
/// }
/// ```
///
/// Will be converted into:
/// ```rust
/// # use untrusted_value::UntrustedValue;
/// #
/// fn index() -> UntrustedValue<String> {
///     UntrustedValue::from(
///         // original function body
///         // ...
///         # "Hello World".to_string()
///     )
/// }
/// ```
///
/// This macro is useful when one writes a library function that provides untrusted data.
/// Since a library user might not want to use the untrusted data crate. That function can
/// be annotated with
/// ```rust
/// # use untrusted_value::derive::untrusted_output;
/// #
/// #[cfg_attr(feature = "some_feature", untrusted_output)]
/// fn some_lib_func() -> String {
///     "abcdef".to_string()
/// }
/// ```
/// such that a normal library user can use the function without caring about tainted data.
/// When enabling the feature `some_feature` the function output is wrapped in `UntrustedValue`
/// and marked as tainted.
#[proc_macro_attribute]
pub fn untrusted_output(_attr: TokenStream, item: TokenStream) -> TokenStream {
    untrusted_output::impl_untrusted_output_macro(item.into()).into()
}

/// This macro can be used to annotate modules/functions/blocks.
/// It will check if it finds any known patterns where tainting should be applied.
/// If the developer did not apply tainting a compile error is thrown.
///
/// This macro does not propagate taint from a pattern further than
/// the next access to the variable where the result of a found pattern
/// is stored.
///
/// This macro does not modify the source code.
///
/// Currently, the macro does nothing, this might change in the future.
/// Feel free to implement and pull request checkers.
///
/// # Examples
/// ```ignore
/// #[require_taint]
/// fn test() {
///     let var = std::env::args().next();
///         // ...
///         // Macro would have expected, something like
///         //  let var = UntrustedValue::from(var);
///         // or
///         //  let var: UntrustedValue<...> = var.into()
///         // ...
///     println!("{}", var); // <-- Macro will raise a compile error here
/// }
/// ```
///
/// # Ignore a found taint pattern
/// Annotate the taint generating block/function/pattern with
/// the macro `#[ignore_tainting]`
///
/// # Limitations
/// This macro does not guarantee that all taint sources are identified.
///
/// # Panics
/// This macro will panic if the input is not a valid Rust token.
// #[proc_macro_attribute]
#[allow(clippy::needless_pass_by_value)]
#[allow(dead_code)]
fn require_tainting(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast: syn::File = syn::parse(item).expect("Failed to parse input");
    let checker = TaintChecker::default();

    checker.process_file(&ast);

    ast.into_token_stream().into()
}

/// Ignores the found taint source pattern. See the macro `#[require_tainting]`.
// #[proc_macro_attribute]
#[allow(clippy::needless_pass_by_value)]
#[allow(dead_code)]
fn ignore_tainting(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

fn extract_struct_fields_from_ast(ast: &syn::DeriveInput) -> &Punctuated<Field, Comma> {
    match &ast.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => &fields_named.named,
            Fields::Unnamed(fields_unnamed) => &fields_unnamed.unnamed,
            Fields::Unit => panic!("Unit structs are not supported"),
        },
        _ => panic!("Only structs are supported"),
    }
}

#[allow(clippy::module_name_repetitions)]
mod require_tainting;
#[allow(clippy::module_name_repetitions)]
mod sanitize_value;
#[allow(clippy::module_name_repetitions)]
mod sanitize_with;
#[allow(clippy::module_name_repetitions)]
mod untrusted_inputs;
#[allow(clippy::module_name_repetitions)]
mod untrusted_output;
#[allow(clippy::module_name_repetitions)]
mod untrusted_variant;
