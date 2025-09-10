/// This trait provides a way to convert an untrusted or trusted type to another untrusted type.
///
/// This trait MUST guarantee the following properties:
/// 1. The conversion result MUST NOT contain untainted data that was tainted in the input.
/// 2. The conversion MUST NOT do any sanitization.
/// 3. If the input is not tainted, all parts of the output MUST be tainted.
///
/// If you could not guarantee 1-3, then you MUST NOT implement this trait. Probably you meant to
/// implement `SanitizeWith` or `SanitizeValue` instead, which allow you to sanitize the data or remove taint.
///
/// This trait is mainly used to map `UntrustedValue<SomeStruct>` to a struct where all members are
/// wrapped inside `UntrustedValue` containers amd vice-versa.
///
/// This trait is auto-implemented by the `#[derive(UntrustedVariant)]` macro, which will turn
/// an instance of a struct like this:
/// ```ignore
/// #[derive(UntrustedVariant)]
/// pub struct Example {
///    pub name: String,
/// }
/// ```
///
/// into an instance of:
/// ```rust
/// # use untrusted_value::UntrustedValue;
/// pub struct ExampleUntrusted {
///   pub name: UntrustedValue<String>,
/// }
/// ```
///
/// More on the guaranteed properties:
/// 1. All data parts in the input that were annotated as `UntrustedValue` must be dropped in the output or somehow again
///    wrapped in a `UntrustedValue` container. For example, when implementing this trait for `UntrustedValue<Example>` property
///    (1) is conserved since the member `name` is wrapped in an `UntrustedValue` container. Dropping parts of the input
///    is allowed since this means that the data can not be used in an untrusted manner anymore.
/// 2. This constraint is placed upon the user of this trait to make security analysis of the code easier. Analysing the
///    sanitization process is in that sense easier because analysts can focus analysing the implementations of `SanitizeWith`
///    and `SanitizeValue` that are designated for sanitization.
/// 3. If the input is not tainted, hence implementing this trait for a trusted type (e.g. the `Example` struct), the input
///    should be regarded the same as in property (1). This means implementing this trait for `UntrustedValue<Example>` or
///    `Example` should yield the same result. This constraint is placed upon the user of this trait since
///    crates providing Serialize/Deserialize features (like Serde) will likely operate on trusted types (e.g. `Example`).
///    Using the `ÌntoUntrustedVariant` trait is therefore the shortcut to first wrap the trusted type in `UntrustedValue`
///    and then calling `IntoUntrustedVariant` on it.
pub trait IntoUntrustedVariant<OtherInsecure> {
    /// Returns an equivalent untrusted type.
    ///
    /// No sanitization is done here, only the conversion to an untrusted type.
    ///
    /// This function MUST guarantee the following properties:
    /// 1. The conversion result MUST NOT contain untainted data that was tainted in the input.
    /// 2. The conversion MUST NOT do any sanitization.
    /// 3. If the input is not tainted, all parts of the output MUST be tainted.
    fn to_untrusted_variant(self) -> OtherInsecure;
}

/// This trait provides a way to convert an untrusted or trusted type to another untrusted type.
///
/// See also [`IntoUntrustedVariant`]. This trait is the inverse of `IntoUntrustedVariant`.
/// This trait is automatically implemented for all types that implement `IntoUntrustedVariant`.
///
/// You don't need to implement this trait manually.
pub trait FromTrustedVariant<OtherInsecure> {
    /// Converts the provided type to an equivalent untrusted type.
    ///
    /// No sanitization is done here, only the conversion to an untrusted type.
    ///
    /// This function MUST guarantee the following properties:
    /// 1. The conversion result MUST NOT contain untainted data that was tainted in the input.
    /// 2. The conversion MUST NOT do any sanitization.
    /// 3. If the input is not tainted, all parts of the output MUST be tainted.
    fn from_untrusted_variant(other: OtherInsecure) -> Self;
}

/// Implement this trait for all types that implement `IntoUntrustedVariant`.
impl<Insecure, OtherInsecure> FromTrustedVariant<OtherInsecure> for Insecure
where
    OtherInsecure: IntoUntrustedVariant<Insecure>,
{
    /// Converts the provided type to an equivalent untrusted type.
    ///
    /// No sanitization is done here, only the conversion to an untrusted type.
    ///
    /// This function MUST guarantee the following properties:
    /// 1. The conversion result MUST NOT contain untainted data that was tainted in the input.
    /// 2. The conversion MUST NOT do any sanitization.
    /// 3. If the input is not tainted, all parts of the output MUST be tainted.
    ///
    /// This method is auto-implemented since the `other` type implements the `IntoUntrustedVariant` trait.
    fn from_untrusted_variant(other: OtherInsecure) -> Self {
        other.to_untrusted_variant()
    }
}
