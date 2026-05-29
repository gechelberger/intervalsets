# Changelog — intervalsets-macros

All notable changes to the `intervalsets-macros` crate are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

This crate ships in lockstep with `intervalsets` and `intervalsets-core`; all
three share the workspace version and are released together via `cargo-release`.
See the repo [CONTRIBUTING.md](../CONTRIBUTING.md) for the versioning policy.

## [Unreleased]

### Added

- Initial release. Proc-macro implementations of `interval!`, `set!`, and
  `enum_interval!` — compile-time-checked literal macros re-exported from
  `intervalsets` / `intervalsets-core`. Grammar matches the runtime `FromStr`
  impls; malformed input fails the build instead of panicking at runtime. Bound
  bodies are tokenized as Rust source, so non-literal bounds (identifiers,
  function calls like `BigDecimal::from(0)`) work alongside numeric literals.
  An optional second argument supplies a storage-type hint emitted as a turbofish
  on the constructor and additionally wraps every bound expression in
  `<T as From<_>>::from(...)`, enabling any `From`-convertible source to be used
  as a bound. UFCS form is used so an inherent `from` method on the user's type
  can't shadow the trait method. Depends on `proc-macro2` / `quote` / `syn` /
  `proc-macro-crate`. Trybuild fixtures gate compile-error messages behind
  `INTERVALSETS_UI_TESTS=1` (run via `just test-ui`) to insulate the default
  workspace test from rustc diagnostic drift.

### Changed

### Deprecated

### Removed

### Fixed

### Security
