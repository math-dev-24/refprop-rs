// ── Thermodynamic properties from a flash calculation ───────────────

/// Result of a TP-flash or PH-flash calculation.
///
/// **Default REFPROP units (molar basis):**
///
/// | Field            | Unit       |
/// |------------------|------------|
/// | temperature      | K          |
/// | pressure         | kPa        |
/// | density          | mol/L      |
/// | enthalpy         | J/mol      |
/// | entropy          | J/(mol·K)  |
/// | cv               | J/(mol·K)  |
/// | cp               | J/(mol·K)  |
/// | sound_speed      | m/s        |
/// | quality          | molar vapor fraction (0–1, >1 or <0 = single phase) |
/// | internal_energy  | J/mol      |
#[derive(Debug, Clone, PartialEq)]
pub struct ThermoProp {
    pub temperature: f64,
    pub pressure: f64,
    pub density: f64,
    pub enthalpy: f64,
    pub entropy: f64,
    pub cv: f64,
    pub cp: f64,
    pub sound_speed: f64,
    pub quality: f64,
    pub internal_energy: f64,
}

impl std::fmt::Display for ThermoProp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "T  = {:.4} K", self.temperature)?;
        writeln!(f, "P  = {:.4} kPa", self.pressure)?;
        writeln!(f, "D  = {:.6} mol/L", self.density)?;
        writeln!(f, "H  = {:.4} J/mol", self.enthalpy)?;
        writeln!(f, "S  = {:.4} J/(mol·K)", self.entropy)?;
        writeln!(f, "Cv = {:.4} J/(mol·K)", self.cv)?;
        writeln!(f, "Cp = {:.4} J/(mol·K)", self.cp)?;
        writeln!(f, "W  = {:.4} m/s", self.sound_speed)?;
        write!(f, "Q  = {:.6}", self.quality)
    }
}

// ── Saturation properties ───────────────────────────────────────────

/// Saturation-line properties returned by `SATPdll` / `SATTdll`.
///
/// Densities are in **mol/L**.
#[derive(Debug, Clone, PartialEq)]
pub struct SaturationProps {
    /// Saturation temperature (K)
    pub temperature: f64,
    /// Saturation pressure (kPa)
    pub pressure: f64,
    /// Saturated-liquid density (mol/L)
    pub density_liquid: f64,
    /// Saturated-vapor density (mol/L)
    pub density_vapor: f64,
}

impl std::fmt::Display for SaturationProps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "T_sat  = {:.4} K ({:.2} °C)", self.temperature, self.temperature - 273.15)?;
        writeln!(f, "P_sat  = {:.4} kPa", self.pressure)?;
        writeln!(f, "D_liq  = {:.6} mol/L", self.density_liquid)?;
        write!(f, "D_vap  = {:.6} mol/L", self.density_vapor)
    }
}

// ── Transport properties ────────────────────────────────────────────

/// Viscosity and thermal conductivity at a given (T, D) state point.
#[derive(Debug, Clone, PartialEq)]
pub struct TransportProps {
    /// Dynamic viscosity (µPa·s)
    pub viscosity: f64,
    /// Thermal conductivity (W/(m·K))
    pub thermal_conductivity: f64,
}

impl std::fmt::Display for TransportProps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "eta = {:.6} µPa·s", self.viscosity)?;
        write!(f, "tcx = {:.6} W/(m·K)", self.thermal_conductivity)
    }
}

// ── Critical point ──────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct CriticalProps {
    /// Critical temperature (K)
    pub temperature: f64,
    /// Critical pressure (kPa)
    pub pressure: f64,
    /// Critical density (mol/L)
    pub density: f64,
}

impl std::fmt::Display for CriticalProps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Tc = {:.4} K ({:.2} °C)", self.temperature, self.temperature - 273.15)?;
        writeln!(f, "Pc = {:.4} kPa ({:.4} bar)", self.pressure, self.pressure / 100.0)?;
        write!(f, "Dc = {:.6} mol/L", self.density)
    }
}

// ── Fluid information ───────────────────────────────────────────────

/// Static information about a pure component (from `INFOdll`).
#[derive(Debug, Clone, PartialEq)]
pub struct FluidInfo {
    /// Molar mass (g/mol)
    pub molar_mass: f64,
    /// Triple-point temperature (K)
    pub triple_point_temp: f64,
    /// Normal boiling point (K)
    pub normal_boiling_point: f64,
    /// Critical temperature (K)
    pub critical_temperature: f64,
    /// Critical pressure (kPa)
    pub critical_pressure: f64,
    /// Critical density (mol/L)
    pub critical_density: f64,
    /// Critical compressibility factor Z_c
    pub compressibility_factor: f64,
    /// Acentric factor
    pub acentric_factor: f64,
    /// Dipole moment (debye)
    pub dipole_moment: f64,
    /// Gas constant R for this fluid (J/(mol·K))
    pub gas_constant: f64,
}

impl std::fmt::Display for FluidInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "M     = {:.4} g/mol", self.molar_mass)?;
        writeln!(f, "T_trp = {:.4} K", self.triple_point_temp)?;
        writeln!(f, "T_nbp = {:.4} K ({:.2} °C)", self.normal_boiling_point, self.normal_boiling_point - 273.15)?;
        writeln!(f, "Tc    = {:.4} K ({:.2} °C)", self.critical_temperature, self.critical_temperature - 273.15)?;
        writeln!(f, "Pc    = {:.4} kPa", self.critical_pressure)?;
        writeln!(f, "Dc    = {:.6} mol/L", self.critical_density)?;
        writeln!(f, "Zc    = {:.6}", self.compressibility_factor)?;
        writeln!(f, "omega = {:.6}", self.acentric_factor)?;
        writeln!(f, "dip   = {:.4} debye", self.dipole_moment)?;
        write!(f, "R     = {:.6} J/(mol·K)", self.gas_constant)
    }
}
