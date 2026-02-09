use thiserror::Error;

#[derive(Error, Debug)]
pub enum RefpropError {
    /// Error returned by a REFPROP routine (ierr > 0).
    #[error("REFPROP error {code}: {message}")]
    Refprop { code: i32, message: String },

    /// Warning returned by a REFPROP routine (ierr < 0).
    /// The result *may* still be usable.
    #[error("REFPROP warning {code}: {message}")]
    Warning { code: i32, message: String },

    /// The REFPROP DLL/so could not be loaded.
    #[error("REFPROP library not found: {0}")]
    LibraryNotFound(String),

    /// A fluid `.FLD` file was not found in the fluids directory.
    #[error("Fluid file not found: {0}")]
    FluidNotFound(String),

    /// Invalid or out‑of‑range input.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Catch‑all for calculation failures.
    #[error("Calculation failed: {0}")]
    CalculationFailed(String),
}

pub type Result<T> = std::result::Result<T, RefpropError>;
