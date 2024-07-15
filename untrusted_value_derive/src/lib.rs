extern crate proc_macro;

use proc_macro::TokenStream;

/// This macro can be used to annotate struct that contains data that
/// might be untrusted. The macro will generate a new struct that wraps
/// the original struct, but all fields will be wrapped in `untrusted_value::UntrustedValue`.
///
/// An instance of a struct like this:
/// ```rust
/// # use untrusted_value_derive::UntrustedVariant;
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
/// pub use untrusted_value_derive::UntrustedVariant;
///
/// #[derive(UntrustedVariant)]
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
#[proc_macro_derive(UntrustedVariant)]
pub fn untrusted_variant_derive(input: TokenStream) -> TokenStream {
    // Parse the string representation
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    untrusted_variant::impl_untrusted_variant(&ast)
}

mod untrusted_variant;
