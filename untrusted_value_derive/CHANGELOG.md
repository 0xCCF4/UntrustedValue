# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0](https://github.com/0xCCF4/UntrustedValue/compare/untrusted_value_derive-v0.2.4...untrusted_value_derive-v0.3.0) - 2024-07-20

### Added
- [**breaking**] trait have now constraints and guarantees that must hold for implementations (see documentation of each trait)
- [**breaking**] changed proc macro implementation to provide an better sanitization experience when using nested structs
- introduced `FromTrustedVariant`
- improved documentation of proc macros

## [0.2.4](https://github.com/0xCCF4/UntrustedValue/compare/untrusted_value_derive-v0.2.3...untrusted_value_derive-v0.2.4) - 2024-07-18

### Added
- added the untrusted_output macro

### Other
- added documentation for untrusted_output

## [0.2.3](https://github.com/0xCCF4/UntrustedValue/compare/untrusted_value_derive-v0.2.2...untrusted_value_derive-v0.2.3) - 2024-07-18

### Other
- all types are reexported in the untrsuted_value crate, to use derive macros just the main crate needs to be imported

## [0.2.2](https://github.com/0xCCF4/UntrustedValue/compare/untrusted_value_derive-v0.2.1...untrusted_value_derive-v0.2.2) - 2024-07-18

### Added
- introduced macro require_tainting

### Other
- moved traits/structs into own file each

## [0.2.1](https://github.com/0xCCF4/UntrustedValue/compare/untrusted_value_derive-v0.2.0...untrusted_value_derive-v0.2.1) - 2024-07-17

### Fixed
- *(doc)* fixed github repo url

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
