/// Sanitize a value using the SanitizeValue trait
use untrusted_value::{SanitizeValue, UntrustedValue};

struct UserDataType {
    data: i32,
}
struct TrustedDataType {
    data: u32,
}

impl From<i32> for UserDataType {
    fn from(data: i32) -> Self {
        UserDataType { data }
    }
}
impl From<u32> for TrustedDataType {
    fn from(data: u32) -> Self {
        TrustedDataType { data }
    }
}

impl SanitizeValue<TrustedDataType, ()> for UserDataType {
    fn sanitize_value(self) -> Result<TrustedDataType, ()> {
        Ok(self.data.unsigned_abs().into())
    }
}

fn main() {
    let user_input: UserDataType = (-36).into();
    let user_input = UntrustedValue::from(user_input);

    let trusted_value: TrustedDataType = user_input.sanitize_value().expect("Sanitization failed");

    println!("Sanitized value: {:?}", trusted_value.data);
}
