# refprop-rs

Safe Rust bindings for [NIST REFPROP](https://www.nist.gov/srd/refprop) -- thermodynamic and transport properties of refrigerants, pure fluids, and mixtures.

## Features

- **Pure fluids** -- `Fluid::new("R134A")`, `Fluid::new("CO2")`, ...
- **Predefined mixtures** -- `Fluid::new("R410A")` (auto-loaded from `.MIX` files)
- **Custom mixtures** -- `Fluid::mixture(&[("R32", 0.5), ("R125", 0.5)])`
- **CoolProp-style `get()`** -- `fluid.get("D", "T", 0.0, "Q", 1.0)`
- **Configurable units** -- work in **°C + bar + kg/m³ + kJ/kg**, or K + kPa, or any mix
- **Flash calculations** -- TP, PH, PS, TQ, PQ
- **Saturation, transport, critical point, fluid info**
- **Thread-safe** -- global mutex with automatic fluid re-setup
- **Dynamic loading** -- no compile-time linking, just point to your REFPROP installation

## Prerequisites

A licensed [REFPROP](https://www.nist.gov/srd/refprop) installation (v9.1 or v10).

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
refprop-rs = { git = "https://github.com/math-dev-24/refprop-rs" }
```

The library name is `refprop`, so you import it as:

```rust
use refprop::{Fluid, UnitSystem};
```

## Configuration

Tell the library where REFPROP is installed.  Create a `.env` file at
the project root (or set the environment variable directly):

```
REFPROP_PATH='C:\Program Files (x86)\REFPROP'
```

The library also checks standard install locations automatically.

## Quick start

### Engineering units (°C, bar, kg/m³, kJ/kg)

```rust
use refprop::{Fluid, UnitSystem};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let co2 = Fluid::with_units("CO2", UnitSystem::engineering())?;

    // Everything is in °C, bar, kg/m³, kJ/kg -- no manual conversion!
    let p = co2.get("P", "T", -5.0, "Q", 1.0)?;
    println!("Psat(-5 °C) = {p:.2} bar");

    let d = co2.get("D", "T", -5.0, "Q", 1.0)?;
    println!("D_vap(-5 °C) = {d:.2} kg/m³");

    let h = co2.get("H", "T", -5.0, "Q", 1.0)?;
    println!("H_vap(-5 °C) = {h:.2} kJ/kg");

    Ok(())
}
```

### REFPROP native units (K, kPa, mol/L, J/mol)

```rust
use refprop::Fluid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let r134a = Fluid::new("R134A")?;

    let d = r134a.get("D", "T", 273.15, "Q", 1.0)?;
    println!("density = {d:.6} mol/L");

    let props = r134a.props_tp(300.0, 500.0)?;
    println!("{props}");

    Ok(())
}
```

## Unit system

Choose your preferred units at construction time.  All inputs and
outputs are automatically converted.

### Presets

| Preset                      | T   | P   | D      | H      | S          | Viscosity | Conductivity |
|-----------------------------|-----|-----|--------|--------|------------|-----------|--------------|
| `UnitSystem::refprop()`     | K   | kPa | mol/L  | J/mol  | J/(mol·K)  | µPa·s     | W/(m·K)      |
| `UnitSystem::engineering()` | °C  | bar | kg/m³  | kJ/kg  | kJ/(kg·K)  | µPa·s     | W/(m·K)      |
| `UnitSystem::si()`          | K   | Pa  | kg/m³  | J/kg   | J/(kg·K)   | Pa·s      | W/(m·K)      |

### Custom builder

Pick only the units you care about; the rest stay at REFPROP defaults:

```rust
use refprop::{Fluid, UnitSystem, TempUnit, PressUnit};

let units = UnitSystem::new()
    .temperature(TempUnit::Celsius)
    .pressure(PressUnit::Bar);
    // density stays mol/L, energy stays J/mol, etc.

