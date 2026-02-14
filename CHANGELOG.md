# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.2] - 2026-02-14

### Added
- **Flash (T,D)** — Temperature-Density flash via `TDFLSHdll`: `props_td()`
- **Flash (P,D)** — Pressure-Density flash via `PDFLSHdll`: `props_pd()`
- **Flash (D,H)** — Density-Enthalpy flash via `DHFLSHdll`: `props_dh()`
- **Flash (D,S)** — Density-Entropy flash via `DSFLSHdll`: `props_ds()`
- **Flash (H,S)** — Enthalpy-Entropy flash via `HSFLSHdll`: `props_hs()`
- All new pairs also available via `get()` (order-independent, supports `D`/`RHO` alias)
- Integration tests for all new pairs (pure fluids and mixtures)

## [0.2.1] - 2026-02-14

### Added
- **Flash (T,H)** — Temperature-Enthalpy flash via `THFLSHdll`: `props_th()`, `get("…", "T", …, "H", …)`
- **Flash (T,S)** — Temperature-Entropy flash via `TSFLSHdll`: `props_ts()`, `get("…", "T", …, "S", …)`
- New FFI bindings for `THFLSHdll` and `TSFLSHdll` (REFPROP 9.1+)
- Integration tests for TH and TS flash (pure fluids and mixtures)

## [0.2.0] - 2026-02-12

### Changed
- **Breaking:** Quality (Q) is now expressed in **percent** (0–100) instead of molar fraction (0–1), both as input and output
- **Breaking:** `input_to_rp` now returns `Result<f64>` (was `f64`) to support input validation
- `props_tq` / `props_pq` quality parameter is now in percent (0–100)
- `ThermoProp.quality` is now returned in percent (0–100)
- `sat_t_inner` / `sat_p_inner` now accept a `kph` parameter (bubble vs dew)

### Added
- `Converter::q_to_rp` — validates Q ∈ [0, 100] and converts to REFPROP fraction (0–1)
- `Converter::q_from_rp` — converts REFPROP fraction (0–1) back to percent (0–100)
- `InvalidInput` error when Q is outside the 0–100 range

### Fixed
- Zeotropic mixtures (e.g. R407C): `flash_tq_inner` / `flash_pq_inner` now select `kph` (bubble vs dew) based on quality — `kph=1` (bubble) when Q < 0.5, `kph=2` (dew) when Q ≥ 0.5 — giving correct saturation pressures for both ends
- `interpolate_quality` now uses the saturation pressure from SATTdll/SATPdll directly instead of recomputing it via THERMdll, fixing incorrect pressure for zeotropic mixtures at Q=0 and Q=1

## [0.1.1] - 2026-02-11

### Added
- Implement `Serialize` and `Deserialize` on unit structs

## [0.1.0] - 2026-02-10

### Added
- Initial release
- Safe Rust bindings for NIST REFPROP
- Thermodynamic and transport property calculations
- Error handling with `thiserror`

[Unreleased]: https://github.com/math-dev-24/refprop-rs/compare/v0.2.2...HEAD
[0.2.2]: https://github.com/math-dev-24/refprop-rs/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/math-dev-24/refprop-rs/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/math-dev-24/refprop-rs/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/math-dev-24/refprop-rs/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/math-dev-24/refprop-rs/releases/tag/v0.1.0
