use refprop::Fluid;

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
    //  3. CoolProp‑style generic "get" function
    // ================================================================
    println!("=== Generic get() – CoolProp style ===\n");

    // Density of R410A saturated vapor at 0 °C
    let d = r410a.get("D", "T", 273.15, "Q", 1.0)?;
    println!("R410A  D(T=273.15, Q=1) = {d:.6} mol/L");

    // Pressure of R410A at T=273.15 K, Q=0 (saturated liquid)
    let p = r410a.get("P", "T", 273.15, "Q", 0.0)?;
    println!("R410A  P(T=273.15, Q=0) = {p:.4} kPa");

    // Enthalpy of R454C at P=500 kPa, T=298.15 K
    let h = r454c.get("H", "P", 500.0, "T", 298.15)?;
    println!("R454C  H(P=500, T=298.15) = {h:.4} J/mol");

    // Viscosity of R410A at P=500 kPa, T=298.15 K
    let eta = r410a.get("ETA", "P", 500.0, "T", 298.15)?;
    println!("R410A  eta(P=500, T=298.15) = {eta:.4} µPa·s");

    // Entropy of R410A at T=280 K, Q=0.5 (two‑phase)
    let s = r410a.get("S", "T", 280.0, "Q", 0.5)?;
    println!("R410A  S(T=280, Q=0.5) = {s:.4} J/(mol·K)");

    Ok(())
}
