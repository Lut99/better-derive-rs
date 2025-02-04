# CHANGELOG for the `better-derive`-crate
This file keeps track of notable changes to the `better-derive`-crate.

The project uses [semantic versioning](https://semver.org). As such, breaking changes are indicated by **(BREAKING)**.


## v1.1.0 - 2025-02-04
### Added
- The `Clone` derive macro.
- The `Copy` derive macro.

### Changed
- Updated `proc-macro2` and `syn` dependencies to latest versions.


## v1.0.1 - 2025-01-09
### Added
- `Eq`, `Hash` and `PartialEq` to the examples.

### Fixed
- Various problems in the implementations of `Eq`, `Hash` and `PartialEq`.


## v1.0.0 - 2025-01-09
Renamed to the `better-derive` crate. **(BREAKING)**

### Added
- The `Eq`, `Hash` and `PartialEq` macros.


## v0.1.0 - 2024-12-26
Initial release!

### Added
- The `Debug`-derive macro as a drop-in replacement for the builtin one.
