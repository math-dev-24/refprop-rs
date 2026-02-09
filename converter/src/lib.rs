//! Configurable unit conversion for REFPROP values.
//!
//! REFPROP internally uses: **K, kPa, mol/L, J/mol, J/(mol·K), µPa·s,
//! W/(m·K), m/s**.  This crate lets you work in whatever units you
//! prefer (°C, bar, kg/m³, kJ/kg, …) and handles the conversion
//! transparently.
//!
//! # Presets
//!
//! | Preset          | T   | P   | D     | H      | S         |
//! |-----------------|-----|-----|-------|--------|-----------|
//! | `refprop()`     | K   | kPa | mol/L | J/mol  | J/(mol·K) |
//! | `engineering()` | °C  | bar | kg/m³ | kJ/kg  | kJ/(kg·K) |
//! | `si()`          | K   | Pa  | kg/m³ | J/kg   | J/(kg·K)  |
//!
//! # Builder
//!
//! ```
//! use converter::{UnitSystem, TempUnit, PressUnit};
//!
//! let units = UnitSystem::new()
//!     .temperature(TempUnit::Celsius)
//!     .pressure(PressUnit::Bar);
//! ```

// ────────────────────────────────────────────────────────────────────
//  Unit enums
// ────────────────────────────────────────────────────────────────────

/// Temperature unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TempUnit {
    /// Kelvin (REFPROP native)
    Kelvin,
    /// Degrees Celsius
    Celsius,
    /// Degrees Fahrenheit
    Fahrenheit,
}

/// Pressure unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressUnit {
    /// Kilopascal (REFPROP native)
    KPa,
    /// Bar (1 bar = 100 kPa)
    Bar,
    /// Megapascal
    MPa,
    /// Pascal
    Pa,
    /// Standard atmosphere (101.325 kPa)
    Atm,
    /// Pounds per square inch
    Psi,
}

/// Density unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DensityUnit {
    /// mol/L (REFPROP native)
    MolPerL,
    /// kg/m³ (requires molar mass)
    KgPerM3,
}

/// Energy / enthalpy unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnergyUnit {
    /// J/mol (REFPROP native)
    JPerMol,
    /// kJ/kg (requires molar mass)
    KJPerKg,
    /// J/kg (requires molar mass)
    JPerKg,
}

/// Entropy / heat‑capacity unit (energy per temperature).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntropyUnit {
    /// J/(mol·K) (REFPROP native)
    JPerMolK,
    /// kJ/(kg·K) (requires molar mass)
    KJPerKgK,
    /// J/(kg·K) (requires molar mass)
    JPerKgK,
}

/// Dynamic viscosity unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViscosityUnit {
    /// µPa·s (REFPROP native)
    MicroPaS,
    /// mPa·s (= centipoise)
    MilliPaS,
    /// Pa·s
    PaS,
}

/// Thermal conductivity unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConductivityUnit {
    /// W/(m·K) (REFPROP native)
    WPerMK,
    /// mW/(m·K)
    MilliWPerMK,
}

// ────────────────────────────────────────────────────────────────────
//  UnitSystem — user configuration (no molar mass needed yet)
// ────────────────────────────────────────────────────────────────────

/// Describes the set of units the user wants to work in.
///
/// Create one with a preset (`refprop()`, `engineering()`, `si()`) or
/// customise individual properties with the builder methods.
#[derive(Debug, Clone)]
pub struct UnitSystem {
    pub temperature:  TempUnit,
    pub pressure:     PressUnit,
    pub density:      DensityUnit,
    pub energy:       EnergyUnit,
    pub entropy:      EntropyUnit,
    pub viscosity:    ViscosityUnit,
    pub conductivity: ConductivityUnit,
}

impl UnitSystem {
    /// Start from REFPROP‑native units.  Use the builder methods to
    /// change individual properties.
    pub fn new() -> Self { Self::refprop() }

    // ── Presets ──────────────────────────────────────────────────────

    /// REFPROP native: K, kPa, mol/L, J/mol, J/(mol·K), µPa·s, W/(m·K).
    pub fn refprop() -> Self {
        Self {
            temperature:  TempUnit::Kelvin,
            pressure:     PressUnit::KPa,
            density:      DensityUnit::MolPerL,
            energy:       EnergyUnit::JPerMol,
            entropy:      EntropyUnit::JPerMolK,
            viscosity:    ViscosityUnit::MicroPaS,
            conductivity: ConductivityUnit::WPerMK,
        }
    }

    /// Engineering / HVAC: °C, bar, kg/m³, kJ/kg, kJ/(kg·K).
    pub fn engineering() -> Self {
        Self {
            temperature:  TempUnit::Celsius,
            pressure:     PressUnit::Bar,
            density:      DensityUnit::KgPerM3,
            energy:       EnergyUnit::KJPerKg,
            entropy:      EntropyUnit::KJPerKgK,
            viscosity:    ViscosityUnit::MicroPaS,
            conductivity: ConductivityUnit::WPerMK,
        }
    }

