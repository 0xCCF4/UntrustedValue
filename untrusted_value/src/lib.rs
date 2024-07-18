//! # Untrusted Value
//! This crate aim to provide a type-safe way to handle and sanitize potentially untrusted values
//! like user input.
//!
//! It aims to provide compile-time [Taint checking](https://en.wikipedia.org/wiki/Taint_checking)
//! into Rust. All user input or in general all input coming from the outside
//! world into the program must be seen as untrusted and potentially malicious (called tainted).
//! A tainted value keeps its taint until a proper sanitation function is called
//! upon the tainted data, clearing its taint.
//!
//! This crate introduces several data types, traits and macros to simplify the process
//! of taint tracking.
//!
//! ## Example usage
//! User data must be wrapped within the container [UntrustedValue] which
//! provides marks the contained data as tainted.
//! ```rust
//! use untrusted_value::{UntrustedValue, SanitizeWith};
//! #
//! # let user_input: i32 = -36;
//! let user_input = UntrustedValue::from(user_input);
//!
//! let trusted_value: u32 = user_input.sanitize_with(
//! # |value| {
//! # Ok::<u32, ()>(value.unsigned_abs())
//! # }
//!     // ...
//! ).expect("Sanitization failed");
//!
//! println!("Sanitized value: {:?}", trusted_value);
//! ```
//!
//! When user data is a struct of different subtypes:
//!
//! ```rust
//! pub use untrusted_value::{IntoUntrustedVariant, SanitizeValue};
//! use untrusted_value::UntrustedValue;
//! pub use untrusted_value_derive::UntrustedVariant;
//!
//! use untrusted_value_derive_internals::SanitizeWith;
//!
//! #[derive(UntrustedVariant)]
//! #[untrusted_derive(Clone)] // tainted variant should be Cloneable
//! pub struct NetworkConfig {
//!     pub port: u32,
//!     pub listen_address: String,
//! }
//!
//! # fn no_sanitize<T>(value: T) -> Result<T, ()>{
//! #     Ok(value)
//! # }
//! #
//! impl SanitizeValue<NetworkConfig> for NetworkConfigUntrusted {
//!     type Error = ();
//!
//!     fn sanitize_value(self) -> Result<NetworkConfig, Self::Error> {
//!         Ok(NetworkConfig {
//!             port: self.port.sanitize_with(no_sanitize)?,
//!             listen_address: self.listen_address.sanitize_with(no_sanitize)?
//!         })
//!     }
//! }
//!
//! fn load_from_config() -> NetworkConfigUntrusted {
//!     let from_serde = NetworkConfig {
//!         port: 1111,
//!         listen_address: "0.0.0.0".into(),
//!     };
//!     from_serde.to_untrusted_variant()
//! }
//!
//! let user_data = load_from_config();
//!
//! // user data cannot be used on accident, since it is contained inside UntrustedValues
//!
//! let user_data_clean = user_data.sanitize_value();
//! ```
//!
//! When a function is called by an application framework like Rocket/Poem/...,
//! the macro `untrusted_inputs` may be used to taint the function inputs:
//!
//! ```rust
//! use untrusted_value_derive::untrusted_inputs;
//! use untrusted_value_derive_internals::SanitizeWith;
//! #
//! # fn no_sanitize<T>(value: T) -> Result<T, ()>{
//! #    Ok(value)
//! # }
//!
//! // #[route(path = "/"), method = "get"]
//! #[untrusted_inputs]
//! fn index(name: &str) -> Result<String, ()> {
//!     // MACRO inserts the following code:
//!         // let name = UntrustedValue::from(name);
//!         // let ... = UntrustedValue::from(...);
//!
//!     // we can not use "name" directly, since it is
//!     // wrapped in an UntrustedValue
//!
//!     // we must explicitly sanitize the value before usage
//!     let name = name.sanitize_with(no_sanitize)?;
//!     Ok(format!("Hello, {}!", name))
//! }
//! ```
//!
//! See also the examples in the `examples` directory.
//!
//! ## Installation
//! The library is written in Rust, and can be added using `cargo`:
//! ```bash
//! cargo add untrusted-value
//! ```
//!
//! ## Features
//! Enabled by default:
//!  * `allow_usage_without_sanitization`: enables the method `use_untrusted_value` to just use clear the taint of a value.
//!  * `derive`: enables the macros to automatically generate code (`#[derive(UntrustedVariant)`, `#[derive(SanitizeValue)`, `#[untrusted_inputs]`)
//!
//! Optional features:
//!  * `derive_harden_sanitize`: enables hardening for the derive macro `SanitizeValue`. When this feature is disabled, the
//!     implemented `fn sanitize_value(self)` errors-early. Which may be undesired if sanitizing timing side
//!     channels are a concern. When enabling this feature, first all sanitizers are run, then
//!     the first error is propagated.
//!
//! ## What's the goal of this crate?
//! The goal of this crate is to help design more secure applications. By tainting all
//! program inputs, unsanitized data can not be used by accident. By providing a sanitizing
//! interface to tainted data, security analysis can focus on analysing the implemented sanitizing functions
//! instead of identifying where tainted data is located, and where it is used.
//!
//! ## Limitations
//! Providing a taint tracking system is nice but still requires the developer to
//! taint the data properly. Currently, we are working on providing a crate level macro
//! to automatically check common taint source like input from environment variables, args, and
//! common frameworks, that will create a compile error if input data has not been tainted.
//!
//! This crate does only provide an interface to taint and sanitize data. Using this system, still this does
//! not make an application inherently secure. The developer must still implement
//! appropriate sanitizing functions to clear the taint of the data. This unified
//! interface should help to focus security analysis on the sanitizing functions
//! instead of on potentially all places where tainted data might be used.
//!
//! ## Contribution
//! Contributions to the project are welcome! If you have a feature request,
//! bug report, or want to contribute to the code, please open an
//! issue or a pull request.
#![warn(missing_docs)]

pub use untrusted_value_derive_internals::*;

mod untrusted_value;
pub use untrusted_value::*;

mod maybe_untrusted;
pub use maybe_untrusted::*;
