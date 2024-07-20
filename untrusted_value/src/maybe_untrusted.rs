use super::UntrustedValue;
use untrusted_value_derive_internals::{SanitizeValue, SanitizeWith};

/// Represents a value that might be untrusted. See `UntrustedValue` for more information.
pub enum MaybeUntrusted<Insecure, Trusted = Insecure> {
    /// Trusted value variant
    Ok(Trusted),
    /// Untrusted value variant
    Untrusted(UntrustedValue<Insecure>),
}

impl<Insecure> MaybeUntrusted<Insecure> {
    /// Be sure that you carefully handle the returned value since
    /// it may be controllable by a malicious actor (when it is a `MaybeUntrusted::Untrusted`).
    ///
    /// See the method documentation of the function returning this value
    pub fn use_untrusted_value(self) -> Insecure {
        match self {
            MaybeUntrusted::Ok(value) => value,
            MaybeUntrusted::Untrusted(value) => value.use_untrusted_value(),
        }
    }

    /// Wraps the provided value as maybe untrusted, according to given boolean
    pub fn wrap(value: Insecure, untrusted: bool) -> Self {
        if untrusted {
            Self::wrap_untrusted(value)
        } else {
            Self::wrap_ok(value)
        }
    }
}

impl<Insecure, Trusted> MaybeUntrusted<Insecure, Trusted> {
    /// Returns true if the value is untrusted
    pub fn is_untrusted(&self) -> bool {
        match self {
            MaybeUntrusted::Ok(_) => false,
            MaybeUntrusted::Untrusted(_) => true,
        }
    }

    /// Returns true if the value is not untrusted
    pub fn is_ok(&self) -> bool {
        !self.is_untrusted()
    }

    /// Wraps the provided values as Untrusted
    pub fn wrap_untrusted(value: Insecure) -> Self {
        MaybeUntrusted::Untrusted(value.into())
    }

    /// Wraps the provided values as Ok
    pub fn wrap_ok(value: Trusted) -> Self {
        MaybeUntrusted::Ok(value)
    }
}

impl<Insecure, Trusted> SanitizeWith<Insecure, Trusted> for MaybeUntrusted<Insecure, Trusted> {
    /// Sanitizes the value using the provided sanitizer if the value is untrusted.
    ///
    /// The sanitizer may transmute the value to a different type.
    /// If sanitization fails, an error must be returned.
    fn sanitize_with<Sanitizer, Error>(self, sanitizer: Sanitizer) -> Result<Trusted, Error>
    where
        Sanitizer: FnOnce(Insecure) -> Result<Trusted, Error>,
    {
        match self {
            MaybeUntrusted::Ok(value) => Ok(value),
            MaybeUntrusted::Untrusted(value) => value.sanitize_with(sanitizer),
        }
    }
}

impl<Insecure, Trusted> From<UntrustedValue<Insecure>> for MaybeUntrusted<Insecure, Trusted> {
    /// Converts an [`UntrustedValue`] to a [`MaybeUntrusted`] value
    fn from(value: UntrustedValue<Insecure>) -> Self {
        MaybeUntrusted::Untrusted(value)
    }
}

#[allow(clippy::expl_impl_clone_on_copy)]
impl<Insecure: Clone, Trusted: Clone> Clone for MaybeUntrusted<Insecure, Trusted> {
    /// Clones the value
    fn clone(&self) -> Self {
        match self {
            MaybeUntrusted::Ok(value) => MaybeUntrusted::Ok(value.clone()),
            MaybeUntrusted::Untrusted(value) => MaybeUntrusted::Untrusted(value.clone()),
        }
    }
}

impl<Insecure: Copy, Trusted: Copy> Copy for MaybeUntrusted<Insecure, Trusted> {}

impl<E, Insecure: SanitizeValue<Insecure, Error = E>> SanitizeValue<Insecure>
    for MaybeUntrusted<Insecure>
{
    type Error = E;

    fn sanitize_value(self) -> Result<Insecure, Self::Error> {
        match self {
            MaybeUntrusted::Ok(value) => Ok(value),
            MaybeUntrusted::Untrusted(value) => value.sanitize_value(),
        }
    }
}
