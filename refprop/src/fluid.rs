use converter::{Converter, UnitSystem};

use crate::backend::refprop::RefpropBackend;
use crate::error::*;
use crate::properties::*;
use std::env;
use std::path::Path;
use std::sync::Once;

/// High‑level entry point for REFPROP calculations.
///
/// Works with **pure fluids**, **predefined mixtures**, and **custom
/// mixtures**.  An optional [`UnitSystem`] lets you work in °C + bar
/// (or any other combination) instead of REFPROP's native K + kPa.
///
/// # Quick example (engineering units)
/// ```no_run
/// use refprop::{Fluid, UnitSystem};
///
/// let co2 = Fluid::with_units("CO2", UnitSystem::engineering())?;
/// // Inputs and outputs are now in °C, bar, kg/m³, kJ/kg, …
/// let d = co2.get("D", "T", 25.0, "P", 50.0)?;
/// println!("density = {d:.2} kg/m³");
/// # Ok::<(), refprop::RefpropError>(())
/// ```
pub struct Fluid {
    backend: RefpropBackend,
    conv: Converter,
}

impl Fluid {
    // ── Constructors ─────────────────────────────────────────────────

    /// Create a `Fluid` using **REFPROP‑native units** (K, kPa, mol/L,
    /// J/mol, …).  Fully backward‑compatible.
    pub fn new(fluid_name: &str) -> Result<Self> {
        Self::with_units(fluid_name, UnitSystem::refprop())
    }

    /// Create a `Fluid` with a **custom unit system**.
    ///
    /// ```no_run
    /// use refprop::{Fluid, UnitSystem};
    ///
    /// let f = Fluid::with_units("R134A", UnitSystem::engineering())?;
    /// let p = f.get("P", "T", -5.0, "Q", 1.0)?;   // °C → bar
    /// # Ok::<(), refprop::RefpropError>(())
    /// ```
    pub fn with_units(fluid_name: &str, units: UnitSystem) -> Result<Self> {
        Self::load_dotenv();
        let refprop_path = Self::find_refprop_path()?;
        let backend = RefpropBackend::new(fluid_name, &refprop_path)?;
        let mm = backend.molar_mass_mix()?;
        let conv = Converter::new(units, mm);
        Ok(Self { backend, conv })
    }

    /// Create a **custom mixture** with REFPROP‑native units.
    pub fn mixture(components: &[(&str, f64)]) -> Result<Self> {
        Self::mixture_with_units(components, UnitSystem::refprop())
    }

    /// Create a **custom mixture** with a **custom unit system**.
    ///
    /// ```no_run
    /// use refprop::{Fluid, UnitSystem};
    ///
    /// let r454c = Fluid::mixture_with_units(
    ///     &[("R32", 0.215), ("R1234YF", 0.785)],
    ///     UnitSystem::engineering(),
    /// )?;
    /// # Ok::<(), refprop::RefpropError>(())
    /// ```
    pub fn mixture_with_units(
        components: &[(&str, f64)],
        units: UnitSystem,
    ) -> Result<Self> {
        Self::load_dotenv();
        let refprop_path = Self::find_refprop_path()?;
        let backend = RefpropBackend::new_mixture(components, &refprop_path)?;
        let mm = backend.molar_mass_mix()?;
        let conv = Converter::new(units, mm);
        Ok(Self { backend, conv })
    }

    // ── .env loading (once) ──────────────────────────────────────────

