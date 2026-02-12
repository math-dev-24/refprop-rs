//! Low-level FFI bindings for NIST REFPROP.
//!
//! This module dynamically loads the REFPROP shared library (DLL/so)
//! at runtime via [`libloading`] and pre-resolves all function pointers
//! for zero-overhead calls.

#![allow(non_snake_case)]

use std::os::raw::{c_char, c_double, c_int, c_long};
use std::path::Path;

use libloading::Library;

// ── REFPROP constants ───────────────────────────────────────────────
pub const REFPROP_STRLEN: usize = 255;
pub const REFPROP_FILESTR: usize = 10000;
pub const REFPROP_NC_MAX: usize = 20;

// ── Error type ──────────────────────────────────────────────────────
#[derive(Debug)]
pub enum RefpropSysError {
    /// The DLL/so could not be found or loaded.
    LibraryLoadFailed(String),
    /// A required symbol was not found in the library.
    SymbolNotFound(String),
}

impl std::fmt::Display for RefpropSysError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LibraryLoadFailed(msg) => write!(f, "REFPROP library load failed: {msg}"),
            Self::SymbolNotFound(sym) => {
                write!(f, "Symbol not found in REFPROP library: {sym}")
            }
        }
    }
}

impl std::error::Error for RefpropSysError {}

// ── Function pointer type aliases ───────────────────────────────────
// These match the Fortran calling convention used by REFPROP.
// Grouped by signature similarity for readability.

/// SETPATHdll(hpath, length)
type FnSetpath = unsafe extern "C" fn(*const c_char, c_long);

/// SETUPdll(nc, hfld, hfmix, hrf, ierr, herr, len...)
type FnSetup = unsafe extern "C" fn(
    *const c_int,
    *const c_char,
    *const c_char,
    *const c_char,
    *mut c_int,
    *mut c_char,
    c_long,
    c_long,
    c_long,
    c_long,
);

/// TPFLSHdll / PHFLSHdll / PSFLSHdll – all share the same signature:
/// (in1, in2, z, out1..out12, ierr, herr, herr_length)
type FnFlash = unsafe extern "C" fn(
    *const c_double,
    *const c_double,
    *const c_double,
    *mut c_double,
    *mut c_double,
    *mut c_double,
    *mut c_double,
    *mut c_double,
    *mut c_double,
    *mut c_double,
    *mut c_double,
    *mut c_double,
    *mut c_double,
    *mut c_double,
    *mut c_double,
    *mut c_int,
    *mut c_char,
    c_long,
);

/// SATTdll / SATPdll – same signature:
/// (in, z, kph, out1..out5, ierr, herr, herr_length)
type FnSat = unsafe extern "C" fn(
    *const c_double,
    *const c_double,
    *const c_int,
    *mut c_double,
    *mut c_double,
    *mut c_double,
    *mut c_double,
    *mut c_double,
    *mut c_int,
    *mut c_char,
    c_long,
);

/// CRITPdll(z, tc, pc, dc, ierr, herr, herr_length)
type FnCritp = unsafe extern "C" fn(
    *const c_double,
    *mut c_double,
    *mut c_double,
    *mut c_double,
    *mut c_int,
    *mut c_char,
    c_long,
);

/// TRNPRPdll(t, d, z, eta, tcx, ierr, herr, herr_length)
type FnTrnprp = unsafe extern "C" fn(
    *const c_double,
    *const c_double,
    *const c_double,
    *mut c_double,
    *mut c_double,
    *mut c_int,
    *mut c_char,
    c_long,
);

/// SETMIXdll(hmxnme, hfmix, hrf, nc, hfld, z, ierr, herr, len...)
type FnSetmix = unsafe extern "C" fn(
    *const c_char,
    *const c_char,
    *const c_char,
    *mut c_int,
    *mut c_char,
    *mut c_double,
    *mut c_int,
    *mut c_char,
    c_long,
    c_long,
    c_long,
    c_long,
    c_long,
);

