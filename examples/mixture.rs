use refprop::{Fluid, UnitSystem};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fluid_name = "XL40";
    // ================================================================
    //  1. Predefined mixture (loaded from .MIX file)
    // ================================================================
    println!("=== {fluid_name} (predefined mixture from .MIX file) ===\n");
    let r410a = Fluid::new(fluid_name)?;

    let crit = r410a.critical_point()?;
    println!("Critical point:\n{crit}\n");

    let props = r410a.props_tp(298.15, 500.0)?;
    println!("TP flash (T=298.15 K, P=500 kPa):\n{props}\n");

    let sat = r410a.saturation_t(273.15)?;
    println!("Saturation at T=273.15 K:\n{sat}\n");

    // ================================================================
    //  2. Custom mixture with explicit composition
    //     R454C = R32 (21.5 %) + R1234yf (78.5 %)
    // ================================================================
    println!("=== R454C (custom mixture: R32 21.5% + R1234YF 78.5%) ===\n");
    let r454c = Fluid::mixture(&[("R32", 0.215), ("R1234YF", 0.785)])?;

    let props2 = r454c.props_tp(298.15, 500.0)?;
    println!("TP flash (T=298.15 K, P=500 kPa):\n{props2}\n");

    let sat2 = r454c.saturation_p(500.0)?;
    println!("Saturation at P=500 kPa:\n{sat2}\n");

    // ================================================================
    //  3. CoolProp-style generic "get" function
    // ================================================================
    println!("=== Generic get() – CoolProp style ===\n");

    // Density of R410A saturated vapor at 0 °C
    let d = r410a.get("D", "T", 273.15, "Q", 100.0)?;
    println!("R410A  D(T=273.15, Q=100) = {d:.6} mol/L");

    // Pressure of R410A at T=273.15 K, Q=0 (saturated liquid)
    let p = r410a.get("P", "T", 273.15, "Q", 0.0)?;
    println!("R410A  P(T=273.15, Q=0) = {p:.4} kPa");

    // Enthalpy of R454C at P=500 kPa, T=298.15 K
    let h = r454c.get("H", "P", 500.0, "T", 298.15)?;
    println!("R454C  H(P=500, T=298.15) = {h:.4} J/mol");

    // Viscosity of R410A at P=500 kPa, T=298.15 K
    let eta = r410a.get("ETA", "P", 500.0, "T", 298.15)?;
    println!("R410A  eta(P=500, T=298.15) = {eta:.4} µPa·s");

    // Entropy of R410A at T=280 K, Q=0.5 (two-phase)
    let s = r410a.get("S", "T", 280.0, "Q", 50.0)?;
    println!("R410A  S(T=280, Q=50) = {s:.4} J/(mol·K)");

    // ================================================================
    //  4. Zeotropic mixture — R407C bubble vs dew pressure
    //     At 20 °C: P_bubble ≈ 10.38 bar, P_dew ≈ 8.8 bar
    // ================================================================
    println!("\n=== R407C zeotropic mixture — bubble vs dew at 20 °C ===\n");
    let r407c = Fluid::with_units("R407C", UnitSystem::engineering())?;

    let p_bubble = r407c.get("P", "T", 20.0, "Q", 0.0)?;
    let p_dew    = r407c.get("P", "T", 20.0, "Q", 100.0)?;
    println!("R407C  P_bubble(T=20 °C, Q=0)   = {p_bubble:.2} bar  (expected ≈ 10.38)");
    println!("R407C  P_dew   (T=20 °C, Q=100) = {p_dew:.2} bar  (expected ≈  8.80)");

    // Quick sanity check
    assert!(
        (p_bubble - 10.38).abs() < 0.1,
        "P_bubble should be ≈ 10.38 bar, got {p_bubble:.4}"
    );
    assert!(
        (p_dew - 8.80).abs() < 0.1,
        "P_dew should be ≈ 8.80 bar, got {p_dew:.4}"
    );
    println!("✓ Both pressures match expected values.");

    Ok(())
}