let fluid = Fluid::with_units("R134A", units)?;
let sat = fluid.saturation_t(0.0)?;  // 0 °C directly
println!("Psat = {:.2} bar", sat.pressure);
```

### Available unit choices

| Property         | Options                                        |
|------------------|------------------------------------------------|
| Temperature      | `Kelvin`, `Celsius`, `Fahrenheit`              |
| Pressure         | `KPa`, `Bar`, `MPa`, `Pa`, `Atm`, `Psi`       |
| Density          | `MolPerL`, `KgPerM3`                           |
| Energy/Enthalpy  | `JPerMol`, `KJPerKg`, `JPerKg`                 |
| Entropy/Cv/Cp    | `JPerMolK`, `KJPerKgK`, `JPerKgK`             |
| Viscosity        | `MicroPaS`, `MilliPaS`, `PaS`                 |
| Conductivity     | `WPerMK`, `MilliWPerMK`                        |

## Mixtures

```rust
use refprop::{Fluid, UnitSystem};

// Predefined mixture (from .MIX file)
let r410a = Fluid::with_units("R410A", UnitSystem::engineering())?;

// Custom composition
let r454c = Fluid::mixture_with_units(
    &[("R32", 0.215), ("R1234YF", 0.785)],
    UnitSystem::engineering(),
)?;

let p = r454c.get("P", "T", 0.0, "Q", 0.0)?;
println!("R454C Psat(0 °C) = {p:.2} bar");
```

## `get()` -- generic property lookup

```rust
// get(output, key1, val1, key2, val2) -> f64
let density = fluid.get("D", "T", 25.0, "P", 10.0)?;
```

### Input pairs (order-independent)

| Pair      | Description              |
|-----------|--------------------------|
| `T`, `P`  | Temperature + Pressure   |
| `P`, `H`  | Pressure + Enthalpy      |
| `P`, `S`  | Pressure + Entropy       |
| `T`, `Q`  | Temperature + Quality    |
| `P`, `Q`  | Pressure + Quality       |

### Output keys

| Key   | Property              |
|-------|-----------------------|
| `T`   | Temperature           |
| `P`   | Pressure              |
| `D`   | Density               |
| `H`   | Enthalpy              |
| `S`   | Entropy               |
| `Q`   | Quality (vapor frac.) |
| `Cv`  | Heat capacity (v)     |
| `Cp`  | Heat capacity (p)     |
| `W`   | Speed of sound        |
| `E`   | Internal energy       |
| `ETA` | Dynamic viscosity     |
| `TCX` | Thermal conductivity  |

Units depend on the `UnitSystem` you chose at construction time.

## Flash & saturation methods

All methods respect the configured unit system.

```rust
let props = fluid.props_tp(25.0, 10.0)?;   // TP flash
let props = fluid.props_ph(10.0, 250.0)?;  // PH flash
let props = fluid.props_ps(10.0, 1.2)?;    // PS flash
let props = fluid.props_tq(0.0, 1.0)?;     // TQ flash (saturation)
let props = fluid.props_pq(5.0, 0.0)?;     // PQ flash (saturation)

let sat = fluid.saturation_t(0.0)?;        // saturation at T
let sat = fluid.saturation_p(5.0)?;        // saturation at P

let crit = fluid.critical_point()?;        // Tc, Pc, Dc
let trn  = fluid.transport(25.0, d)?;      // viscosity, conductivity
let info = fluid.info()?;                  // molar mass, Ttrp, Tnbp, ...
```

## Project structure

```
refprop-rs/
├── Cargo.toml              single crate: refprop-rs
├── src/
│   ├── lib.rs              public API & re-exports
│   ├── fluid.rs            Fluid struct (high-level API)
│   ├── converter.rs        UnitSystem + Converter
│   ├── sys.rs              low-level FFI (libloading)
│   ├── error.rs            error types
│   ├── properties.rs       result structs
│   └── backend/
│       └── refprop.rs      REFPROP backend (flash, sat, etc.)
└── examples/
    ├── demo.rs             engineering units showcase
    ├── simple.rs           pure fluid, native units
    └── mixture.rs          predefined & custom mixtures
```

| Module              | Role                                           |
|---------------------|------------------------------------------------|
| `sys`               | Dynamic DLL loading + raw FFI function wrappers |
| `converter`         | `UnitSystem` + `Converter` (unit conversion)    |
| `fluid`             | High-level API: `Fluid`, `get()`, flash, units  |
| `backend::refprop`  | Core REFPROP calls, global state management     |

## License

MIT
