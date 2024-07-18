use untrusted_value_derive_internals::{SanitizeValue, SanitizeWith};

/// Represents an untrusted/untrustworthy value.
///
/// An attacker might be able to control (part) of the returned value.
/// Take special care processing this data.
///
/// See the method documentation of the function returning this value
pub struct UntrustedValue<Insecure> {
    value: Insecure,
}

impl<Insecure> UntrustedValue<Insecure> {
    /// Be sure that you carefully handle the returned value since
    /// it may be controllable by a malicious actor.
    ///
    /// See the method documentation of the function returning this value
    #[cfg(feature = "allow_usage_without_sanitization")]
    pub fn use_untrusted_value(self) -> Insecure {
        self.value
    }

    /// Wraps the provided value as [UntrustedValue]
    pub fn wrap(value: Insecure) -> Self {
        UntrustedValue { value }
    }
}

// does explicitly not implement Debug, Display, etc. to avoid processing untrusted data
// if desired, implement these traits manually for UntrustedValue<SomeCustomType>

impl<Insecure, Trusted> SanitizeWith<Insecure, Trusted> for UntrustedValue<Insecure> {
    /// Sanitizes the value using the provided sanitizer.
    ///
    /// The sanitizer may transmute the value to a different type.
    /// If sanitization fails, an error must be returned.
    fn sanitize_with<Sanitizer, Error>(self, sanitizer: Sanitizer) -> Result<Trusted, Error>
    where
        Sanitizer: FnOnce(Insecure) -> Result<Trusted, Error>,
    {
        sanitizer(self.value)
    }
}

impl<Insecure> From<Insecure> for UntrustedValue<Insecure> {
    /// Wraps the provided value as [UntrustedValue]
    fn from(value: Insecure) -> Self {
        UntrustedValue::wrap(value)
    }
}

impl<Insecure: Clone> Clone for UntrustedValue<Insecure> {
    /// Clones the value
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
        }
    }
}

impl<Sanitized, E, Insecure: SanitizeValue<Sanitized, Error = E>> SanitizeValue<Sanitized>
    for UntrustedValue<Insecure>
{
    type Error = E;

    /// Sanitizes the value.
    ///
    /// The returned value is sanitized and can be safely used.
    /// If the value cannot be sanitized, an error must be returned.
    fn sanitize_value(self) -> Result<Sanitized, Self::Error> {
        self.value.sanitize_value()
    }
}

impl<Insecure: Copy> Copy for UntrustedValue<Insecure> {}
