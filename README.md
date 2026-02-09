# refprop-rs

Safe Rust bindings for [NIST REFPROP](https://www.nist.gov/srd/refprop) — thermodynamic and transport properties of refrigerants, pure fluids, and mixtures.

## Features

- **Pure fluids** — `Fluid::new("R134A")`, `Fluid::new("CO2")`, …
- **Predefined mixtures** — `Fluid::new("R410A")` (auto-loaded from `.MIX` files)
- **Custom mixtures** — `Fluid::mixture(&[("R32", 0.5), ("R125", 0.5)])`
- **CoolProp-style `get()`** — `fluid.get("D", "T", 273.15, "Q", 1.0)`
- **Flash calculations** — TP, PH, PS, TQ, PQ
- **Saturation, transport, critical point, fluid info**
- **Thread-safe** — global mutex with automatic fluid re-setup
- **Dynamic loading** — no compile-time linking, just point to your REFPROP installation

## Prerequisites

A licensed [REFPROP](https://www.nist.gov/srd/refprop) installation (v9.1 or v10).

## Quick start

Add to your `Cargo.toml`:

```toml
[dependencies]
refprop = { path = "path/to/refprop-rs/refprop" }
```

Create a `.env` file (or set the environment variable):

```
REFPROP_PATH='C:\Program Files (x86)\REFPROP'
```

```rust
use refprop::Fluid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let r134a = Fluid::new("R134A")?;

    // CoolProp-style: density of saturated vapor at 0 °C
    let d = r134a.get("D", "T", 273.15, "Q", 1.0)?;
    println!("density = {d:.6} mol/L");

    // Full flash calculation
    let props = r134a.props_tp(300.0, 500.0)?;
    println!("{props}");

    Ok(())
}
```

## Mixtures

```rust
// Predefined (from .MIX file)
let r410a = Fluid::new("R410A")?;

// Custom composition
let mix = Fluid::mixture(&[("R32", 0.5), ("R125", 0.5)])?;

let p = mix.get("P", "T", 273.15, "Q", 0.0)?;
```

## `get()` — generic property lookup

```rust
fluid.get(output, key1, val1, key2, val2)
```

| Input pairs | Description |
|-------------|-------------|
| `T`, `P` | Temperature + Pressure |
| `P`, `H` | Pressure + Enthalpy |
| `P`, `S` | Pressure + Entropy |
| `T`, `Q` | Temperature + Quality |
| `P`, `Q` | Pressure + Quality |

| Output keys | Property | Unit |
|-------------|----------|------|
| `T` | Temperature | K |
| `P` | Pressure | kPa |
| `D` | Density | mol/L |
| `H` | Enthalpy | J/mol |
| `S` | Entropy | J/(mol·K) |
| `Q` | Quality | — |
| `Cv` | Heat capacity (v) | J/(mol·K) |
| `Cp` | Heat capacity (p) | J/(mol·K) |
| `W` | Speed of sound | m/s |
| `E` | Internal energy | J/mol |
| `ETA` | Viscosity | µPa·s |
| `TCX` | Thermal conductivity | W/(m·K) |

## Crate structure

| Crate | Role |
|-------|------|
| `refprop-sys` | Low-level FFI (dynamic loading via `libloading`) |
| `refprop` | Safe, ergonomic public API |

## License

MIT
