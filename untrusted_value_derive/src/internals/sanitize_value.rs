/// The type implementing this struct can be sanitized.
///
/// Calling `sanitize_value()` on the implementing type should return a sanitized version of the value.
/// If the value cannot be sanitized, an error should be returned.
///
/// The `sanitize_value` function SHOULD clear all taint from the input.
pub trait SanitizeValue<Trusted> {
    /// The error type that is returned in case of a sanitization failure.
    type Error;

    /// Sanitizes the value.
    ///
    /// # Errors
    /// If the sanitization fails
    fn sanitize_value(self) -> Result<Trusted, Self::Error>;
}
