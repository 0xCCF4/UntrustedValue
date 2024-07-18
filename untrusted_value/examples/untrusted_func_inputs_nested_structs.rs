use untrusted_value::derive::untrusted_inputs;
use untrusted_value::derive::UntrustedVariant;
use untrusted_value::SanitizeWith;
use untrusted_value::{IntoUntrustedVariant, SanitizeValue, UntrustedValue};

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

#[untrusted_inputs]
fn response_from_database(config: GeneralConfig) -> Result<GeneralConfig, ()> {
    // we can not use name directly, since it is
    // wrapped in an UntrustedValue

    // unpacks the untrusted value but propagate taint to members
    let unpacked = config.to_untrusted_variant();

    // now we can sanitize the value
    unpacked.sanitize_value()
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
