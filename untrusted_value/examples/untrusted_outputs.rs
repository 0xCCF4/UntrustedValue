use std::any::Any;
use untrusted_value::derive::untrusted_output;
use untrusted_value::UntrustedValue;

// Some function inside other library
#[cfg_attr(feature = "derive", untrusted_output)]
fn some_lib_func() -> String {
    // if cfg matches, then use untrusted_output to wrap the
    // function output in UntrustedValue

    "abcdef".to_string()
}

fn main() {
    // call library function
    assert_eq!(
        some_lib_func().type_id(),
        UntrustedValue::wrap("test".to_string()).type_id()
    );
}