/// THERMdll(t, d, z, p, e, h, s, cv, cp, w, hjt)
type FnTherm = unsafe extern "C" fn(
    *const c_double,
    *const c_double,
    *const c_double,
    *mut c_double,
    *mut c_double,
    *mut c_double,
    *mut c_double,
    *mut c_double,
    *mut c_double,
    *mut c_double,
    *mut c_double,
);

/// INFOdll(icomp, wmm, ttrp, tnbpt, tc, pc, dc, zc, acf, dip, rgas)
type FnInfo = unsafe extern "C" fn(
    *const c_int,
    *mut c_double,
    *mut c_double,
    *mut c_double,
    *mut c_double,
    *mut c_double,
    *mut c_double,
    *mut c_double,
    *mut c_double,
    *mut c_double,
    *mut c_double,
);

// ── Dynamic library wrapper ─────────────────────────────────────────

/// Holds a dynamically-loaded REFPROP shared library with **pre-resolved
/// function pointers** for zero-overhead calls.
///
/// All function symbols are resolved once at construction time.  If any
/// required symbol is missing the constructor returns an error instead
/// of panicking later.
///
/// All methods are `unsafe` because they forward raw pointers to Fortran
/// code that cannot be verified by the Rust compiler.
pub struct RefpropLibrary {
    /// The underlying library handle.  Must stay alive to keep the DLL
    /// loaded and the function pointers valid.
    _lib: Library,

    // ── Cached function pointers ────────────────────────────────────
    fn_setpath: FnSetpath,
    fn_setup: FnSetup,
    fn_tpflsh: FnFlash,
    fn_phflsh: FnFlash,
    fn_psflsh: FnFlash,
    fn_satt: FnSat,
    fn_satp: FnSat,
    fn_critp: FnCritp,
    fn_trnprp: FnTrnprp,
    fn_setmix: FnSetmix,
    fn_therm: FnTherm,
    fn_info: FnInfo,
}

impl RefpropLibrary {
    // ── Symbol resolution ───────────────────────────────────────────

    /// Resolve a single symbol from the library as a typed function
    /// pointer.  Returns `Err(SymbolNotFound)` if the symbol is absent.
    fn resolve<T: Copy>(lib: &Library, name: &[u8]) -> Result<T, RefpropSysError> {
        // SAFETY: We are loading a known symbol name from a REFPROP DLL.
        // The caller (resolve_all) ensures all type aliases match the
        // actual Fortran calling convention.
        let sym: libloading::Symbol<T> = unsafe { lib.get(name) }.map_err(|_| {
            // Strip trailing \0 for display.
            let display =
                String::from_utf8_lossy(&name[..name.len().saturating_sub(1)]).to_string();
            RefpropSysError::SymbolNotFound(display)
        })?;
        Ok(*sym)
    }

    /// Resolve **all** required REFPROP symbols from an already-loaded
    /// library.  Fails on the first missing symbol.
    fn resolve_all(lib: Library) -> Result<Self, RefpropSysError> {
        Ok(Self {
            fn_setpath: Self::resolve(&lib, b"SETPATHdll\0")?,
            fn_setup: Self::resolve(&lib, b"SETUPdll\0")?,
            fn_tpflsh: Self::resolve(&lib, b"TPFLSHdll\0")?,
            fn_phflsh: Self::resolve(&lib, b"PHFLSHdll\0")?,
            fn_psflsh: Self::resolve(&lib, b"PSFLSHdll\0")?,
            fn_satt: Self::resolve(&lib, b"SATTdll\0")?,
            fn_satp: Self::resolve(&lib, b"SATPdll\0")?,
            fn_critp: Self::resolve(&lib, b"CRITPdll\0")?,
            fn_trnprp: Self::resolve(&lib, b"TRNPRPdll\0")?,
            fn_setmix: Self::resolve(&lib, b"SETMIXdll\0")?,
            fn_therm: Self::resolve(&lib, b"THERMdll\0")?,
            fn_info: Self::resolve(&lib, b"INFOdll\0")?,
            _lib: lib,
        })
    }

