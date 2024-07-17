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
//!  * `derive`: enables the macros to automatically generate code (`#[derive(UntrustedVariant)`, `#[derive(SanitizeValue)`)
//!
//! Optional features:
//!  * `derive_harden_sanitize`: enables hardening for the derive macro `SanitizeValue`. When this feature is disabled, the
//!     implemented `fn sanitize_value(self)` errors-early. Which may be undesired if sanitizing timing side
//!     channels are a concern. When enabling this feature, first all sanitizers are run, then
//!     the first error is propagated.
//!
//! ## Limitations
//! Providing a taint tracking system is nice but still requires the developer to
//! taint the data properly. Currently, we are working on providing a crate level macro
//! to automatically check common taint source like input from environment variables, args, and
//! common frameworks, that will create a compile error if input data has not been tainted.
//!
//! ## Contribution
//! Contributions to the project are welcome! If you have a feature request,
//! bug report, or want to contribute to the code, please open an
//! issue or a pull request.
#![warn(missing_docs)]

pub use untrusted_value_derive_internals::*;

/// Represents an untrusted/untrustworthy value.
///
/// An attacker might be able to control (part) of the returned value.
/// Take special care processing this data.
///
/// See the method documentation of the function returning this value
pub struct UntrustedValue<Insecure> {
    value: Insecure,
}

impl<Insecure> UntrustedValue<Insecure> {
    /// Be sure that you carefully handle the returned value since
    /// it may be controllable by a malicious actor.
    ///
    /// See the method documentation of the function returning this value
    #[cfg(feature = "allow_usage_without_sanitization")]
    pub fn use_untrusted_value(self) -> Insecure {
        self.value
    }

    /// Wraps the provided value as [UntrustedValue]
    pub fn wrap(value: Insecure) -> Self {
        UntrustedValue { value }
    }
}

// does explicitly not implement Debug, Display, etc. to avoid processing untrusted data
// if desired, implement these traits manually for UntrustedValue<SomeCustomType>

impl<Insecure, Trusted> SanitizeWith<Insecure, Trusted> for UntrustedValue<Insecure> {
    /// Sanitizes the value using the provided sanitizer.
    ///
    /// The sanitizer may transmute the value to a different type.
    /// If sanitization fails, an error must be returned.
    fn sanitize_with<Sanitizer, Error>(self, sanitizer: Sanitizer) -> Result<Trusted, Error>
    where
        Sanitizer: FnOnce(Insecure) -> Result<Trusted, Error>,
    {
        sanitizer(self.value)
    }
}

impl<Insecure> From<Insecure> for UntrustedValue<Insecure> {
    /// Wraps the provided value as [UntrustedValue]
    fn from(value: Insecure) -> Self {
        UntrustedValue::wrap(value)
    }
}

impl<Insecure: Clone> Clone for UntrustedValue<Insecure> {
    /// Clones the value
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
        }
    }
}

impl<Sanitized, E, Insecure: SanitizeValue<Sanitized, Error = E>> SanitizeValue<Sanitized>
    for UntrustedValue<Insecure>
{
    type Error = E;

    /// Sanitizes the value.
    ///
    /// The returned value is sanitized and can be safely used.
    /// If the value cannot be sanitized, an error must be returned.
    fn sanitize_value(self) -> Result<Sanitized, Self::Error> {
        self.value.sanitize_value()
    }
}

impl<Insecure: Copy> Copy for UntrustedValue<Insecure> {}

/// Represents a value that might be untrusted. See UntrustedValue for more information.
pub enum MaybeUntrusted<Insecure, Trusted = Insecure> {
    /// Trusted value variant
    Ok(Trusted),
    /// Untrusted value variant
    Untrusted(UntrustedValue<Insecure>),
}

impl<Insecure> MaybeUntrusted<Insecure, Insecure> {
    /// Be sure that you carefully handle the returned value since
    /// it may be controllable by a malicious actor (when it is a MaybeUntrusted::Untrusted).
    ///
    /// See the method documentation of the function returning this value
    #[cfg(feature = "allow_usage_without_sanitization")]
    pub fn use_untrusted_value(self) -> Insecure {
        match self {
            MaybeUntrusted::Ok(value) => value,
            MaybeUntrusted::Untrusted(value) => value.use_untrusted_value(),
        }
    }

    /// Wraps the provided value as maybe untrusted, according to given boolean
    pub fn wrap(value: Insecure, untrusted: bool) -> Self {
        match untrusted {
            true => Self::wrap_untrusted(value),
            false => Self::wrap_ok(value),
        }
    }
}

impl<Insecure, Trusted> MaybeUntrusted<Insecure, Trusted> {
    /// Returns true if the value is untrusted
    pub fn is_untrusted(&self) -> bool {
        match self {
            MaybeUntrusted::Ok(_) => false,
            MaybeUntrusted::Untrusted(_) => true,
        }
    }

    /// Returns true if the value is not untrusted
    pub fn is_ok(&self) -> bool {
        !self.is_untrusted()
    }

    /// Wraps the provided values as Untrusted
    pub fn wrap_untrusted(value: Insecure) -> Self {
        MaybeUntrusted::Untrusted(value.into())
    }

    /// Wraps the provided values as Ok
    pub fn wrap_ok(value: Trusted) -> Self {
        MaybeUntrusted::Ok(value)
    }
}

impl<Insecure, Trusted> SanitizeWith<Insecure, Trusted> for MaybeUntrusted<Insecure, Trusted> {
    /// Sanitizes the value using the provided sanitizer if the value is untrusted.
    ///
    /// The sanitizer may transmute the value to a different type.
    /// If sanitization fails, an error must be returned.
    fn sanitize_with<Sanitizer, Error>(self, sanitizer: Sanitizer) -> Result<Trusted, Error>
    where
        Sanitizer: FnOnce(Insecure) -> Result<Trusted, Error>,
    {
        match self {
            MaybeUntrusted::Ok(value) => Ok(value),
            MaybeUntrusted::Untrusted(value) => value.sanitize_with(sanitizer),
        }
    }
}

impl<Insecure, Trusted> From<UntrustedValue<Insecure>> for MaybeUntrusted<Insecure, Trusted> {
    /// Converts an [UntrustedValue] to a [MaybeUntrusted] value
    fn from(value: UntrustedValue<Insecure>) -> Self {
        MaybeUntrusted::Untrusted(value)
    }
}

impl<Insecure: Clone, Trusted: Clone> Clone for MaybeUntrusted<Insecure, Trusted> {
    /// Clones the value
    fn clone(&self) -> Self {
        match self {
            MaybeUntrusted::Ok(value) => MaybeUntrusted::Ok(value.clone()),
            MaybeUntrusted::Untrusted(value) => MaybeUntrusted::Untrusted(value.clone()),
        }
    }
}

impl<Insecure: Copy, Trusted: Copy> Copy for MaybeUntrusted<Insecure, Trusted> {}

impl<E, Insecure: SanitizeValue<Insecure, Error = E>> SanitizeValue<Insecure>
    for MaybeUntrusted<Insecure, Insecure>
{
    type Error = E;

    fn sanitize_value(self) -> Result<Insecure, Self::Error> {
        match self {
            MaybeUntrusted::Ok(value) => Ok(value),
            MaybeUntrusted::Untrusted(value) => value.sanitize_value(),
        }
    }
}
