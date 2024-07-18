//! This crate contains the trait definitions for the [untrusted_value](https://docs.rs/untrusted_value/latest/untrusted_value/)
//! and [untrusted_value_derive](https://docs.rs/untrusted_value/latest/untrusted_value_derive/) crate.
//!
//! All types are reexported in the [untrusted_value](https://docs.rs/untrusted_value/latest/untrusted_value/) crate, so
//! you should properly use that crate instead.
//!
//! See also the main repo at [https://github.com/0xCCF4/UntrustedValue](https://github.com/0xCCF4/UntrustedValue).
#![warn(missing_docs)]

mod internals;

pub use internals::*;