    // ── Constructors ────────────────────────────────────────────────

    /// Try to load the REFPROP shared library from a **directory** that
    /// contains the DLL / .so.  Common file names are tried automatically.
    ///
    /// On 64-bit Windows the 64-bit DLL (`REFPRP64.DLL`) is tried first.
    /// If a candidate file exists but cannot be loaded (e.g. architecture
    /// mismatch), the next candidate is tried.
    ///
    /// All required symbols are resolved eagerly.  If any symbol is
    /// missing, an error is returned immediately.
    pub fn load_from_dir(dir: &Path) -> Result<Self, RefpropSysError> {
        // Order matters: prefer 64-bit DLL on 64-bit targets.
        let candidates: &[&str] = if cfg!(target_os = "windows") {
            if cfg!(target_pointer_width = "64") {
                &["REFPRP64.DLL", "REFPROP.DLL", "refprop.dll"]
            } else {
                &["REFPROP.DLL", "refprop.dll", "REFPRP64.DLL"]
            }
        } else if cfg!(target_os = "macos") {
            &["librefprop.dylib", "libREFPROP.dylib"]
        } else {
            &["librefprop.so", "libREFPROP.so"]
        };

        let mut errors = Vec::new();

        // 1. Try full paths inside the directory.
        //    If a file exists but fails to load, keep trying the rest.
        for name in candidates {
            let full = dir.join(name);
            if full.exists() {
                match unsafe { Library::new(&full) } {
                    Ok(lib) => return Self::resolve_all(lib),
                    Err(e) => {
                        errors.push(format!("{}: {e}", full.display()));
                    }
                }
            }
        }

        // 2. Fall back to system-wide search (PATH / LD_LIBRARY_PATH)
        for name in candidates {
            if let Ok(lib) = unsafe { Library::new(*name) } {
                return Self::resolve_all(lib);
            }
        }

        let detail = if errors.is_empty() {
            format!(
                "No REFPROP library found in {} (tried: {candidates:?})",
                dir.display()
            )
        } else {
            format!(
                "REFPROP library found but could not be loaded:\n  - {}",
                errors.join("\n  - ")
            )
        };
        Err(RefpropSysError::LibraryLoadFailed(detail))
    }

    /// Load the REFPROP shared library from an **exact file path**.
    pub fn load_from_file(path: &Path) -> Result<Self, RefpropSysError> {
        let lib = unsafe { Library::new(path) }
            .map_err(|e| RefpropSysError::LibraryLoadFailed(format!("{}: {e}", path.display())))?;
        Self::resolve_all(lib)
    }

    // ── REFPROP function wrappers ───────────────────────────────────
    //
    // Each method calls the pre-resolved function pointer directly.
    // No symbol lookup occurs at call time – this is the key
    // performance improvement over the previous design.

    /// Set the path where REFPROP will look for fluid files, mixture
    /// files, etc.
    pub unsafe fn SETPATHdll(&self, hpath: *const c_char, length: c_long) {
        unsafe { (self.fn_setpath)(hpath, length) };
    }

    /// Set up a fluid or mixture for subsequent calculations.
    pub unsafe fn SETUPdll(
        &self,
        nc: *const c_int,
        hfld: *const c_char,
        hfmix: *const c_char,
        hrf: *const c_char,
        ierr: *mut c_int,
        herr: *mut c_char,
        hfld_length: c_long,
        hfmix_length: c_long,
        hrf_length: c_long,
        herr_length: c_long,
    ) {
        unsafe {
            (self.fn_setup)(
                nc,
                hfld,
                hfmix,
                hrf,
                ierr,
                herr,
                hfld_length,
                hfmix_length,
                hrf_length,
                herr_length,
            );
        }
    }

