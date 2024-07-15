# Untrusted Value
This crate aim to provide a type-safe way to handle and sanitize potentially untrusted values
like user input.

Problem statement: Example: a String coming from a user input should be considered untrusted
while another String may be trusted application-wise. This difference
should be enforced using the Rust type-system.
It is therefore impossible to use user input or other untrusted values
without sanitizing them first.

## Example usage
```rust
use untrusted_value::{UntrustedValue};

let user_input: i32 = -36;
let user_input = UntrustedValue::from(user_input);

let trusted_value: u32 = user_input.clone().sanitize_with(|value| {
   Ok::<u32, ()>(value.abs() as u32)
}).expect("Sanitization failed");

println!("Sanitized value: {:?}", trusted_value);

// OR

let trusted_value: u32 = user_input.sanitize_with(|value| {
   if value < -100 {
      Err("Failed to sanitize value")
   } else {
      Ok(value.abs() as u32)
   }
}).expect("Sanitization failed");

println!("Sanitized value: {:?}", trusted_value);
```

See also the examples in the `examples` directory.

If a type may be untrusted or not, the type `MaybeUntrusted` can be used.

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
