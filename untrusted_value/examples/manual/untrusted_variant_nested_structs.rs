use untrusted_value::derive::UntrustedVariant;
use untrusted_value::{IntoUntrustedVariant, SanitizeValue, UntrustedValue};

// note:
// - trusted version: support debugs
// - untrusted version: does not support debugs, since it may be unsafe to print the values
// Since all sub structs implement `SanitizeValue`, the unsafe version can
// use the `SanitizeValue` macro to automatically implement the `SanitizeValue` trait.
#[derive(Debug, UntrustedVariant)] // <-- Implements `GeneralConfigUntrusted`
#[untrusted_derive(Clone, SanitizeValue)]
pub struct GeneralConfig {
    pub network: NetworkConfig,
    pub database: DatabaseConfig,
}

#[derive(Clone, Debug, UntrustedVariant)] // <-- Implements `NetworkConfigUntrusted`
#[untrusted_derive(Clone)]
pub struct NetworkConfig {
    pub port: u32,
    pub listen_address: String,
}

#[derive(Clone, Debug)]
pub struct DatabaseConfig {}

/// Sanitize the tainted version of `NetworkConfig`
impl SanitizeValue<NetworkConfig> for UntrustedValue<NetworkConfig> {
    type Error = ();

    fn sanitize_value(self) -> Result<NetworkConfig, Self::Error> {
        let unpacked = self.use_untrusted_value().to_untrusted_variant();
        Ok(NetworkConfig {
            port: unpacked.port.use_untrusted_value(),
            listen_address: unpacked.listen_address.use_untrusted_value(),
        }) // in real application: do some sanitizing
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
