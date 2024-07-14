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
use untrusted_value::{SanitizeValue, UntrustedValue};

struct UserDataType {data: i32}
struct TrustedDataType {data: u32}

impl From<i32> for UserDataType {
    fn from(data: i32) -> Self {
        UserDataType {data}
    }
}
impl From<u32> for TrustedDataType {
    fn from(data: u32) -> Self {
        TrustedDataType {data}
    }
}

impl SanitizeValue<TrustedDataType, ()> for UserDataType {
    fn sanitize_value(self) -> Result<TrustedDataType, ()> {
        Ok( (self.data.abs() as u32).into() )
    }
}

/* USAGE */

let user_input: UserDataType = (-36).into();
let user_input = UntrustedValue::from(user_input);

let trusted_value: TrustedDataType = user_input.sanitize_value().expect("Sanitization failed");
```

If a type may be untrusted or not, the type `MaybeUntrusted` can be used.

## Installation
The tool is written in Rust, and can be installed using `cargo`:
```bash
cargo add untrusted-value
```

## Features
The features enabled by default include:
* `allow_usage_without_sanitization`: enables the method `use_untrusted_value`
   to just use unpack an untrusted value. 

## Contribution
Contributions to the project are welcome! If you have a feature request,
bug report, or want to contribute to the code, please open an
issue or a pull request.

## License
This project is licensed under the MIT license. See the LICENSE file for details.
