use crate::backend::refprop::RefpropBackend;
use crate::error::*;
use crate::properties::*;
use std::env;
use std::path::Path;
use std::sync::Once;

/// High‑level entry point for REFPROP calculations.
///
/// Works with **pure fluids** (`Fluid::new("R134A")`), **predefined
/// mixtures** (`Fluid::new("R454C")`), and **custom mixtures**
/// (`Fluid::mixture(&[("R32", 0.6817), ("R1234YF", 0.3183)])`).
///
/// # Quick example
/// ```no_run
/// use refprop::Fluid;
///
/// let co2 = Fluid::new("CO2").unwrap();
/// let d = co2.get("D", "T", 300.0, "P", 8000.0).unwrap();
/// println!("density = {d} mol/L");
/// ```
pub struct Fluid {
    backend: RefpropBackend,
}

impl Fluid {
    /// Create a `Fluid` by name.
    ///
    /// * Pure fluids: `"R134A"`, `"WATER"`, `"CO2"`, …
    /// * Predefined mixtures: `"R454C"`, `"R410A"`, … (loaded from
    ///   the `mixtures/` directory)
    ///
    /// The REFPROP installation directory is located automatically via
    /// `REFPROP_PATH` (environment variable or `.env` file).
    pub fn new(fluid_name: &str) -> Result<Self> {
        Self::load_dotenv();
        let refprop_path = Self::find_refprop_path()?;
        let backend = RefpropBackend::new(fluid_name, &refprop_path)?;
        Ok(Self { backend })
    }

    /// Create a `Fluid` from an explicit list of components and mole
    /// fractions.
    ///
    /// ```no_run
    /// use refprop::Fluid;
    ///
    /// // R454C by hand: R32 (21.5 %) + R1234yf (78.5 %)
    /// let mix = Fluid::mixture(&[("R32", 0.215), ("R1234YF", 0.785)]).unwrap();
    /// let props = mix.props_tp(298.15, 500.0).unwrap();
    /// println!("{props}");
    /// ```
    pub fn mixture(components: &[(&str, f64)]) -> Result<Self> {
        Self::load_dotenv();
        let refprop_path = Self::find_refprop_path()?;
        let backend = RefpropBackend::new_mixture(components, &refprop_path)?;
        Ok(Self { backend })
    }

    // ── .env loading (executed only once) ────────────────────────────

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

    /// **Generic property lookup** – CoolProp‑style.
    ///
    /// Retrieve a single thermodynamic property given two input
    /// constraints.
    ///
    /// ```no_run
    /// # use refprop::Fluid;
    /// let f = Fluid::new("R134A").unwrap();
    ///
    /// // Density of saturated vapor at 0 °C
    /// let d = f.get("D", "T", 273.15, "Q", 1.0).unwrap();
    ///
    /// // Pressure at T = 300 K, H = 42000 J/mol
    /// let p = f.get("P", "H", 42000.0, "T", 300.0).unwrap();
    /// ```
    ///
    /// **Input pairs** (order doesn't matter):
    /// `(T,P)` `(P,H)` `(P,S)` `(T,Q)` `(P,Q)`
    ///
    /// **Output keys:**
    /// `T` `P` `D` `H` `S` `Q` `Cv` `Cp` `W` `E` `ETA` `TCX`
    pub fn get(
        &self,
        output: &str,
        key1: &str, val1: f64,
        key2: &str, val2: f64,
    ) -> Result<f64> {
        self.backend.get(output, key1, val1, key2, val2)
    }

    /// Temperature–pressure flash (T in K, P in kPa).
    pub fn props_tp(&self, t_kelvin: f64, p_kpa: f64) -> Result<ThermoProp> {
        self.backend.props_tp(t_kelvin, p_kpa)
    }

    /// Pressure–enthalpy flash (P in kPa, H in J/mol).
    pub fn props_ph(&self, p_kpa: f64, h_jmol: f64) -> Result<ThermoProp> {
        self.backend.props_ph(p_kpa, h_jmol)
    }

    /// Pressure–entropy flash (P in kPa, S in J/(mol·K)).
    pub fn props_ps(&self, p_kpa: f64, s_jmolk: f64) -> Result<ThermoProp> {
        self.backend.props_ps(p_kpa, s_jmolk)
    }

    /// Temperature–quality flash (T in K, Q ∈ [0,1]).
    pub fn props_tq(&self, t_kelvin: f64, quality: f64) -> Result<ThermoProp> {
        self.backend.props_tq(t_kelvin, quality)
    }

    /// Pressure–quality flash (P in kPa, Q ∈ [0,1]).
    pub fn props_pq(&self, p_kpa: f64, quality: f64) -> Result<ThermoProp> {
        self.backend.props_pq(p_kpa, quality)
    }

    /// Saturation properties at a given pressure (kPa).
    pub fn saturation_p(&self, p_kpa: f64) -> Result<SaturationProps> {
        self.backend.saturation_p(p_kpa)
    }

    /// Saturation properties at a given temperature (K).
    pub fn saturation_t(&self, t_kelvin: f64) -> Result<SaturationProps> {
        self.backend.saturation_t(t_kelvin)
    }

    /// Transport properties (viscosity, thermal conductivity) at (T, D).
    pub fn transport(&self, t_kelvin: f64, density_mol_l: f64) -> Result<TransportProps> {
        self.backend.transport(t_kelvin, density_mol_l)
    }

    /// Critical point (Tc, Pc, Dc).
    pub fn critical_point(&self) -> Result<CriticalProps> {
        self.backend.critical_point()
    }

    /// Static fluid information (molar mass, triple point, …).
    pub fn info(&self) -> Result<FluidInfo> {
        self.backend.fluid_info()
    }
}
