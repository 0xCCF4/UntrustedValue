#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

/// The type implementing this struct can be sanitized.
///
/// Calling `sanitize_value()` on the implementing type should return a sanitized version of the value.
/// If the value cannot be sanitized, an error should be returned.
pub trait SanitizeValue<Trusted, Error> {
    /// Sanitizes the value.
    /// The returned value might be of a different type.
    ///
    /// The returned value is sanitized and can be safely used.
    /// If the value cannot be sanitized, an error must be returned.
    fn sanitize_value(self) -> Result<Trusted, Error>;
}

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

    /// Sanitizes the value using the provided sanitizer.
    ///
    /// The sanitizer may transmute the value to a different type.
    /// If sanitization fails, an error must be returned.
    pub fn sanitize_with<Sanitizer, Trusted, Error>(
        self,
        sanitizer: Sanitizer,
    ) -> Result<Trusted, Error>
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

impl<Sanitized, Error, Insecure: SanitizeValue<Sanitized, Error>> SanitizeValue<Sanitized, Error>
    for UntrustedValue<Insecure>
{
    /// Sanitizes the value.
    ///
    /// The returned value is sanitized and can be safely used.
    /// If the value cannot be sanitized, an error must be returned.
    fn sanitize_value(self) -> Result<Sanitized, Error> {
        self.value.sanitize_value()
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

impl<Insecure: Copy> Copy for UntrustedValue<Insecure> {}

/// Represents a value that might be untrusted. See UntrustedValue for more information.
pub enum MaybeUntrusted<Insecure, Trusted = Insecure> {
    /// Trusted value variant
    Ok(Trusted),
    /// Untrusted value variant
    Untrusted(UntrustedValue<Insecure>),
}

impl<Insecure> MaybeUntrusted<Insecure, Insecure> {
    /// Be sure that you carefully handle the returned value since
    /// it may be controllable by a malicious actor (when it is a MaybeUntrusted::Untrusted).
    ///
    /// See the method documentation of the function returning this value
    #[cfg(feature = "allow_usage_without_sanitization")]
    pub fn use_untrusted_value(self) -> Insecure {
        match self {
            MaybeUntrusted::Ok(value) => value,
            MaybeUntrusted::Untrusted(value) => value.use_untrusted_value(),
        }
    }

    /// Wraps the provided value as maybe untrusted, according to given boolean
    pub fn wrap(value: Insecure, untrusted: bool) -> Self {
        match untrusted {
            true => Self::wrap_untrusted(value),
            false => Self::wrap_ok(value),
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

    /// Sanitizes the value using the provided sanitizer if the value is untrusted.
    ///
    /// The sanitizer may transmute the value to a different type.
    /// If sanitization fails, an error must be returned.
    pub fn sanitize_with<Sanitizer, Error>(self, sanitizer: Sanitizer) -> Result<Trusted, Error>
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
    /// Converts an [UntrustedValue] to a [MaybeUntrusted] value
    fn from(value: UntrustedValue<Insecure>) -> Self {
        MaybeUntrusted::Untrusted(value)
    }
}

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
