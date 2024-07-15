use untrusted_value::{IntoUntrustedVariant, SanitizeWith};
use untrusted_value_derive::UntrustedVariant;

#[derive(UntrustedVariant)]
pub struct NetworkConfig {
    pub port: u32,
    pub listen_address: String,
}

fn sanitize_ip_address(address: String) -> Result<String, ()> {
    // somehow sanitize the address
    Ok(address)
}

fn sanitize_port(port: u32) -> Result<u32, ()> {
    // somehow sanitize the port
    Ok(port)
}

fn main() {
    let user_input_from_config = NetworkConfig {
        port: 3000,
        listen_address: "127.0.0.0.0.0.1".to_string(),
    };
    let user_input_from_config = user_input_from_config.to_untrusted_variant();

    //let clone = user_input_from_config.clone();

    let _value = user_input_from_config
        .sanitize_with(|value| {
            Ok::<NetworkConfig, ()>(NetworkConfig {
                port: value.port.sanitize_with(sanitize_port)?,
                listen_address: value.listen_address.sanitize_with(sanitize_ip_address)?,
            })
        })
        .expect("Sanitization failed");
}
