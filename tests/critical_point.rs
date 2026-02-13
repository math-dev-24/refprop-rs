use refprop::{Fluid, UnitSystem};

// ═══════════════════════════════════════════════════════════════════
//  Point critique — valeurs connues
// ═══════════════════════════════════════════════════════════════════

#[test]
fn r134a_critical_point() {
    // R134A: Tc ≈ 101.06 °C, Pc ≈ 40.59 bar, Dc ≈ 511.9 kg/m³
    let r134a = Fluid::with_units("R134A", UnitSystem::engineering()).unwrap();
    let crit = r134a.critical_point().unwrap();

    assert!(
        (crit.temperature - 101.06).abs() < 1.0,
        "R134A Tc expected ≈ 101 °C, got {:.4}",
        crit.temperature
    );
    assert!(
        (crit.pressure - 40.59).abs() < 0.5,
        "R134A Pc expected ≈ 40.59 bar, got {:.4}",
        crit.pressure
    );
    assert!(
        (crit.density - 511.9).abs() < 10.0,
        "R134A Dc expected ≈ 512 kg/m³, got {:.4}",
        crit.density
    );
}

#[test]
fn co2_critical_point() {
    // CO2: Tc ≈ 30.98 °C, Pc ≈ 73.77 bar
    let co2 = Fluid::with_units("CO2", UnitSystem::engineering()).unwrap();
    let crit = co2.critical_point().unwrap();

    assert!(
        (crit.temperature - 30.98).abs() < 1.0,
        "CO2 Tc expected ≈ 31 °C, got {:.4}",
        crit.temperature
    );
    assert!(
        (crit.pressure - 73.77).abs() < 0.5,
        "CO2 Pc expected ≈ 73.77 bar, got {:.4}",
        crit.pressure
    );
}

#[test]
fn water_critical_point() {
    // Water: Tc ≈ 373.95 °C, Pc ≈ 220.64 bar
    let water = Fluid::with_units("WATER", UnitSystem::engineering()).unwrap();
    let crit = water.critical_point().unwrap();

    assert!(
        (crit.temperature - 373.95).abs() < 1.0,
        "Water Tc expected ≈ 374 °C, got {:.4}",
        crit.temperature
    );
    assert!(
        (crit.pressure - 220.64).abs() < 1.0,
        "Water Pc expected ≈ 220.64 bar, got {:.4}",
        crit.pressure
    );
}

#[test]
fn critical_values_are_positive() {
    let r134a = Fluid::with_units("R134A", UnitSystem::engineering()).unwrap();
    let crit = r134a.critical_point().unwrap();

    assert!(crit.pressure > 0.0, "Pc must be positive");
    assert!(crit.density > 0.0, "Dc must be positive");
    // En °C, Tc peut être négatif pour certains fluides, mais pas pour R134A
    assert!(crit.temperature > 0.0, "R134A Tc must be > 0 °C");
}
