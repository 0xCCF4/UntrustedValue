use untrusted_value::{IntoUntrustedVariant, SanitizeWith};
use untrusted_value_derive::UntrustedVariant;

#[derive(UntrustedVariant)]
pub struct NetworkConfig {
    pub port: u32,
    pub listen_address: String,
}

fn main() {
    let user_input_from_config = NetworkConfig {
        port: 3000,
        listen_address: "127.0.0.0.0.0.1".to_string(),
    }
    .to_untrusted_variant();

    let _value = user_input_from_config
        .sanitize_with(|value| {
            Ok::<NetworkConfig, ()>(NetworkConfig {
                port: value
                    .port
                    .sanitize_with(|port| Ok::<u32, ()>(port + 1))
                    .unwrap(),
                listen_address: value
                    .listen_address
                    .sanitize_with(|address| Ok::<String, ()>(address + "test"))
                    .unwrap(),
            })
        })
        .expect("Sanitization failed");
}
