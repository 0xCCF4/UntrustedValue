# Untrusted Value
This crate aim to provide a type-safe way to handle and sanitize potentially untrusted values
like user input.

It provides way to introduce compile-time [Taint checking](https://en.wikipedia.org/wiki/Taint_checking)
into Rust. All user input or in general all input coming from the outside
world into the program must be seen as untrusted and potentially malicious (called tainted).
A tainted value keeps its taint until a proper sanitation function is called
upon the tainted data, clearing its taint.

This crate introduces several data types, traits and macros to simplify the process
of taint tracking.

## Example usage
User data must be wrapped within the container `UntrustedValue` which
provides marks the contained data as tainted.
```rust
use untrusted_value::{UntrustedValue, SanitizeWith};

let user_input: i32 = -36;
let user_input = UntrustedValue::from(user_input);

// user data cannot be used on accident, since it is contained inside UntrustedValues
// UntrustedValue does only provide a limited set of implemented traits like Clone

let user_input = user_input.sanitize_with(...) // removes the taint
```

When user data is a struct of different subtypes:

```rust
pub use untrusted_value::{IntoUntrustedVariant, SanitizeValue};
pub use untrusted_value_derive::UntrustedVariant;

#[derive(UntrustedVariant)]
#[untrusted_derive(Clone)] // tainted variant should be Cloneable
pub struct NetworkConfig {
  pub port: u32,
  pub listen_address: String,
}

impl SanitizeValue<NetworkConfig> for NetworkConfigUntrusted {
    type Error = // ...
    fn sanitize_value(self) -> Result<NetworkConfig, Self::Error> { /* ... */ }
}

let user_data = load_from_config().to_untrusted_variant();

// user data cannot be used on accident, since it is contained inside UntrustedValues

let user_data = user_data.sanitize_value();
```

See also the examples in the `examples` directory.

## Installation
The library is written in Rust, and can be added using `cargo`:
```bash
cargo add untrusted-value
```

## Features
Enabled by default:
 * `allow_usage_without_sanitization`: enables the method `use_untrusted_value` to just use clear the taint of a value.
 * `derive`: enables the macros to automatically generate code (`#[derive(UntrustedVariant)`, `#[derive(SanitizeValue)`)

Optional features:
 * `derive_harden_sanitize`: enables hardening for the derive macro `SanitizeValue`. When this feature is disabled, the
    implemented `fn sanitize_value(self)` errors-early. Which may be undesired if sanitizing timing side
    channels are a concern. When enabling this feature, first all sanitizers are run, then
    the first error is propagated.

## Contribution
Contributions to the project are welcome! If you have a feature request,
bug report, or want to contribute to the code, please open an
issue or a pull request.

## License
This project is licensed under the MIT license. See the LICENSE file for details.
