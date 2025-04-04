use untrusted_value_derive_internals::{SanitizeValue, SanitizeWith};

/// Represents an untrusted/untrustworthy value.
/// The data contained inside this type is called tainted.
///
/// An attacker might be able to control (part) of the returned value.
/// Take special care processing this data.
///
/// Taint can be cleared by using one of the traits [`SanitizeWith`] or [`SanitizeValue`].
/// Effectively, sanitizing the data.
///
/// This type does explicitly not implement common traits like Debug, Display, etc.
/// since the data contained is considered untrusted.
/// If desired you COULD implement these traits in for your custom types.
///
/// For naming purposes an untrusted value mapped inside this type is considered safe/trusted
/// since it can not be accessed without sanitization.
#[repr(transparent)]
pub struct UntrustedValue<Insecure> {
    value: Insecure,
}

/// Implementation of the `UntrustedValue` type.
impl<Insecure> UntrustedValue<Insecure> {
    /// Be sure that you carefully handle the returned value since
    /// it may be controllable by a malicious actor.
    ///
    /// Does not perform any sanitization on the returned value.
    pub fn use_untrusted_value(self) -> Insecure {
        self.value
    }

    /// Wraps the provided value as [`UntrustedValue`]
    pub fn wrap(value: Insecure) -> Self {
        UntrustedValue { value }
    }
}

/// Taint can be cleared from the value by using a sanitizer.
/// Effectively unpacking the value; passing it to the sanitizer and returning the result.
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

/// Provide easy conversion from some value to an [`UntrustedValue`].
impl<Insecure> From<Insecure> for UntrustedValue<Insecure> {
    /// Wraps the provided value as [`UntrustedValue`]
    fn from(value: Insecure) -> Self {
        UntrustedValue::wrap(value)
    }
}

/// A tainted value may be cloned if the underlying value is cloneable. This is considered safe
/// since the taint is also cloned.
#[allow(clippy::expl_impl_clone_on_copy)]
impl<Insecure: Clone> Clone for UntrustedValue<Insecure> {
    /// Clones the value
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
        }
    }
}

// If the underlying value is copyable, the tainted value can also be copied. This is considered
// safe since the taint is also copied.
impl<Insecure: Copy> Copy for UntrustedValue<Insecure> {}

/// If the tainted data type can be sanitized using the [`SanitizeValue`] trait, implement also
/// the [`SanitizeValue`] trait for this [`UntrustedValue`] type.
impl<Sanitized, E, Insecure: SanitizeValue<Sanitized, Error = E>> SanitizeValue<Sanitized>
    for UntrustedValue<Insecure>
{
    /// The error type will be propagated from the underlying `SanitizeValue` implementation.
    type Error = E;

    /// Sanitizes the value.
    ///
    /// The returned value is sanitized and can be safely used.
    /// If the value cannot be sanitized, an error must be returned.
    fn sanitize_value(self) -> Result<Sanitized, Self::Error> {
        self.value.sanitize_value()
    }
}
