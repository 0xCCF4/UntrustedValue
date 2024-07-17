use untrusted_value_derive::untrusted_inputs;
use untrusted_value_derive_internals::SanitizeWith;

fn no_sanitize<T>(value: T) -> Result<T, ()> {
    Ok(value)
}

// Imagine: some webserver specification
// #[oai(path = "/"), method = "get"]
#[untrusted_inputs]
fn index(name: &str) -> Result<String, ()> {
    // we can not use name directly, since it is
    // wrapped in an UntrustedValue

    let name = name.sanitize_with(no_sanitize)?;
    Ok(format!("Hello, {}!", name))
}

fn main() {
    // do a call to index route
    assert!(index("<script>alert('xss')</script>").is_err());
}
