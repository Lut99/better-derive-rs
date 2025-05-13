# CHANGELOG for the `better-derive`-crate
This file keeps track of notable changes to the `better-derive`-crate.

The project uses [semantic versioning](https://semver.org). As such, breaking changes are indicated by **(BREAKING)**.


## v1.5.0 - 2025-05-13
### Added
- The `Serialize` derive macro from [`serde`](https://serde.rs).

### Changed
- Minimum required dependency versions are now more lenient, as this _is_ a library and not a binary.


## v1.4.0 - 2025-02-13
### Added
- The `Ord` derive macro.
- The `PartialOrd` derive macro.
    - Note: this one's actually relatively complex due to the potential inclusion of discriminants
      while ordering. For now, it uses a simple assumption where discriminants are always literals.
      For a more complex implementation, check the <https://github.com/Lut99/const-eval-rs>-crate.

### Fixed
- The crate having the wrong version number.


## v1.3.0 - 2025-02-06
### Added
- The toplevel `#[better_derive(bounds = (...))]`-attribute for manually deciding which types to bind on.
- The `#[better_derive(...)]`-attribute as an alias for every (applicable) macro-specific attribute.

### Changed
- Relaxed the bound generation algorithm by only binding types that involve generics.

### Fixed
- `PartialEq` impl not being correct (always comparing self with self).


## v1.2.0 - 2025-02-05
### Added
- The `#[debug(skip)]`-attribute for the `Debug`-derive macro.
- The `#[hash(skip)]`-attribute for the `Hash`-derive macro.
- The `#[partial_eq(skip)]`-attribute for the `PartialEq`-derive macro.


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
