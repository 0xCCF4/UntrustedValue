# Untrusted Value
This crate aim to provide a type-safe way to handle and sanitize potentially untrusted values
like user input.

It aims to provide compile-time [Taint checking](https://en.wikipedia.org/wiki/Taint_checking)
into Rust. All user input or in general all input coming from the outside
world into the program must be seen as untrusted and potentially malicious (called tainted).
A tainted value keeps its taint until a proper sanitization function is called
upon the tainted data, clearing its taint.

This crate introduces several data types, traits and macros to simplify the process
of taint tracking.

## What's the goal of this crate?
The goal of this crate is to help design more secure applications. By tainting all
program inputs, unsanitized data can not be used by accident. By providing a sanitizing
interface to tainted data, security analysis can focus on analysing the implemented sanitizing functions
instead of identifying where tainted data is located, and where it is used.

## Example usage
User data must be wrapped within the container `UntrustedValue` which
provides/marks the contained data as tainted.
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
pub use untrusted_value::derive::UntrustedVariant;

#[derive(UntrustedVariant)]
#[untrusted_derive(SanitizeValueEnd, Clone)] // tainted variant of NetworkConfig should be Cloneable
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

When a function is called by an application framework like Rocket/Poem/...,
the macro `untrusted_inputs` may be used to taint the function inputs:

```rust
#[route(path = "/"), method = "get"]
#[untrusted_inputs]
fn index(name: &str) -> Result<String, ()> {
    // MACRO inserts the following code:
        // let name = UntrustedValue::from(name);
        // let ... = UntrustedValue::from(...);
    
    // we can not use "name" directly, since it is
    // wrapped in an UntrustedValue

    // we must explicitly sanitize the value before usage
    let name = name.sanitize_with(/* func */)?;
    Ok(format!("Hello, {}!", name))
}
```

A library providing a function that returns untrusted data may use the macro `untrusted_output` to conditionally
taint the output if the library user desires this:

```rust
#[cfg_attr(feature = "some_feature", untrusted_output)]
pub fn query_database() -> String {
    // if cfg matches, then use untrusted_output to wrap the
    // function output in UntrustedValue

    // the macro will wrap the body with:
        // UntrustedValue::from(
    "abcdef".to_string()
        // )
}
```

See also the examples in the `examples` directory.

## Installation
The library is written in Rust, and can be added using `cargo`:
```bash
cargo add untrusted-value
```

## Runtime overhead
When using compile optimizations there should be no runtime overhead since
we are essentially just "renaming" data. The `UntrustedValue`
struct only contains a single field of the original data type.
When compiling for release the compiler should optimize all usage
of the `UntrustedValue` struct away.

## Features
Enabled by default:
 * `derive`: enables the macros to automatically generate code

Optional features:
 * `derive_harden_sanitize`: enables hardening for the derive macro `SanitizeValue`. When this feature is disabled, the
    implemented `fn sanitize_value(self)` errors-early. Which may be undesired if sanitizing timing side
    channels are a concern. When enabling this feature, first all sanitizers are run, then
    the first error is propagated.

## Limitations
Providing a taint tracking system is nice but still requires the developer to
taint the data properly. Currently, we are working on providing a crate level macro
to automatically check common taint source like input from environment variables, args, and
common frameworks, that will create a compile error if input data has not been tainted.

This crate does only provide an interface to taint and sanitize data. Using this system, still this does
not make an application inherently secure. The developer must still implement
appropriate sanitizing functions to clear the taint of the data. This unified
interface should help to focus security analysis on the sanitizing functions
instead of on potentially all places where tainted data might be used.

## Contribution
Contributions to the project are welcome! If you have a feature request,
bug report, or want to contribute to the code, please open an
issue or a pull request.

## License
This project is licensed under the MIT license. See the LICENSE file for details.
