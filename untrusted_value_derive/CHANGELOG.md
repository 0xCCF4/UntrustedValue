# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0](https://github.com/0xCCF4/UntrustedValue/compare/untrusted_value_derive-v0.1.1...untrusted_value_derive-v0.2.0) - 2024-07-17

### Added
- Added #[untrusted_inputs] func macro
- [**breaking**] Added derive macro for SanitizeValue and option to add derive macros for the UntrustedVariant using #[untrusted_derive(...)]

### Fixed
- fixed typo in derive_sanitize_harden implementation

### Other
- added keywords taint, static-analysis
- changes macro path to untrusted_value to absolute
- fixed failing doc tests

## [0.1.1](https://github.com/0xCCF4/UntrustedValue/compare/untrusted_value_derive-v0.1.0...untrusted_value_derive-v0.1.1) - 2024-07-15

### Fixed
- *(doc)* fixed doctests
