/// The type implementing this struct can be sanitized.
///
/// Calling `sanitize_value()` on the implementing type should return a sanitized version of the value.
/// If the value cannot be sanitized, an error should be returned.
pub trait SanitizeValue<Trusted> {
    /// The error type that is returned in case of a sanitization failure.
    type Error;

    /// Sanitizes the value.
    /// The returned value might be of a different type.
    ///
    /// The returned value is sanitized and can be safely used.
    /// If the value cannot be sanitized, an error must be returned.
    fn sanitize_value(self) -> Result<Trusted, Self::Error>;
}
