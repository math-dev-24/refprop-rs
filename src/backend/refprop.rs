use std::os::raw::c_long;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard};

use crate::sys::*;

use crate::error::*;
use crate::properties::*;

// ── Global lock (REFPROP is NOT thread-safe) ────────────────────────
// The lock value tracks which backend ID is currently loaded so we
// only re-call SETUPdll when the active fluid changes.
static REFPROP_LOCK: Mutex<usize> = Mutex::new(0);
static NEXT_BACKEND_ID: AtomicUsize = AtomicUsize::new(1);

// ── Backend ─────────────────────────────────────────────────────────

#[allow(dead_code)]
pub struct RefpropBackend {
    id: usize,
    lib: RefpropLibrary,
    refprop_path: PathBuf,
    /// Number of components (1 for pure fluids).
    nc: usize,
    /// Molar composition array.
    z: [f64; REFPROP_NC_MAX],
    /// Pipe-separated fluid file string, e.g. `"R134A.FLD"` or
    /// `"R32.FLD|R125.FLD"`.
    hfld_str: String,
}

impl RefpropBackend {
    // ================================================================
    //  Constructors
    // ================================================================

    /// Create a backend for a **pure fluid** or a **predefined mixture**
    /// (auto-detected from `.FLD` / `.MIX` files).
    pub fn new(fluid_name: &str, refprop_path: &str) -> Result<Self> {
        let path = PathBuf::from(refprop_path);
        if !path.exists() {
            return Err(RefpropError::LibraryNotFound(refprop_path.to_string()));
        }

        let lib = RefpropLibrary::load_from_dir(&path)
            .map_err(|e| RefpropError::LibraryNotFound(e.to_string()))?;

        // Set REFPROP path first (needed for both pure & mix)
        Self::set_path_raw(&lib, &path);

        let upper = fluid_name.to_uppercase();
        let fld_exists = Self::fluid_file_exists(&path, &upper);
        let mix_path = Self::find_mix_file(&path, &upper);

        if let Some(mix) = mix_path {
            // ── Predefined mixture (.MIX file) ──────────────────────
            let _guard = Self::lock_refprop()?;

            let mix_str = mix.to_str().unwrap_or_default();
            let hmxnme = to_c_string(mix_str, REFPROP_STRLEN);
            let hfmix = to_c_string("HMX.BNC", REFPROP_STRLEN);
            let hrf = to_c_string("DEF", REFPROP_STRLEN);

            let mut nc: i32 = 0;
            let mut hfld_buf = [0i8; REFPROP_FILESTR];
            let mut z = [0.0f64; REFPROP_NC_MAX];
            let mut ierr: i32 = 0;
            let mut herr = [0i8; REFPROP_STRLEN];

            unsafe {
                lib.SETMIXdll(
                    hmxnme.as_ptr(),
                    hfmix.as_ptr(),
                    hrf.as_ptr(),
                    &mut nc,
                    hfld_buf.as_mut_ptr(),
                    z.as_mut_ptr(),
                    &mut ierr,
                    herr.as_mut_ptr(),
                    REFPROP_STRLEN as c_long,
                    REFPROP_STRLEN as c_long,
                    REFPROP_STRLEN as c_long,
                    REFPROP_FILESTR as c_long,
                    REFPROP_STRLEN as c_long,
                );
            }
            Self::check_err(ierr, &herr)?;

            let id = NEXT_BACKEND_ID.fetch_add(1, Ordering::Relaxed);
            let hfld_str = from_c_string(&hfld_buf);

            Ok(Self {
                id,
                lib,
                refprop_path: path,
                nc: nc as usize,
                z,
                hfld_str,
            })
        } else if fld_exists {
            // ── Pure fluid (.FLD file) ──────────────────────────────
            let mut z = [0.0f64; REFPROP_NC_MAX];
            z[0] = 1.0;
            let hfld_str = format!("{}.FLD", upper);
            let id = NEXT_BACKEND_ID.fetch_add(1, Ordering::Relaxed);
            let backend = Self {
                id,
                lib,
                refprop_path: path,
                nc: 1,
                z,
                hfld_str,
            };
            backend.setup_fluid_locked()?;
            Ok(backend)
        } else {
            Err(RefpropError::FluidNotFound(format!(
                "{fluid_name} (no .FLD in fluids/ and no .MIX in mixtures/)"
            )))
        }
    }

    /// Create a backend for a **custom mixture** with explicit
    /// composition.
    pub fn new_mixture(components: &[(&str, f64)], refprop_path: &str) -> Result<Self> {
        let path = PathBuf::from(refprop_path);
        if !path.exists() {
            return Err(RefpropError::LibraryNotFound(refprop_path.to_string()));
        }
        if components.is_empty() || components.len() > REFPROP_NC_MAX {
            return Err(RefpropError::InvalidInput(format!(
                "Number of components must be 1–{REFPROP_NC_MAX}, got {}",
                components.len()
            )));
        }

        let lib = RefpropLibrary::load_from_dir(&path)
            .map_err(|e| RefpropError::LibraryNotFound(e.to_string()))?;

        Self::set_path_raw(&lib, &path);

        let nc = components.len();
        let hfld_str: String = components
            .iter()
            .map(|(name, _)| format!("{}.FLD", name.to_uppercase()))
            .collect::<Vec<_>>()
            .join("|");

        let mut z = [0.0f64; REFPROP_NC_MAX];
        for (i, (_, frac)) in components.iter().enumerate() {
            z[i] = *frac;
        }

        let id = NEXT_BACKEND_ID.fetch_add(1, Ordering::Relaxed);
        let backend = Self {
            id,
            lib,
            refprop_path: path,
            nc,
            z,
            hfld_str,
        };
        backend.setup_fluid_locked()?;
        Ok(backend)
    }

