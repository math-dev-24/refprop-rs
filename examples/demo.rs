use refprop::{Fluid, UnitSystem};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ── Engineering units: °C, bar, kg/m³, kJ/kg ────────────────────
    let co2 = Fluid::with_units("CO2", UnitSystem::engineering())?;

    let crit = co2.critical_point()?;
    println!("CO2 critical point: {:.2} °C, {:.2} bar", crit.temperature, crit.pressure);

    // Saturation pressures — input directly in °C, output in bar
    let p_evp = co2.get("P", "T", -5.0, "Q", 100.0)?;
    let p_cond = co2.get("P", "T", 25.0, "Q", 100.0)?;
    println!("P_evp(-5 °C) = {p_evp:.2} bar");
    println!("P_cond(25 °C) = {p_cond:.2} bar");

    // Density of saturated vapor at -5 °C — directly in kg/m³
    let d = co2.get("D", "T", -5.0, "Q", 100.0)?;
    println!("D_vap(-5 °C) = {d:.2} kg/m³");

    // Enthalpy — directly in kJ/kg
    let h = co2.get("H", "T", -5.0, "Q", 100.0)?;
    println!("H_vap(-5 °C) = {h:.2} kJ/kg");

    // ── Custom units: °C + bar, but molar densities ─────────────────
    let r134a = Fluid::with_units("R134A",
        UnitSystem::new()
            .temperature(refprop::TempUnit::Celsius)
            .pressure(refprop::PressUnit::Bar),
    )?;

    let sat = r134a.saturation_t(0.0)?;   // 0 °C
    println!("\nR134A saturation at 0 °C:");
    println!("  P = {:.4} bar", sat.pressure);
    println!("  D_liq = {:.4} mol/L", sat.density_liquid);

    Ok(())
}