    /// Strict SI: K, Pa, kg/m³, J/kg, J/(kg·K), Pa·s.
    pub fn si() -> Self {
        Self {
            temperature:  TempUnit::Kelvin,
            pressure:     PressUnit::Pa,
            density:      DensityUnit::KgPerM3,
            energy:       EnergyUnit::JPerKg,
            entropy:      EntropyUnit::JPerKgK,
            viscosity:    ViscosityUnit::PaS,
            conductivity: ConductivityUnit::WPerMK,
        }
    }

    // ── Builder methods ─────────────────────────────────────────────

    pub fn temperature(mut self, u: TempUnit) -> Self { self.temperature = u; self }
    pub fn pressure(mut self, u: PressUnit) -> Self { self.pressure = u; self }
    pub fn density(mut self, u: DensityUnit) -> Self { self.density = u; self }
    pub fn energy(mut self, u: EnergyUnit) -> Self { self.energy = u; self }
    pub fn entropy(mut self, u: EntropyUnit) -> Self { self.entropy = u; self }
    pub fn viscosity(mut self, u: ViscosityUnit) -> Self { self.viscosity = u; self }
    pub fn conductivity(mut self, u: ConductivityUnit) -> Self { self.conductivity = u; self }
}

impl Default for UnitSystem {
    fn default() -> Self { Self::refprop() }
}

// ────────────────────────────────────────────────────────────────────
//  Converter — UnitSystem + molar mass → ready to convert
// ────────────────────────────────────────────────────────────────────

/// Performs conversions between user units and REFPROP internal units.
///
/// Created by combining a [`UnitSystem`] with the fluid's molar mass
/// (needed for mol ↔ kg conversions).
#[derive(Debug, Clone)]
pub struct Converter {
    pub units: UnitSystem,
    /// Molar mass in g/mol (mixture‑averaged for mixtures).
    pub molar_mass: f64,
}

impl Converter {
    pub fn new(units: UnitSystem, molar_mass: f64) -> Self {
        Self { units, molar_mass }
    }

    /// Identity converter — no conversion at all (REFPROP native units,
    /// molar mass = 1 so mass‑based formulas still work formally).
    pub fn identity() -> Self {
        Self { units: UnitSystem::refprop(), molar_mass: 1.0 }
    }

    // ── Temperature ─────────────────────────────────────────────────

    /// User → REFPROP (K)
    pub fn t_to_rp(&self, t: f64) -> f64 {
        match self.units.temperature {
            TempUnit::Kelvin  => t,
            TempUnit::Celsius => t + 273.15,
            TempUnit::Fahrenheit => (t - 32.0) * 5.0 / 9.0 + 273.15,
        }
    }

    /// REFPROP (K) → User
    pub fn t_from_rp(&self, t: f64) -> f64 {
        match self.units.temperature {
            TempUnit::Kelvin  => t,
            TempUnit::Celsius => t - 273.15,
            TempUnit::Fahrenheit => (t - 273.15) * 9.0 / 5.0 + 32.0,
        }
    }

    // ── Pressure ────────────────────────────────────────────────────

    /// User → REFPROP (kPa)
    pub fn p_to_rp(&self, p: f64) -> f64 {
        match self.units.pressure {
            PressUnit::KPa => p,
            PressUnit::Bar => p * 100.0,
            PressUnit::MPa => p * 1000.0,
            PressUnit::Pa  => p / 1000.0,
            PressUnit::Atm => p * 101.325,
            PressUnit::Psi => p * 6.894_757,
        }
    }

    /// REFPROP (kPa) → User
    pub fn p_from_rp(&self, p: f64) -> f64 {
        match self.units.pressure {
            PressUnit::KPa => p,
            PressUnit::Bar => p / 100.0,
            PressUnit::MPa => p / 1000.0,
            PressUnit::Pa  => p * 1000.0,
            PressUnit::Atm => p / 101.325,
            PressUnit::Psi => p / 6.894_757,
        }
    }

    // ── Density ─────────────────────────────────────────────────────

    /// User → REFPROP (mol/L)
    pub fn d_to_rp(&self, d: f64) -> f64 {
        match self.units.density {
            DensityUnit::MolPerL => d,
            DensityUnit::KgPerM3 => d / self.molar_mass,
        }
    }

    /// REFPROP (mol/L) → User
    pub fn d_from_rp(&self, d: f64) -> f64 {
        match self.units.density {
            DensityUnit::MolPerL => d,
            DensityUnit::KgPerM3 => d * self.molar_mass,
        }
    }

