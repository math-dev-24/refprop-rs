//! # refprop
//!
//! Safe, ergonomic Rust bindings for
//! [NIST REFPROP](https://www.nist.gov/srd/refprop) — thermodynamic and
//! transport properties of refrigerants, pure fluids, and mixtures.
//!
//! ## Highlights
//!
//! * **Pure fluids** — `Fluid::new("R134A")`
//! * **Predefined mixtures** — `Fluid::new("R410A")` (loaded from `.MIX`)
//! * **Custom mixtures** — `Fluid::mixture(&[("R32", 0.5), ("R125", 0.5)])`
//! * **CoolProp-style `get()`** — `fluid.get("D", "T", 0.0, "Q", 100.0)`
//! * **Configurable units** — work in °C + bar, K + kPa, or any combination
//! * **Thread-safe** — global mutex prevents data races on REFPROP's singleton state
//!
//! ## Quick example
//!
//! ```no_run
//! use refprop::{Fluid, UnitSystem};
//!
//! // Engineering units: °C, bar, kg/m³, kJ/kg
//! let co2 = Fluid::with_units("CO2", UnitSystem::engineering())?;
//!
//! let p = co2.get("P", "T", -5.0, "Q", 100.0)?;
//! println!("Psat(-5 °C) = {p:.2} bar");
//!
//! let d = co2.get("D", "T", -5.0, "Q", 100.0)?;
//! println!("D_vap(-5 °C) = {d:.2} kg/m³");
//! # Ok::<(), refprop::RefpropError>(())
//! ```
//!
//! ## Unit system
//!
//! Choose units at construction time with [`UnitSystem`] presets
//! ([`refprop()`](UnitSystem::refprop), [`engineering()`](UnitSystem::engineering),
//! [`si()`](UnitSystem::si)) or the builder:
//!
//! ```
//! use refprop::{UnitSystem, TempUnit, PressUnit};
//!
//! let units = UnitSystem::new()
//!     .temperature(TempUnit::Celsius)
//!     .pressure(PressUnit::Bar);
//! ```
//!
//! ## Mixtures
//!
//! ```no_run
//! use refprop::{Fluid, UnitSystem};
//!
//! // Predefined (from .MIX file)
//! let r410a = Fluid::with_units("R410A", UnitSystem::engineering())?;
//!
//! // Custom composition
//! let r454c = Fluid::mixture_with_units(
//!     &[("R32", 0.215), ("R1234YF", 0.785)],
//!     UnitSystem::engineering(),
//! )?;
//! # Ok::<(), refprop::RefpropError>(())
//! ```

// ── Internal modules ─────────────────────────────────────────────────
mod backend;
pub mod converter;
pub mod error;
pub mod sys;
pub mod fluid;
pub mod properties;

// ── Public re-exports ────────────────────────────────────────────────
pub use error::{RefpropError, Result};
pub use fluid::Fluid;
pub use properties::{
    CriticalProps, FluidInfo, SaturationProps, ThermoProp, TransportProps,
};

pub use converter::{
    Converter, UnitSystem,
    TempUnit, PressUnit, DensityUnit, EnergyUnit, EntropyUnit,
    ViscosityUnit, ConductivityUnit,
};
