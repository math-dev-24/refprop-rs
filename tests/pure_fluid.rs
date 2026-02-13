use refprop::{Fluid, UnitSystem};

// ═══════════════════════════════════════════════════════════════════
//  R134A — properties using engineering units (°C, bar, kg/m³, kJ/kg)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn r134a_saturation_pressure_at_0c() {
    // R134A: Psat(0 °C) ≈ 2.93 bar
    let r134a = Fluid::with_units("R134A", UnitSystem::engineering()).unwrap();
    let p = r134a.get("P", "T", 0.0, "Q", 0.0).unwrap();
    assert!(
        (p - 2.93).abs() < 0.1,
        "R134A Psat(0 °C) expected ≈ 2.93 bar, got {p:.4}"
    );
}

#[test]
fn r134a_saturation_pressure_at_minus26c() {
    // R134A: Psat(-26.07 °C) ≈ 1.0 bar (point d'ébullition normal)
    let r134a = Fluid::with_units("R134A", UnitSystem::engineering()).unwrap();
    let p = r134a.get("P", "T", -26.07, "Q", 0.0).unwrap();
    assert!(
        (p - 1.0).abs() < 0.05,
        "R134A Psat(-26 °C) expected ≈ 1.0 bar, got {p:.4}"
    );
}

#[test]
fn r134a_density_saturated_vapor_at_0c() {
    // R134A: D_vap(0 °C) ≈ 14.4 kg/m³
    let r134a = Fluid::with_units("R134A", UnitSystem::engineering()).unwrap();
    let d = r134a.get("D", "T", 0.0, "Q", 100.0).unwrap();
    assert!(
        (d - 14.4).abs() < 1.0,
        "R134A D_vap(0 °C) expected ≈ 14.4 kg/m³, got {d:.4}"
    );
}

#[test]
fn r134a_density_saturated_liquid_at_0c() {
    // R134A: D_liq(0 °C) ≈ 1295 kg/m³
    let r134a = Fluid::with_units("R134A", UnitSystem::engineering()).unwrap();
    let d = r134a.get("D", "T", 0.0, "Q", 0.0).unwrap();
    assert!(
        (d - 1295.0).abs() < 10.0,
        "R134A D_liq(0 °C) expected ≈ 1295 kg/m³, got {d:.4}"
    );
}

#[test]
fn r134a_enthalpy_saturated_vapor_at_0c() {
    // R134A: H_vap(0 °C) ≈ 398 kJ/kg
    let r134a = Fluid::with_units("R134A", UnitSystem::engineering()).unwrap();
    let h = r134a.get("H", "T", 0.0, "Q", 100.0).unwrap();
    assert!(
        (h - 398.0).abs() < 5.0,
        "R134A H_vap(0 °C) expected ≈ 398 kJ/kg, got {h:.4}"
    );
}

// ═══════════════════════════════════════════════════════════════════
//  CO2 — properties
// ═══════════════════════════════════════════════════════════════════

#[test]
fn co2_saturation_pressure_at_0c() {
    // CO2: Psat(0 °C) ≈ 34.85 bar
    let co2 = Fluid::with_units("CO2", UnitSystem::engineering()).unwrap();
    let p = co2.get("P", "T", 0.0, "Q", 0.0).unwrap();
    assert!(
        (p - 34.85).abs() < 0.5,
        "CO2 Psat(0 °C) expected ≈ 34.85 bar, got {p:.4}"
    );
}

#[test]
fn co2_density_superheated() {
    // CO2 vapeur surchauffée à 50 °C, 20 bar — densité ~36 kg/m³
    let co2 = Fluid::with_units("CO2", UnitSystem::engineering()).unwrap();
    let d = co2.get("D", "T", 50.0, "P", 20.0).unwrap();
    assert!(
        d > 20.0 && d < 60.0,
        "CO2 density(50 °C, 20 bar) expected ~36 kg/m³, got {d:.4}"
    );
}

// ═══════════════════════════════════════════════════════════════════
//  Water — properties
// ═══════════════════════════════════════════════════════════════════

#[test]
fn water_boiling_point_at_1atm() {
    // Water: Psat(100 °C) ≈ 1.01325 bar
    let water = Fluid::with_units("WATER", UnitSystem::engineering()).unwrap();
    let p = water.get("P", "T", 100.0, "Q", 0.0).unwrap();
    assert!(
        (p - 1.01325).abs() < 0.02,
        "Water Psat(100 °C) expected ≈ 1.013 bar, got {p:.4}"
    );
}

#[test]
fn water_density_liquid_at_20c() {
    // Water density at 20 °C, 1 bar ≈ 998 kg/m³
    let water = Fluid::with_units("WATER", UnitSystem::engineering()).unwrap();
    let d = water.get("D", "T", 20.0, "P", 1.0).unwrap();
    assert!(
        (d - 998.0).abs() < 5.0,
        "Water D(20 °C, 1 bar) expected ≈ 998 kg/m³, got {d:.4}"
    );
}

#[test]
fn water_latent_heat_at_100c() {
    // Chaleur latente de vaporisation à 100 °C ≈ 2257 kJ/kg
    let water = Fluid::with_units("WATER", UnitSystem::engineering()).unwrap();
    let h_vap = water.get("H", "T", 100.0, "Q", 100.0).unwrap();
    let h_liq = water.get("H", "T", 100.0, "Q", 0.0).unwrap();
    let latent = h_vap - h_liq;
    assert!(
        (latent - 2257.0).abs() < 15.0,
        "Water latent heat(100 °C) expected ≈ 2257 kJ/kg, got {latent:.4}"
    );
}