    // ================================================================
    //  Lock helper
    // ================================================================

    /// Acquire the global REFPROP lock, recovering gracefully from
    /// poisoning instead of panicking.
    fn lock_refprop() -> Result<MutexGuard<'static, usize>> {
        REFPROP_LOCK.lock().map_err(|_| {
            RefpropError::CalculationFailed(
                "REFPROP global lock is poisoned (a previous call panicked)".into(),
            )
        })
    }

    // ================================================================
    //  Input validation
    // ================================================================

    /// Ensure a value is a finite number (not NaN, not ±Infinity).
    fn validate_finite(name: &str, value: f64) -> Result<()> {
        if !value.is_finite() {
            return Err(RefpropError::InvalidInput(format!(
                "{name} must be a finite number, got {value}"
            )));
        }
        Ok(())
    }

    // ================================================================
    //  Setup helpers
    // ================================================================

    fn set_path_raw(lib: &RefpropLibrary, path: &PathBuf) {
        let path_str = path.to_str().unwrap_or_default();
        let path_c = to_c_string(path_str, REFPROP_STRLEN);
        unsafe { lib.SETPATHdll(path_c.as_ptr(), path_str.len() as c_long) };
    }

    fn fluid_file_exists(base: &PathBuf, upper_name: &str) -> bool {
        let fld = format!("{upper_name}.FLD");
        base.join("fluids").join(&fld).exists() || base.join("FLUIDS").join(&fld).exists()
    }

    fn find_mix_file(base: &PathBuf, upper_name: &str) -> Option<PathBuf> {
        let mix = format!("{upper_name}.MIX");
        let p1 = base.join("mixtures").join(&mix);
        if p1.exists() {
            return Some(p1);
        }
        let p2 = base.join("MIXTURES").join(&mix);
        if p2.exists() {
            return Some(p2);
        }
        None
    }

    /// Call SETUPdll under the lock (used by constructors).
    fn setup_fluid_locked(&self) -> Result<()> {
        let mut current_id = Self::lock_refprop()?;
        self.setup_fluid_inner()?;
        *current_id = self.id;
        Ok(())
    }

    /// Call SETPATHdll + SETUPdll.  **Caller must hold REFPROP_LOCK.**
    fn setup_fluid_inner(&self) -> Result<()> {
        Self::set_path_raw(&self.lib, &self.refprop_path);

        let nc_i: i32 = self.nc as i32;
        let hfld = to_c_string(&self.hfld_str, REFPROP_FILESTR);
        let hfmix = to_c_string("HMX.BNC", REFPROP_STRLEN);
        let hrf = to_c_string("DEF", REFPROP_STRLEN);
        let mut ierr: i32 = 0;
        let mut herr = [0i8; REFPROP_STRLEN];

        unsafe {
            self.lib.SETUPdll(
                &nc_i,
                hfld.as_ptr(),
                hfmix.as_ptr(),
                hrf.as_ptr(),
                &mut ierr,
                herr.as_mut_ptr(),
                REFPROP_FILESTR as c_long,
                REFPROP_STRLEN as c_long,
                REFPROP_STRLEN as c_long,
                REFPROP_STRLEN as c_long,
            );
        }
        Self::check_err(ierr, &herr)?;
        Ok(())
    }

    /// Ensure REFPROP is set up for *this* backend.
    /// **Caller must hold `current_id` from REFPROP_LOCK.**
    fn ensure_setup(&self, current_id: &mut usize) -> Result<()> {
        if *current_id != self.id {
            self.setup_fluid_inner()?;
            *current_id = self.id;
        }
        Ok(())
    }

    // ================================================================
    //  Inner methods (caller MUST hold REFPROP_LOCK and call
    //  ensure_setup first)
    // ================================================================

    fn flash_tp_inner(&self, t: f64, p: f64) -> Result<ThermoProp> {
        let (mut d, mut dl, mut dv) = (0.0, 0.0, 0.0);
        let mut x = [0.0f64; REFPROP_NC_MAX];
        let mut y = [0.0f64; REFPROP_NC_MAX];
        let (mut q, mut e, mut h, mut s, mut cv, mut cp, mut w) =
            (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let mut ierr: i32 = 0;
        let mut herr = [0i8; REFPROP_STRLEN];

        unsafe {
            self.lib.TPFLSHdll(
                &t,
                &p,
                self.z.as_ptr(),
                &mut d,
                &mut dl,
                &mut dv,
                x.as_mut_ptr(),
                y.as_mut_ptr(),
                &mut q,
                &mut e,
                &mut h,
                &mut s,
                &mut cv,
                &mut cp,
                &mut w,
                &mut ierr,
                herr.as_mut_ptr(),
                REFPROP_STRLEN as c_long,
            );
        }
        Self::check_err(ierr, &herr)?;
        Ok(ThermoProp {
            temperature: t,
            pressure: p,
            density: d,
            enthalpy: h,
            entropy: s,
            cv,
            cp,
            sound_speed: w,
            quality: q,
            internal_energy: e,
        })
    }

    fn flash_ph_inner(&self, p: f64, h_in: f64) -> Result<ThermoProp> {
        let (mut t, mut d, mut dl, mut dv) = (0.0, 0.0, 0.0, 0.0);
        let mut x = [0.0f64; REFPROP_NC_MAX];
        let mut y = [0.0f64; REFPROP_NC_MAX];
        let (mut q, mut e, mut s, mut cv, mut cp, mut w) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let mut ierr: i32 = 0;
        let mut herr = [0i8; REFPROP_STRLEN];

        unsafe {
            self.lib.PHFLSHdll(
                &p,
                &h_in,
                self.z.as_ptr(),
                &mut t,
                &mut d,
                &mut dl,
                &mut dv,
                x.as_mut_ptr(),
                y.as_mut_ptr(),
                &mut q,
                &mut e,
                &mut s,
                &mut cv,
                &mut cp,
                &mut w,
                &mut ierr,
                herr.as_mut_ptr(),
                REFPROP_STRLEN as c_long,
            );
        }
        Self::check_err(ierr, &herr)?;
        Ok(ThermoProp {
            temperature: t,
            pressure: p,
            density: d,
            enthalpy: h_in,
            entropy: s,
            cv,
            cp,
            sound_speed: w,
            quality: q,
            internal_energy: e,
        })
    }

    fn flash_ps_inner(&self, p: f64, s_in: f64) -> Result<ThermoProp> {
        let (mut t, mut d, mut dl, mut dv) = (0.0, 0.0, 0.0, 0.0);
        let mut x = [0.0f64; REFPROP_NC_MAX];
        let mut y = [0.0f64; REFPROP_NC_MAX];
        let (mut q, mut e, mut h, mut cv, mut cp, mut w) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let mut ierr: i32 = 0;
        let mut herr = [0i8; REFPROP_STRLEN];

        unsafe {
            self.lib.PSFLSHdll(
                &p,
                &s_in,
                self.z.as_ptr(),
                &mut t,
                &mut d,
                &mut dl,
                &mut dv,
                x.as_mut_ptr(),
                y.as_mut_ptr(),
                &mut q,
                &mut e,
                &mut h,
                &mut cv,
                &mut cp,
                &mut w,
                &mut ierr,
                herr.as_mut_ptr(),
                REFPROP_STRLEN as c_long,
            );
        }
        Self::check_err(ierr, &herr)?;
        Ok(ThermoProp {
            temperature: t,
            pressure: p,
            density: d,
            enthalpy: h,
            entropy: s_in,
            cv,
            cp,
            sound_speed: w,
            quality: q,
            internal_energy: e,
        })
    }

    /// SATTdll wrapper.
    ///
    /// `kph`: **1** = bubble point, **2** = dew point.
    fn sat_t_inner(&self, t: f64, kph: i32) -> Result<SaturationProps> {
        let (mut p, mut dl, mut dv) = (0.0, 0.0, 0.0);
        let mut x = [0.0f64; REFPROP_NC_MAX];
        let mut y = [0.0f64; REFPROP_NC_MAX];
        let mut ierr: i32 = 0;
        let mut herr = [0i8; REFPROP_STRLEN];

        unsafe {
            self.lib.SATTdll(
                &t,
                self.z.as_ptr(),
                &kph,
                &mut p,
                &mut dl,
                &mut dv,
                x.as_mut_ptr(),
                y.as_mut_ptr(),
                &mut ierr,
                herr.as_mut_ptr(),
                REFPROP_STRLEN as c_long,
            );
        }
        Self::check_err(ierr, &herr)?;
        Ok(SaturationProps {
            temperature: t,
            pressure: p,
            density_liquid: dl,
            density_vapor: dv,
        })
    }

    /// SATPdll wrapper.
    ///
    /// `kph`: **1** = bubble point, **2** = dew point.
    fn sat_p_inner(&self, p: f64, kph: i32) -> Result<SaturationProps> {
        let (mut t, mut dl, mut dv) = (0.0, 0.0, 0.0);
        let mut x = [0.0f64; REFPROP_NC_MAX];
        let mut y = [0.0f64; REFPROP_NC_MAX];
        let mut ierr: i32 = 0;
        let mut herr = [0i8; REFPROP_STRLEN];

        unsafe {
            self.lib.SATPdll(
                &p,
                self.z.as_ptr(),
                &kph,
                &mut t,
                &mut dl,
                &mut dv,
                x.as_mut_ptr(),
                y.as_mut_ptr(),
                &mut ierr,
                herr.as_mut_ptr(),
                REFPROP_STRLEN as c_long,
            );
        }
        Self::check_err(ierr, &herr)?;
        Ok(SaturationProps {
            temperature: t,
            pressure: p,
            density_liquid: dl,
            density_vapor: dv,
        })
    }

    /// THERMdll: compute all thermo props from (T, D).
    fn therm_inner(&self, t: f64, d: f64) -> ThermoProp {
        let (mut p, mut e, mut h, mut s, mut cv, mut cp, mut w, mut hjt) =
            (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        unsafe {
            self.lib.THERMdll(
                &t,
                &d,
                self.z.as_ptr(),
                &mut p,
                &mut e,
                &mut h,
                &mut s,
                &mut cv,
                &mut cp,
                &mut w,
                &mut hjt,
            );
        }
        ThermoProp {
            temperature: t,
            pressure: p,
            density: d,
            enthalpy: h,
            entropy: s,
            cv,
            cp,
            sound_speed: w,
            quality: f64::NAN,
            internal_energy: e,
        }
    }

    fn transport_inner(&self, t: f64, d: f64) -> Result<TransportProps> {
        let (mut eta, mut tcx) = (0.0, 0.0);
        let mut ierr: i32 = 0;
        let mut herr = [0i8; REFPROP_STRLEN];

        unsafe {
            self.lib.TRNPRPdll(
                &t,
                &d,
                self.z.as_ptr(),
                &mut eta,
                &mut tcx,
                &mut ierr,
                herr.as_mut_ptr(),
                REFPROP_STRLEN as c_long,
            );
        }
        Self::check_err(ierr, &herr)?;
        Ok(TransportProps {
            viscosity: eta,
            thermal_conductivity: tcx,
        })
    }

    fn flash_td_inner(&self, t: f64, d_in: f64) -> Result<ThermoProp> {
        let (mut p, mut dl, mut dv) = (0.0, 0.0, 0.0);
        let mut x = [0.0f64; REFPROP_NC_MAX];
        let mut y = [0.0f64; REFPROP_NC_MAX];
        let (mut q, mut e, mut h, mut s, mut cv, mut cp, mut w) =
            (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let mut ierr: i32 = 0;
        let mut herr = [0i8; REFPROP_STRLEN];

        unsafe {
            self.lib.TDFLSHdll(
                &t,
                &d_in,
                self.z.as_ptr(),
                &mut p,
                &mut dl,
                &mut dv,
                x.as_mut_ptr(),
                y.as_mut_ptr(),
                &mut q,
                &mut e,
                &mut h,
                &mut s,
                &mut cv,
                &mut cp,
                &mut w,
                &mut ierr,
                herr.as_mut_ptr(),
                REFPROP_STRLEN as c_long,
            );
        }
        Self::check_err(ierr, &herr)?;
        Ok(ThermoProp {
            temperature: t,
            pressure: p,
            density: d_in,
            enthalpy: h,
            entropy: s,
            cv,
            cp,
            sound_speed: w,
            quality: q,
            internal_energy: e,
        })
    }

    fn flash_pd_inner(&self, p: f64, d_in: f64) -> Result<ThermoProp> {
        let (mut t, mut dl, mut dv) = (0.0, 0.0, 0.0);
        let mut x = [0.0f64; REFPROP_NC_MAX];
        let mut y = [0.0f64; REFPROP_NC_MAX];
        let (mut q, mut e, mut h, mut s, mut cv, mut cp, mut w) =
            (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let mut ierr: i32 = 0;
        let mut herr = [0i8; REFPROP_STRLEN];

        unsafe {
            self.lib.PDFLSHdll(
                &p,
                &d_in,
                self.z.as_ptr(),
                &mut t,
                &mut dl,
                &mut dv,
                x.as_mut_ptr(),
                y.as_mut_ptr(),
                &mut q,
                &mut e,
                &mut h,
                &mut s,
                &mut cv,
                &mut cp,
                &mut w,
                &mut ierr,
                herr.as_mut_ptr(),
                REFPROP_STRLEN as c_long,
            );
        }
        Self::check_err(ierr, &herr)?;
        Ok(ThermoProp {
            temperature: t,
            pressure: p,
            density: d_in,
            enthalpy: h,
            entropy: s,
            cv,
            cp,
            sound_speed: w,
            quality: q,
            internal_energy: e,
        })
    }

    fn flash_th_inner(&self, t: f64, h_in: f64) -> Result<ThermoProp> {
        let (mut kr, mut p, mut d, mut dl, mut dv) = (1.0, 0.0, 0.0, 0.0, 0.0);
        let mut x = [0.0f64; REFPROP_NC_MAX];
        let mut y = [0.0f64; REFPROP_NC_MAX];
        let (mut q, mut e, mut s, mut cv, mut cp, mut w) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let mut ierr: i32 = 0;
        let mut herr = [0i8; REFPROP_STRLEN];

        unsafe {
            self.lib.THFLSHdll(
                &t,
                &h_in,
                self.z.as_ptr(),
                &mut kr,
                &mut p,
                &mut d,
                &mut dl,
                &mut dv,
                x.as_mut_ptr(),
                y.as_mut_ptr(),
                &mut q,
                &mut e,
                &mut s,
                &mut cv,
                &mut cp,
                &mut w,
                &mut ierr,
                herr.as_mut_ptr(),
                REFPROP_STRLEN as c_long,
            );
        }
        Self::check_err(ierr, &herr)?;
        Ok(ThermoProp {
            temperature: t,
            pressure: p,
            density: d,
            enthalpy: h_in,
            entropy: s,
            cv,
            cp,
            sound_speed: w,
            quality: q,
            internal_energy: e,
        })
    }

    fn flash_ts_inner(&self, t: f64, s_in: f64) -> Result<ThermoProp> {
        let (mut kr, mut p, mut d, mut dl, mut dv) = (1.0, 0.0, 0.0, 0.0, 0.0);
        let mut x = [0.0f64; REFPROP_NC_MAX];
        let mut y = [0.0f64; REFPROP_NC_MAX];
        let (mut q, mut e, mut h, mut cv, mut cp, mut w) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let mut ierr: i32 = 0;
        let mut herr = [0i8; REFPROP_STRLEN];

        unsafe {
            self.lib.TSFLSHdll(
                &t,
                &s_in,
                self.z.as_ptr(),
                &mut kr,
                &mut p,
                &mut d,
                &mut dl,
                &mut dv,
                x.as_mut_ptr(),
                y.as_mut_ptr(),
                &mut q,
                &mut e,
                &mut h,
                &mut cv,
                &mut cp,
                &mut w,
                &mut ierr,
                herr.as_mut_ptr(),
                REFPROP_STRLEN as c_long,
            );
        }
        Self::check_err(ierr, &herr)?;
        Ok(ThermoProp {
            temperature: t,
            pressure: p,
            density: d,
            enthalpy: h,
            entropy: s_in,
            cv,
            cp,
            sound_speed: w,
            quality: q,
            internal_energy: e,
        })
    }

    fn flash_dh_inner(&self, d_in: f64, h_in: f64) -> Result<ThermoProp> {
        let (mut t, mut p, mut dl, mut dv) = (0.0, 0.0, 0.0, 0.0);
        let mut x = [0.0f64; REFPROP_NC_MAX];
        let mut y = [0.0f64; REFPROP_NC_MAX];
        let (mut q, mut e, mut s, mut cv, mut cp, mut w) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let mut ierr: i32 = 0;
        let mut herr = [0i8; REFPROP_STRLEN];

        unsafe {
            self.lib.DHFLSHdll(
                &d_in,
                &h_in,
                self.z.as_ptr(),
                &mut t,
                &mut p,
                &mut dl,
                &mut dv,
                x.as_mut_ptr(),
                y.as_mut_ptr(),
                &mut q,
                &mut e,
                &mut s,
                &mut cv,
                &mut cp,
                &mut w,
                &mut ierr,
                herr.as_mut_ptr(),
                REFPROP_STRLEN as c_long,
            );
        }
        Self::check_err(ierr, &herr)?;
        Ok(ThermoProp {
            temperature: t,
            pressure: p,
            density: d_in,
            enthalpy: h_in,
            entropy: s,
            cv,
            cp,
            sound_speed: w,
            quality: q,
            internal_energy: e,
        })
    }

    fn flash_ds_inner(&self, d_in: f64, s_in: f64) -> Result<ThermoProp> {
        let (mut t, mut p, mut dl, mut dv) = (0.0, 0.0, 0.0, 0.0);
        let mut x = [0.0f64; REFPROP_NC_MAX];
        let mut y = [0.0f64; REFPROP_NC_MAX];
        let (mut q, mut e, mut h, mut cv, mut cp, mut w) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let mut ierr: i32 = 0;
        let mut herr = [0i8; REFPROP_STRLEN];

        unsafe {
            self.lib.DSFLSHdll(
                &d_in,
                &s_in,
                self.z.as_ptr(),
                &mut t,
                &mut p,
                &mut dl,
                &mut dv,
                x.as_mut_ptr(),
                y.as_mut_ptr(),
                &mut q,
                &mut e,
                &mut h,
                &mut cv,
                &mut cp,
                &mut w,
                &mut ierr,
                herr.as_mut_ptr(),
                REFPROP_STRLEN as c_long,
            );
        }
        Self::check_err(ierr, &herr)?;
        Ok(ThermoProp {
            temperature: t,
            pressure: p,
            density: d_in,
            enthalpy: h,
            entropy: s_in,
            cv,
            cp,
            sound_speed: w,
            quality: q,
            internal_energy: e,
        })
    }

    fn flash_hs_inner(&self, h_in: f64, s_in: f64) -> Result<ThermoProp> {
        let (mut t, mut p, mut d, mut dl, mut dv) = (0.0, 0.0, 0.0, 0.0, 0.0);
        let mut x = [0.0f64; REFPROP_NC_MAX];
        let mut y = [0.0f64; REFPROP_NC_MAX];
        let (mut q, mut e, mut cv, mut cp, mut w) = (0.0, 0.0, 0.0, 0.0, 0.0);
        let mut ierr: i32 = 0;
        let mut herr = [0i8; REFPROP_STRLEN];

        unsafe {
            self.lib.HSFLSHdll(
                &h_in,
                &s_in,
                self.z.as_ptr(),
                &mut t,
                &mut p,
                &mut d,
                &mut dl,
                &mut dv,
                x.as_mut_ptr(),
                y.as_mut_ptr(),
                &mut q,
                &mut e,
                &mut cv,
                &mut cp,
                &mut w,
                &mut ierr,
                herr.as_mut_ptr(),
                REFPROP_STRLEN as c_long,
            );
        }
        Self::check_err(ierr, &herr)?;
        Ok(ThermoProp {
            temperature: t,
            pressure: p,
            density: d,
            enthalpy: h_in,
            entropy: s_in,
            cv,
            cp,
            sound_speed: w,
            quality: q,
            internal_energy: e,
        })
    }

    /// T–Q flash: saturation + interpolation via THERMdll.
    ///
    /// For zeotropic mixtures the saturation curve depends on `kph`:
    /// `kph = 1` (bubble) when Q < 0.5, `kph = 2` (dew) when Q ≥ 0.5.
    fn flash_tq_inner(&self, t: f64, q: f64) -> Result<ThermoProp> {
        let kph = if q >= 0.5 { 2 } else { 1 };
        let sat = self.sat_t_inner(t, kph)?;
        self.interpolate_quality(t, sat.pressure, sat.density_liquid, sat.density_vapor, q)
    }

    /// P–Q flash: saturation + interpolation via THERMdll.
    ///
    /// For zeotropic mixtures the saturation curve depends on `kph`:
    /// `kph = 1` (bubble) when Q < 0.5, `kph = 2` (dew) when Q ≥ 0.5.
    fn flash_pq_inner(&self, p: f64, q: f64) -> Result<ThermoProp> {
        let kph = if q >= 0.5 { 2 } else { 1 };
        let sat = self.sat_p_inner(p, kph)?;
        self.interpolate_quality(sat.temperature, p, sat.density_liquid, sat.density_vapor, q)
    }

    /// Interpolate between saturated liquid and vapor using quality.
    ///
    /// For zeotropic mixtures, THERMdll may recompute a pressure that
    /// differs from the saturation pressure returned by SATTdll/SATPdll.
    /// We therefore always use the saturation pressure `p` directly.
    fn interpolate_quality(&self, t: f64, p: f64, dl: f64, dv: f64, q: f64) -> Result<ThermoProp> {
        if q <= 0.0 {
            let mut props = self.therm_inner(t, dl);
            props.quality = 0.0;
            props.pressure = p;
            return Ok(props);
        }
        if q >= 1.0 {
            let mut props = self.therm_inner(t, dv);
            props.quality = 1.0;
            props.pressure = p;
            return Ok(props);
        }
        let liq = self.therm_inner(t, dl);
        let vap = self.therm_inner(t, dv);

        let d = 1.0 / ((1.0 - q) / dl + q / dv);
        let lerp = |a: f64, b: f64| a * (1.0 - q) + b * q;

        Ok(ThermoProp {
            temperature: t,
            pressure: p,
            density: d,
            enthalpy: lerp(liq.enthalpy, vap.enthalpy),
            entropy: lerp(liq.entropy, vap.entropy),
            cv: lerp(liq.cv, vap.cv),
            cp: lerp(liq.cp, vap.cp),
            sound_speed: lerp(liq.sound_speed, vap.sound_speed),
            quality: q,
            internal_energy: lerp(liq.internal_energy, vap.internal_energy),
        })
    }

    // ================================================================
    //  Public locked methods
    // ================================================================

    pub fn props_tp(&self, t: f64, p: f64) -> Result<ThermoProp> {
        Self::validate_finite("temperature", t)?;
        Self::validate_finite("pressure", p)?;
        let mut cid = Self::lock_refprop()?;
        self.ensure_setup(&mut cid)?;
        self.flash_tp_inner(t, p)
    }

    pub fn props_ph(&self, p: f64, h: f64) -> Result<ThermoProp> {
        Self::validate_finite("pressure", p)?;
        Self::validate_finite("enthalpy", h)?;
        let mut cid = Self::lock_refprop()?;
        self.ensure_setup(&mut cid)?;
        self.flash_ph_inner(p, h)
    }

    pub fn props_ps(&self, p: f64, s: f64) -> Result<ThermoProp> {
        Self::validate_finite("pressure", p)?;
        Self::validate_finite("entropy", s)?;
        let mut cid = Self::lock_refprop()?;
        self.ensure_setup(&mut cid)?;
        self.flash_ps_inner(p, s)
    }

    pub fn props_tq(&self, t: f64, q: f64) -> Result<ThermoProp> {
        Self::validate_finite("temperature", t)?;
        Self::validate_finite("quality", q)?;
        let mut cid = Self::lock_refprop()?;
        self.ensure_setup(&mut cid)?;
        self.flash_tq_inner(t, q)
    }

    pub fn props_pq(&self, p: f64, q: f64) -> Result<ThermoProp> {
        Self::validate_finite("pressure", p)?;
        Self::validate_finite("quality", q)?;
        let mut cid = Self::lock_refprop()?;
        self.ensure_setup(&mut cid)?;
        self.flash_pq_inner(p, q)
    }

    pub fn props_th(&self, t: f64, h: f64) -> Result<ThermoProp> {
        Self::validate_finite("temperature", t)?;
        Self::validate_finite("enthalpy", h)?;
        let mut cid = Self::lock_refprop()?;
        self.ensure_setup(&mut cid)?;
        self.flash_th_inner(t, h)
    }

    pub fn props_ts(&self, t: f64, s: f64) -> Result<ThermoProp> {
        Self::validate_finite("temperature", t)?;
        Self::validate_finite("entropy", s)?;
        let mut cid = Self::lock_refprop()?;
        self.ensure_setup(&mut cid)?;
        self.flash_ts_inner(t, s)
    }

    pub fn props_td(&self, t: f64, d: f64) -> Result<ThermoProp> {
        Self::validate_finite("temperature", t)?;
        Self::validate_finite("density", d)?;
        let mut cid = Self::lock_refprop()?;
        self.ensure_setup(&mut cid)?;
        self.flash_td_inner(t, d)
    }

    pub fn props_pd(&self, p: f64, d: f64) -> Result<ThermoProp> {
        Self::validate_finite("pressure", p)?;
        Self::validate_finite("density", d)?;
        let mut cid = Self::lock_refprop()?;
        self.ensure_setup(&mut cid)?;
        self.flash_pd_inner(p, d)
    }

    pub fn props_dh(&self, d: f64, h: f64) -> Result<ThermoProp> {
        Self::validate_finite("density", d)?;
        Self::validate_finite("enthalpy", h)?;
        let mut cid = Self::lock_refprop()?;
        self.ensure_setup(&mut cid)?;
        self.flash_dh_inner(d, h)
    }

    pub fn props_ds(&self, d: f64, s: f64) -> Result<ThermoProp> {
        Self::validate_finite("density", d)?;
        Self::validate_finite("entropy", s)?;
        let mut cid = Self::lock_refprop()?;
        self.ensure_setup(&mut cid)?;
        self.flash_ds_inner(d, s)
    }

    pub fn props_hs(&self, h: f64, s: f64) -> Result<ThermoProp> {
        Self::validate_finite("enthalpy", h)?;
        Self::validate_finite("entropy", s)?;
        let mut cid = Self::lock_refprop()?;
        self.ensure_setup(&mut cid)?;
        self.flash_hs_inner(h, s)
    }

    pub fn saturation_p(&self, p: f64) -> Result<SaturationProps> {
        Self::validate_finite("pressure", p)?;
        let mut cid = Self::lock_refprop()?;
        self.ensure_setup(&mut cid)?;
        self.sat_p_inner(p, 1) // kph=1 → bubble point
    }

    pub fn saturation_t(&self, t: f64) -> Result<SaturationProps> {
        Self::validate_finite("temperature", t)?;
        let mut cid = Self::lock_refprop()?;
        self.ensure_setup(&mut cid)?;
        self.sat_t_inner(t, 1) // kph=1 → bubble point
    }

    pub fn transport(&self, t: f64, d: f64) -> Result<TransportProps> {
        Self::validate_finite("temperature", t)?;
        Self::validate_finite("density", d)?;
        let mut cid = Self::lock_refprop()?;
        self.ensure_setup(&mut cid)?;
        self.transport_inner(t, d)
    }

    pub fn critical_point(&self) -> Result<CriticalProps> {
        let mut cid = Self::lock_refprop()?;
        self.ensure_setup(&mut cid)?;

        let (mut tc, mut pc, mut dc) = (0.0, 0.0, 0.0);
        let mut ierr: i32 = 0;
        let mut herr = [0i8; REFPROP_STRLEN];

        unsafe {
            self.lib.CRITPdll(
                self.z.as_ptr(),
                &mut tc,
                &mut pc,
                &mut dc,
                &mut ierr,
                herr.as_mut_ptr(),
                REFPROP_STRLEN as c_long,
            );
        }
        Self::check_err(ierr, &herr)?;
        Ok(CriticalProps {
            temperature: tc,
            pressure: pc,
            density: dc,
        })
    }

    pub fn fluid_info(&self) -> Result<FluidInfo> {
        let mut cid = Self::lock_refprop()?;
        self.ensure_setup(&mut cid)?;

        let icomp: i32 = 1;
        let (mut wmm, mut ttrp, mut tnbpt) = (0.0, 0.0, 0.0);
        let (mut tc, mut pc, mut dc) = (0.0, 0.0, 0.0);
        let (mut zc, mut acf, mut dip, mut rgas) = (0.0, 0.0, 0.0, 0.0);

        unsafe {
            self.lib.INFOdll(
                &icomp, &mut wmm, &mut ttrp, &mut tnbpt, &mut tc, &mut pc, &mut dc, &mut zc,
                &mut acf, &mut dip, &mut rgas,
            );
        }
        Ok(FluidInfo {
            molar_mass: wmm,
            triple_point_temp: ttrp,
            normal_boiling_point: tnbpt,
            critical_temperature: tc,
            critical_pressure: pc,
            critical_density: dc,
            compressibility_factor: zc,
            acentric_factor: acf,
            dipole_moment: dip,
            gas_constant: rgas,
        })
    }

    // ================================================================
    //  Molar mass (mixture-averaged)
    // ================================================================

    /// Compute the molar mass of the loaded fluid or mixture (g/mol).
    ///
    /// For pure fluids this is identical to `fluid_info().molar_mass`.
    /// For mixtures it returns M_mix = Σ z_i · M_i.
    pub fn molar_mass_mix(&self) -> Result<f64> {
        let mut cid = Self::lock_refprop()?;
        self.ensure_setup(&mut cid)?;

        let mut m_mix = 0.0;
        for i in 0..self.nc {
            let icomp: i32 = (i + 1) as i32;
            let (mut wmm, mut d1, mut d2, mut d3, mut d4) = (0.0, 0.0, 0.0, 0.0, 0.0);
            let (mut d5, mut d6, mut d7, mut d8, mut d9) = (0.0, 0.0, 0.0, 0.0, 0.0);
            unsafe {
                self.lib.INFOdll(
                    &icomp, &mut wmm, &mut d1, &mut d2, &mut d3, &mut d4, &mut d5, &mut d6,
                    &mut d7, &mut d8, &mut d9,
                );
            }
            m_mix += self.z[i] * wmm;
        }
        Ok(m_mix)
    }

    // ================================================================
    //  Generic "get" – CoolProp-style PropsSI
    // ================================================================

    /// Retrieve a single property value given two input constraints.
    ///
    /// ```text
    /// fluid.get("D", "T", 273.15, "Q", 100.0)  // density of sat. vapor at 0 °C
    /// fluid.get("P", "T", 300.0,  "D", 12.0)   // pressure at T=300 K, D=12 mol/L
    /// fluid.get("H", "P", 500.0,  "T", 298.15) // enthalpy at 5 bar, 25 °C
    /// ```
    ///
    /// Supported input pairs: **(T,P) (T,D) (T,H) (T,S) (T,Q) (P,D) (P,H) (P,S) (P,Q) (D,H) (D,S) (H,S)**.
    /// Keys are **case-insensitive**.
    pub fn get(&self, output: &str, key1: &str, val1: f64, key2: &str, val2: f64) -> Result<f64> {
        Self::validate_finite(key1, val1)?;
        Self::validate_finite(key2, val2)?;

        let mut cid = Self::lock_refprop()?;
        self.ensure_setup(&mut cid)?;

        let k1 = key1.to_uppercase();
        let k2 = key2.to_uppercase();

        let props = match (k1.as_str(), k2.as_str()) {
            ("T", "P") => self.flash_tp_inner(val1, val2)?,
            ("P", "T") => self.flash_tp_inner(val2, val1)?,

            ("P", "H") => self.flash_ph_inner(val1, val2)?,
            ("H", "P") => self.flash_ph_inner(val2, val1)?,

            ("P", "S") => self.flash_ps_inner(val1, val2)?,
            ("S", "P") => self.flash_ps_inner(val2, val1)?,

            ("T", "Q") => self.flash_tq_inner(val1, val2)?,
            ("Q", "T") => self.flash_tq_inner(val2, val1)?,

            ("P", "Q") => self.flash_pq_inner(val1, val2)?,
            ("Q", "P") => self.flash_pq_inner(val2, val1)?,

            ("T", "D") | ("T", "RHO") => self.flash_td_inner(val1, val2)?,
            ("D", "T") | ("RHO", "T") => self.flash_td_inner(val2, val1)?,

            ("T", "H") => self.flash_th_inner(val1, val2)?,
            ("H", "T") => self.flash_th_inner(val2, val1)?,

            ("T", "S") => self.flash_ts_inner(val1, val2)?,
            ("S", "T") => self.flash_ts_inner(val2, val1)?,

            ("P", "D") | ("P", "RHO") => self.flash_pd_inner(val1, val2)?,
            ("D", "P") | ("RHO", "P") => self.flash_pd_inner(val2, val1)?,

            ("D", "H") | ("RHO", "H") => self.flash_dh_inner(val1, val2)?,
            ("H", "D") | ("H", "RHO") => self.flash_dh_inner(val2, val1)?,

            ("D", "S") | ("RHO", "S") => self.flash_ds_inner(val1, val2)?,
            ("S", "D") | ("S", "RHO") => self.flash_ds_inner(val2, val1)?,

            ("H", "S") => self.flash_hs_inner(val1, val2)?,
            ("S", "H") => self.flash_hs_inner(val2, val1)?,

            _ => {
                return Err(RefpropError::InvalidInput(format!(
                    "Unsupported input pair ({k1}, {k2}). \
                     Supported: (T,P) (T,D) (T,H) (T,S) (T,Q) (P,D) (P,H) (P,S) (P,Q) (D,H) (D,S) (H,S)"
                )));
            }
        };

        let out = output.to_uppercase();
        match out.as_str() {
            "T" => Ok(props.temperature),
            "P" => Ok(props.pressure),
            "D" | "RHO" => Ok(props.density),
            "H" => Ok(props.enthalpy),
            "S" => Ok(props.entropy),
            "Q" => Ok(props.quality),
            "CV" => Ok(props.cv),
            "CP" => Ok(props.cp),
            "W" | "A" => Ok(props.sound_speed),
            "E" | "U" => Ok(props.internal_energy),
            "ETA" | "V" | "VIS" => {
                let trn = self.transport_inner(props.temperature, props.density)?;
                Ok(trn.viscosity)
            }
            "TCX" | "L" | "LAMBDA" => {
                let trn = self.transport_inner(props.temperature, props.density)?;
                Ok(trn.thermal_conductivity)
            }
            _ => Err(RefpropError::InvalidInput(format!(
                "Unknown output property \"{output}\". \
                 Supported: T P D H S Q Cv Cp W E ETA TCX"
            ))),
        }
    }

    // ================================================================
    //  Helpers
    // ================================================================

    /// Check the REFPROP error code.
    ///
    /// - `ierr > 0`: hard error → returns `Err(RefpropError::Refprop)`
    /// - `ierr < 0`: warning → logs to stderr, returns `Ok(())`
    /// - `ierr == 0`: success → returns `Ok(())`
    fn check_err(ierr: i32, herr: &[i8]) -> Result<()> {
        if ierr > 0 {
            return Err(RefpropError::Refprop {
                code: ierr,
                message: from_c_string(herr),
            });
        }
        if ierr < 0 {
            // REFPROP warning – result may still be usable but log it.
            eprintln!("[refprop] warning {}: {}", ierr, from_c_string(herr));
        }
        Ok(())
    }
}