    /// Temperature-pressure flash calculation.
    pub unsafe fn TPFLSHdll(
        &self,
        t: *const c_double,
        p: *const c_double,
        z: *const c_double,
        d: *mut c_double,
        dl: *mut c_double,
        dv: *mut c_double,
        x: *mut c_double,
        y: *mut c_double,
        q: *mut c_double,
        e: *mut c_double,
        h: *mut c_double,
        s: *mut c_double,
        cv: *mut c_double,
        cp: *mut c_double,
        w: *mut c_double,
        ierr: *mut c_int,
        herr: *mut c_char,
        herr_length: c_long,
    ) {
        unsafe {
            (self.fn_tpflsh)(
                t,
                p,
                z,
                d,
                dl,
                dv,
                x,
                y,
                q,
                e,
                h,
                s,
                cv,
                cp,
                w,
                ierr,
                herr,
                herr_length,
            );
        }
    }

    /// Pressure-enthalpy flash calculation.
    pub unsafe fn PHFLSHdll(
        &self,
        p: *const c_double,
        h: *const c_double,
        z: *const c_double,
        t: *mut c_double,
        d: *mut c_double,
        dl: *mut c_double,
        dv: *mut c_double,
        x: *mut c_double,
        y: *mut c_double,
        q: *mut c_double,
        e: *mut c_double,
        s: *mut c_double,
        cv: *mut c_double,
        cp: *mut c_double,
        w: *mut c_double,
        ierr: *mut c_int,
        herr: *mut c_char,
        herr_length: c_long,
    ) {
        unsafe {
            (self.fn_phflsh)(
                p,
                h,
                z,
                t,
                d,
                dl,
                dv,
                x,
                y,
                q,
                e,
                s,
                cv,
                cp,
                w,
                ierr,
                herr,
                herr_length,
            );
        }
    }

    /// Pressure-entropy flash calculation.
    pub unsafe fn PSFLSHdll(
        &self,
        p: *const c_double,
        s: *const c_double,
        z: *const c_double,
        t: *mut c_double,
        d: *mut c_double,
        dl: *mut c_double,
        dv: *mut c_double,
        x: *mut c_double,
        y: *mut c_double,
        q: *mut c_double,
        e: *mut c_double,
        h: *mut c_double,
        cv: *mut c_double,
        cp: *mut c_double,
        w: *mut c_double,
        ierr: *mut c_int,
        herr: *mut c_char,
        herr_length: c_long,
    ) {
        unsafe {
            (self.fn_psflsh)(
                p,
                s,
                z,
                t,
                d,
                dl,
                dv,
                x,
                y,
                q,
                e,
                h,
                cv,
                cp,
                w,
                ierr,
                herr,
                herr_length,
            );
        }
    }

    /// Saturation properties at a given temperature.
    pub unsafe fn SATTdll(
        &self,
        t: *const c_double,
        z: *const c_double,
        kph: *const c_int,
        p: *mut c_double,
        dl: *mut c_double,
        dv: *mut c_double,
        x: *mut c_double,
        y: *mut c_double,
        ierr: *mut c_int,
        herr: *mut c_char,
        herr_length: c_long,
    ) {
        unsafe { (self.fn_satt)(t, z, kph, p, dl, dv, x, y, ierr, herr, herr_length) };
    }

    /// Saturation properties at a given pressure.
    pub unsafe fn SATPdll(
        &self,
        p: *const c_double,
        z: *const c_double,
        kph: *const c_int,
        t: *mut c_double,
        dl: *mut c_double,
        dv: *mut c_double,
        x: *mut c_double,
        y: *mut c_double,
        ierr: *mut c_int,
        herr: *mut c_char,
        herr_length: c_long,
    ) {
        unsafe { (self.fn_satp)(p, z, kph, t, dl, dv, x, y, ierr, herr, herr_length) };
    }

