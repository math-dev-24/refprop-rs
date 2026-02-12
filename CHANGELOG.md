# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.2] - 2026-02-12

### Changed
- **Breaking:** Quality (Q) is now expressed in **percent** (0–100) instead of molar fraction (0–1), both as input and output
- `input_to_rp` now returns `Result<f64>` (was `f64`) to support input validation
- `props_tq` / `props_pq` quality parameter is now in percent (0–100)
- `ThermoProp.quality` is now returned in percent (0–100)

### Added
- `Converter::q_to_rp` — validates Q ∈ [0, 100] and converts to REFPROP fraction (0–1)
- `Converter::q_from_rp` — converts REFPROP fraction (0–1) back to percent (0–100)
- `InvalidInput` error when Q is outside the 0–100 range

## [0.1.1] - 2026-02-11

### Added
- Implement `Serialize` and `Deserialize` on unit structs

## [0.1.0] - 2026-02-10

### Added
- Initial release
- Safe Rust bindings for NIST REFPROP
- Thermodynamic and transport property calculations
- Error handling with `thiserror`

[Unreleased]: https://github.com/math-dev-24/refprop-rs/compare/v0.1.2...HEAD
[0.1.2]: https://github.com/math-dev-24/refprop-rs/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/math-dev-24/refprop-rs/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/math-dev-24/refprop-rs/releases/tag/v0.1.0
