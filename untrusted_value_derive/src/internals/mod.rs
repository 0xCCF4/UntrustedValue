/// This trait provides a way to convert a struct into a struct containing only `untrusted_value::UntrustedValue` fields
/// as generated by the `#[derive(UntrustedVariant)]` macro.
pub trait IntoUntrustedVariant<Insecure, Trusted>
where
    Insecure: SanitizeWith<Insecure, Trusted>,
{
    /// Returns a wrapped variant of the struct, containing only
    /// `untrusted_value::UntrustedValue` fields.
    ///
    /// An instance of a struct like this:
    /// ```rust
    /// # use untrusted_value_derive::UntrustedVariant;
    /// #[derive(UntrustedVariant)]
    /// pub struct Example {
    ///    pub name: String,
    /// }
    /// ```
    ///
    /// will be turned into an instance of:
    /// ```rust
    /// # use untrusted_value::UntrustedValue;
    /// pub struct ExampleUntrusted {
    ///   pub name: UntrustedValue<String>,
    /// }
    /// ```
    fn to_untrusted_variant(self) -> Insecure;
}

/// This type can be sanitized using a provided sanitizer.
///
/// This trait will be implemented automatically for structs
/// generated by the `#[derive(UntrustedVariant)]` macro.
pub trait SanitizeWith<Insecure, Trusted> {
    /// Sanitizes the value using the provided sanitizer.
    ///
    /// If sanitization fails, an error must be returned.
    fn sanitize_with<Sanitizer, Error>(self, sanitizer: Sanitizer) -> Result<Trusted, Error>
    where
        Sanitizer: FnOnce(Insecure) -> Result<Trusted, Error>;
}