    fn load_dotenv() {
        static DOTENV_INIT: Once = Once::new();
        DOTENV_INIT.call_once(|| {
            if dotenvy::dotenv().is_ok() { return; }
            if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
                let p = std::path::PathBuf::from(dir).join(".env");
                if p.exists() { let _ = dotenvy::from_path(&p); return; }
            }
            if let Ok(exe) = std::env::current_exe() {
                if let Some(dir) = exe.parent() {
                    let p = dir.join(".env");
                    if p.exists() { let _ = dotenvy::from_path(&p); }
                }
            }
        });
    }

    // ── Path discovery ───────────────────────────────────────────────

    fn find_refprop_path() -> Result<String> {
        let mut tried = Vec::<String>::new();

        if let Ok(path) = env::var("REFPROP_PATH") {
            if Path::new(&path).exists() { return Ok(path); }
            tried.push(format!("REFPROP_PATH={path} (directory does not exist)"));
        }

        #[cfg(target_os = "windows")]
        let standard_paths = [
            r"C:\Program Files (x86)\REFPROP",
            r"C:\Program Files\REFPROP",
        ];
        #[cfg(target_os = "linux")]
        let standard_paths = ["/opt/refprop", "/usr/local/lib/refprop"];
        #[cfg(target_os = "macos")]
        let standard_paths = ["/Applications/REFPROP", "/opt/refprop"];

        for path in standard_paths {
            if Path::new(path).exists() { return Ok(path.to_string()); }
            tried.push(format!("{path} (not found)"));
        }

        Err(RefpropError::LibraryNotFound(format!(
            "REFPROP directory not found. Tried:\n  - {}\n\
             Set REFPROP_PATH to the directory containing REFPROP.DLL and the fluids/ folder.",
            tried.join("\n  - ")
        )))
    }

    // ── Public API ───────────────────────────────────────────────────

    /// **Generic property lookup** — CoolProp‑style.
    ///
    /// All values are in the unit system configured at construction.
    ///
    /// ```no_run
    /// # use refprop::{Fluid, UnitSystem};
    /// let f = Fluid::with_units("R134A", UnitSystem::engineering())?;
    /// let d = f.get("D", "T", 0.0, "Q", 1.0)?;   // 0 °C → kg/m³
    /// # Ok::<(), refprop::RefpropError>(())
    /// ```
    pub fn get(
        &self,
        output: &str,
        key1: &str, val1: f64,
        key2: &str, val2: f64,
    ) -> Result<f64> {
        let v1 = self.conv.input_to_rp(key1, val1);
        let v2 = self.conv.input_to_rp(key2, val2);
        let raw = self.backend.get(output, key1, v1, key2, v2)?;
        Ok(self.conv.output_from_rp(output, raw))
    }

    /// Temperature–pressure flash.
    pub fn props_tp(&self, t: f64, p: f64) -> Result<ThermoProp> {
        let raw = self.backend.props_tp(
            self.conv.t_to_rp(t),
            self.conv.p_to_rp(p),
        )?;
        Ok(self.convert_thermo(raw))
    }

    /// Pressure–enthalpy flash.
    pub fn props_ph(&self, p: f64, h: f64) -> Result<ThermoProp> {
        let raw = self.backend.props_ph(
            self.conv.p_to_rp(p),
            self.conv.h_to_rp(h),
        )?;
        Ok(self.convert_thermo(raw))
    }

    /// Pressure–entropy flash.
    pub fn props_ps(&self, p: f64, s: f64) -> Result<ThermoProp> {
        let raw = self.backend.props_ps(
            self.conv.p_to_rp(p),
            self.conv.s_to_rp(s),
        )?;
        Ok(self.convert_thermo(raw))
    }

    /// Temperature–quality flash.
    pub fn props_tq(&self, t: f64, q: f64) -> Result<ThermoProp> {
        let raw = self.backend.props_tq(
            self.conv.t_to_rp(t),
            q,
        )?;
        Ok(self.convert_thermo(raw))
    }

    /// Pressure–quality flash.
    pub fn props_pq(&self, p: f64, q: f64) -> Result<ThermoProp> {
        let raw = self.backend.props_pq(
            self.conv.p_to_rp(p),
            q,
        )?;
        Ok(self.convert_thermo(raw))
    }

    /// Saturation properties at a given pressure.
    pub fn saturation_p(&self, p: f64) -> Result<SaturationProps> {
        let raw = self.backend.saturation_p(self.conv.p_to_rp(p))?;
        Ok(self.convert_sat(raw))
    }

    /// Saturation properties at a given temperature.
    pub fn saturation_t(&self, t: f64) -> Result<SaturationProps> {
        let raw = self.backend.saturation_t(self.conv.t_to_rp(t))?;
        Ok(self.convert_sat(raw))
    }

    /// Transport properties at (T, D) — density must be in user units.
    pub fn transport(&self, t: f64, d: f64) -> Result<TransportProps> {
        let raw = self.backend.transport(
            self.conv.t_to_rp(t),
            self.conv.d_to_rp(d),
        )?;
        Ok(TransportProps {
            viscosity: self.conv.eta_from_rp(raw.viscosity),
            thermal_conductivity: self.conv.tcx_from_rp(raw.thermal_conductivity),
        })
    }

    /// Critical point (Tc, Pc, Dc) in user units.
    pub fn critical_point(&self) -> Result<CriticalProps> {
        let raw = self.backend.critical_point()?;
        Ok(CriticalProps {
            temperature: self.conv.t_from_rp(raw.temperature),
            pressure:    self.conv.p_from_rp(raw.pressure),
            density:     self.conv.d_from_rp(raw.density),
        })
    }

    /// Static fluid information (molar mass, triple point, …).
    ///
    /// **Note:** values in this struct are always in REFPROP‑native
    /// units regardless of the configured `UnitSystem`, because they
    /// describe intrinsic fluid constants.
    pub fn info(&self) -> Result<FluidInfo> {
        self.backend.fluid_info()
    }

    /// Access the active converter (useful for manual conversions).
    pub fn converter(&self) -> &Converter {
        &self.conv
    }

    // ── Internal conversion helpers ──────────────────────────────────

    fn convert_thermo(&self, raw: ThermoProp) -> ThermoProp {
        ThermoProp {
            temperature:     self.conv.t_from_rp(raw.temperature),
            pressure:        self.conv.p_from_rp(raw.pressure),
            density:         self.conv.d_from_rp(raw.density),
            enthalpy:        self.conv.h_from_rp(raw.enthalpy),
            entropy:         self.conv.s_from_rp(raw.entropy),
            cv:              self.conv.s_from_rp(raw.cv),
            cp:              self.conv.s_from_rp(raw.cp),
            sound_speed:     raw.sound_speed,
            quality:         raw.quality,
            internal_energy: self.conv.h_from_rp(raw.internal_energy),
        }
    }

    fn convert_sat(&self, raw: SaturationProps) -> SaturationProps {
        SaturationProps {
            temperature:    self.conv.t_from_rp(raw.temperature),
            pressure:       self.conv.p_from_rp(raw.pressure),
            density_liquid:  self.conv.d_from_rp(raw.density_liquid),
            density_vapor:   self.conv.d_from_rp(raw.density_vapor),
        }
    }
}