    /// Critical-point properties.
    pub unsafe fn CRITPdll(
        &self,
        z: *const c_double,
        tcrit: *mut c_double,
        pcrit: *mut c_double,
        dcrit: *mut c_double,
        ierr: *mut c_int,
        herr: *mut c_char,
        herr_length: c_long,
    ) {
        unsafe { (self.fn_critp)(z, tcrit, pcrit, dcrit, ierr, herr, herr_length) };
    }

    /// Transport properties (viscosity, thermal conductivity).
    pub unsafe fn TRNPRPdll(
        &self,
        t: *const c_double,
        d: *const c_double,
        z: *const c_double,
        eta: *mut c_double,
        tcx: *mut c_double,
        ierr: *mut c_int,
        herr: *mut c_char,
        herr_length: c_long,
    ) {
        unsafe { (self.fn_trnprp)(t, d, z, eta, tcx, ierr, herr, herr_length) };
    }

    /// Load a predefined mixture from a `.MIX` file.
    ///
    /// Returns the number of components (`nc`), the fluid file string
    /// (`hfld`), and the molar composition array (`z`).
    pub unsafe fn SETMIXdll(
        &self,
        hmxnme: *const c_char,
        hfmix: *const c_char,
        hrf: *const c_char,
        nc: *mut c_int,
        hfld: *mut c_char,
        z: *mut c_double,
        ierr: *mut c_int,
        herr: *mut c_char,
        hmxnme_length: c_long,
        hfmix_length: c_long,
        hrf_length: c_long,
        hfld_length: c_long,
        herr_length: c_long,
    ) {
        unsafe {
            (self.fn_setmix)(
                hmxnme,
                hfmix,
                hrf,
                nc,
                hfld,
                z,
                ierr,
                herr,
                hmxnme_length,
                hfmix_length,
                hrf_length,
                hfld_length,
                herr_length,
            );
        }
    }

    /// Compute thermodynamic properties from temperature and density.
    ///
    /// No error return – REFPROP always produces a result.
    pub unsafe fn THERMdll(
        &self,
        t: *const c_double,
        d: *const c_double,
        z: *const c_double,
        p: *mut c_double,
        e: *mut c_double,
        h: *mut c_double,
        s: *mut c_double,
        cv: *mut c_double,
        cp: *mut c_double,
        w: *mut c_double,
        hjt: *mut c_double,
    ) {
        unsafe { (self.fn_therm)(t, d, z, p, e, h, s, cv, cp, w, hjt) };
    }

    /// Fluid information (molar mass, triple point, etc.).
    pub unsafe fn INFOdll(
        &self,
        icomp: *const c_int,
        wmm: *mut c_double,
        ttrp: *mut c_double,
        tnbpt: *mut c_double,
        tc: *mut c_double,
        pc: *mut c_double,
        dc: *mut c_double,
        zc: *mut c_double,
        acf: *mut c_double,
        dip: *mut c_double,
        rgas: *mut c_double,
    ) {
        unsafe { (self.fn_info)(icomp, wmm, ttrp, tnbpt, tc, pc, dc, zc, acf, dip, rgas) };
    }
}

// ── String helpers ──────────────────────────────────────────────────

/// Convert a Rust `&str` into a zero-padded `Vec<c_char>` of length
/// `max_len`, suitable for passing to a Fortran routine.
pub fn to_c_string(s: &str, max_len: usize) -> Vec<c_char> {
    let mut buffer = vec![0 as c_char; max_len];
    let bytes = s.as_bytes();
    let copy_len = bytes.len().min(max_len - 1);
    for i in 0..copy_len {
        buffer[i] = bytes[i] as c_char;
    }
    buffer
}

/// Convert a null-terminated (or fully-filled) Fortran `c_char` buffer
/// back into a trimmed Rust `String`.
pub fn from_c_string(buffer: &[c_char]) -> String {
    let bytes: Vec<u8> = buffer
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8)
        .collect();
    String::from_utf8_lossy(&bytes).trim().to_string()
}
