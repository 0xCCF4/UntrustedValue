use untrusted_value::derive::untrusted_inputs;
use untrusted_value::derive::UntrustedVariant;
use untrusted_value::{SanitizeValue, UntrustedValue};
use untrusted_value_derive_internals::IntoUntrustedVariant;

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

/// Sanitize the tainted version of `DatabaseConfig`
impl SanitizeValue<DatabaseConfig> for UntrustedValue<DatabaseConfig> {
    type Error = ();

    fn sanitize_value(self) -> Result<DatabaseConfig, Self::Error> {
        Ok(DatabaseConfig {}) // do some sanitizing
    }
}

// suppose this function is called by a library such like Rocket/Poem ... when a HTTP request is received
#[untrusted_inputs]
fn response_from_database(config: GeneralConfig) -> Result<GeneralConfig, ()> {
    // we can not use name directly, since it is
    // wrapped in an UntrustedValue

    config.sanitize_value()
}

fn main() {
    // do a call to index route
    assert!(response_from_database(GeneralConfig {
        database: DatabaseConfig {},
        network: NetworkConfig {
            port: 3000,
            listen_address: "<script>alert('xss')</script>".to_string(),
        },
    })
    .is_ok());
}