    // ── Energy / Enthalpy / Internal energy ─────────────────────────

    /// User → REFPROP (J/mol)
    pub fn h_to_rp(&self, h: f64) -> f64 {
        match self.units.energy {
            EnergyUnit::JPerMol => h,
            EnergyUnit::KJPerKg => h * self.molar_mass,
            EnergyUnit::JPerKg  => h * self.molar_mass / 1000.0,
        }
    }

    /// REFPROP (J/mol) → User
    pub fn h_from_rp(&self, h: f64) -> f64 {
        match self.units.energy {
            EnergyUnit::JPerMol => h,
            EnergyUnit::KJPerKg => h / self.molar_mass,
            EnergyUnit::JPerKg  => h * 1000.0 / self.molar_mass,
        }
    }

    // ── Entropy / Cv / Cp ───────────────────────────────────────────

    /// User → REFPROP (J/(mol·K))
    pub fn s_to_rp(&self, s: f64) -> f64 {
        match self.units.entropy {
            EntropyUnit::JPerMolK => s,
            EntropyUnit::KJPerKgK => s * self.molar_mass,
            EntropyUnit::JPerKgK  => s * self.molar_mass / 1000.0,
        }
    }

    /// REFPROP (J/(mol·K)) → User
    pub fn s_from_rp(&self, s: f64) -> f64 {
        match self.units.entropy {
            EntropyUnit::JPerMolK => s,
            EntropyUnit::KJPerKgK => s / self.molar_mass,
            EntropyUnit::JPerKgK  => s * 1000.0 / self.molar_mass,
        }
    }

    // ── Viscosity ───────────────────────────────────────────────────

    /// REFPROP (µPa·s) → User
    pub fn eta_from_rp(&self, eta: f64) -> f64 {
        match self.units.viscosity {
            ViscosityUnit::MicroPaS => eta,
            ViscosityUnit::MilliPaS => eta / 1000.0,
            ViscosityUnit::PaS      => eta / 1_000_000.0,
        }
    }

    /// User → REFPROP (µPa·s)
    pub fn eta_to_rp(&self, eta: f64) -> f64 {
        match self.units.viscosity {
            ViscosityUnit::MicroPaS => eta,
            ViscosityUnit::MilliPaS => eta * 1000.0,
            ViscosityUnit::PaS      => eta * 1_000_000.0,
        }
    }

    // ── Thermal conductivity ────────────────────────────────────────

    /// REFPROP (W/(m·K)) → User
    pub fn tcx_from_rp(&self, tcx: f64) -> f64 {
        match self.units.conductivity {
            ConductivityUnit::WPerMK     => tcx,
            ConductivityUnit::MilliWPerMK => tcx * 1000.0,
        }
    }

    /// User → REFPROP (W/(m·K))
    pub fn tcx_to_rp(&self, tcx: f64) -> f64 {
        match self.units.conductivity {
            ConductivityUnit::WPerMK     => tcx,
            ConductivityUnit::MilliWPerMK => tcx / 1000.0,
        }
    }

    // ── Generic key‑based conversion ────────────────────────────────

    /// Convert a user‑provided input value to REFPROP units, choosing
    /// the right conversion based on the property key (e.g. `"T"`,
    /// `"P"`, `"H"`, …).
    pub fn input_to_rp(&self, key: &str, val: f64) -> f64 {
        match key.to_uppercase().as_str() {
            "T"                     => self.t_to_rp(val),
            "P"                     => self.p_to_rp(val),
            "D" | "RHO"            => self.d_to_rp(val),
            "H"                     => self.h_to_rp(val),
            "S"                     => self.s_to_rp(val),
            "E" | "U"              => self.h_to_rp(val),
            "CV" | "CP"            => self.s_to_rp(val),
            "ETA" | "V" | "VIS"    => self.eta_to_rp(val),
            "TCX" | "L" | "LAMBDA" => self.tcx_to_rp(val),
            _                       => val, // Q, W, etc.
        }
    }

    /// Convert a REFPROP output value to user units.
    pub fn output_from_rp(&self, key: &str, val: f64) -> f64 {
        match key.to_uppercase().as_str() {
            "T"                     => self.t_from_rp(val),
            "P"                     => self.p_from_rp(val),
            "D" | "RHO"            => self.d_from_rp(val),
            "H"                     => self.h_from_rp(val),
            "S"                     => self.s_from_rp(val),
            "E" | "U"              => self.h_from_rp(val),
            "CV" | "CP"            => self.s_from_rp(val),
            "ETA" | "V" | "VIS"    => self.eta_from_rp(val),
            "TCX" | "L" | "LAMBDA" => self.tcx_from_rp(val),
            _                       => val, // Q, W, etc.
        }
    }
}
