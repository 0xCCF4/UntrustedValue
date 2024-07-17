use untrusted_value::{IntoUntrustedVariant, SanitizeValue, UntrustedValue};
use untrusted_value_derive::UntrustedVariant;

// note:
// - trusted version: support debugs
// - untrusted version: does not support debugs, since it may be unsafe to print the values
// Since all sub structs implement `SanitizeValue`, the unsafe version can
// use the `SanitizeValue` macro to automatically implement the `SanitizeValue` trait.
#[derive(UntrustedVariant, Debug)]
#[untrusted_derive(Clone, SanitizeValue)]
pub struct GeneralConfig {
    pub network: NetworkConfig,
    pub database: DatabaseConfig,
}

#[derive(UntrustedVariant, Clone, Debug)]
#[untrusted_derive(Clone)]
pub struct NetworkConfig {
    pub port: u32,
    pub listen_address: String,
}

#[derive(Clone, Debug)]
pub struct DatabaseConfig {}

impl SanitizeValue<NetworkConfig> for UntrustedValue<NetworkConfig> {
    type Error = ();

    fn sanitize_value(self) -> Result<NetworkConfig, Self::Error> {
        Ok(self.use_untrusted_value()) // do some sanitizing
    }
}

impl SanitizeValue<DatabaseConfig> for UntrustedValue<DatabaseConfig> {
    type Error = ();

    fn sanitize_value(self) -> Result<DatabaseConfig, Self::Error> {
        Ok(DatabaseConfig {}) // do some sanitizing
    }
}

fn user_input_func() -> GeneralConfigUntrusted {
    let value_from_deserialize = GeneralConfig {
        database: DatabaseConfig {},
        network: NetworkConfig {
            port: 3000,
            listen_address: "127.0.0.0.0.0.1".to_string(),
        },
    };

    value_from_deserialize.to_untrusted_variant()
}

fn main() {
    let user_input = user_input_func();

    let _value = user_input.sanitize_value();
}
