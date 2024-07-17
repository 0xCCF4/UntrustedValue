# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.1](https://github.com/0xCCF4/UntrustedValue/compare/untrusted_value-v0.2.0...untrusted_value-v0.2.1) - 2024-07-17

### Fixed
- *(doc)* fixed github repo url

## [0.2.0](https://github.com/0xCCF4/UntrustedValue/compare/untrusted_value-v0.1.3...untrusted_value-v0.2.0) - 2024-07-17

### Added
- Added #[untrusted_inputs] func macro
- [**breaking**] Added derive macro for SanitizeValue and option to add derive macros for the UntrustedVariant using #[untrusted_derive(...)]

### Other
- added keywords taint, static-analysis

## [0.1.3](https://github.com/0xCCF4/UntrustedValue/compare/untrusted_value-v0.1.2...untrusted_value-v0.1.3) - 2024-07-15

### Added
- added UntrustedVariant proc macro

### Fixed
- include derive internals even if derive feature is off
- *(doc)* fixed doctests

### Other
- added cargo.toml documentation
- updated documentation to reflect newly added proc macro
