# Untrusted Value
This crate aim to provide a type-safe way to handle and sanitize potentially untrusted values
like user input.

Problem statement: Example: a String coming from a user input should be considered untrusted
while another String may be trusted application-wise. This difference
should be enforced using the Rust type-system.
It is therefore impossible to use user input or other untrusted values
without sanitizing them first.

## Example usage
The backbone of this crate is the `UntrustedValue` type which contains
considered unsafe data. 
```rust
use untrusted_value::{UntrustedValue};

let user_input: i32 = -36;
let user_input = UntrustedValue::from(user_input);

let trusted_value: u32 = user_input.sanitize_with(|value| {
   Ok::<u32, ()>(value.unsigned_abs())
}).expect("Sanitization failed");

println!("Sanitized value: {:?}", trusted_value);
```

When user data is a struct of different subcomponent that may have different 
sanitation procedures:

```rust
pub use untrusted_value::{IntoUntrustedVariant, SanitizeWith};
pub use untrusted_value_derive::UntrustedVariant;

#[derive(UntrustedVariant)]
pub struct NetworkConfig {
  pub port: u32,
  pub listen_address: String,
}

fn sanitize_ip_address(address: &str) -> Result<String, ()> {
    // somehow sanitize the address
    Ok(address.to_string())
}

fn sanitize_port(port: u32) -> Result<u32, ()> {
    // somehow sanitize the port
    Ok(port)
}

fn load_from_config() -> NetworkConfig {
    NetworkConfig {
        port: 1111,
        listen_address: "0.0.0.0".into(),
    }
}

let user_data = load_from_config().to_untrusted_variant();

// user data cannot be used on accident, since it is contained inside UntrustedValues

let user_data_clean = user_data
        .sanitize_with(|value| {
            Ok::<NetworkConfig, ()>(NetworkConfig {
                port: value
                    .port
                    .sanitize_with(sanitize_port)?,
                listen_address: value
                    .listen_address
                    .sanitize_with(sanitize_ip_address)?
            })
        })
        .expect("Sanitization failed");
```

See also the examples in the `examples` directory.

## Installation
The tool is written in Rust, and can be installed using `cargo`:
```bash
cargo add untrusted-value
```

## Features
The features enabled by default include:
* `allow_usage_without_sanitization`: enables the method `use_untrusted_value`
   to just use unpack an untrusted value (which might not be desirable). 

## Contribution
Contributions to the project are welcome! If you have a feature request,
bug report, or want to contribute to the code, please open an
issue or a pull request.

## License
This project is licensed under the MIT license. See the LICENSE file for details.
