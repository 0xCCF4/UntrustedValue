/// Sanitize a value using the sanitize_with function
use untrusted_value::UntrustedValue;

fn main() {
    let user_input: i32 = -36;
    let user_input = UntrustedValue::from(user_input);

    let trusted_value: u32 = user_input
        .sanitize_with(|value| Ok::<u32, ()>(value.unsigned_abs()))
        .expect("Sanitization failed");

    println!("Sanitized value: {:?}", trusted_value);

    // OR

    let trusted_value: u32 = user_input
        .sanitize_with(|value| {
            if value < -100 {
                Err("Failed to sanitize value")
            } else {
                Ok(value.unsigned_abs())
            }
        })
        .expect("Sanitization failed");

    println!("Sanitized value: {:?}", trusted_value);
}
