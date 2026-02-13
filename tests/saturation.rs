use refprop::{Fluid, UnitSystem};

// ═══════════════════════════════════════════════════════════════════
//  Saturation par température
// ═══════════════════════════════════════════════════════════════════

#[test]
fn r134a_saturation_t_at_0c() {
    let r134a = Fluid::with_units("R134A", UnitSystem::engineering()).unwrap();
    let sat = r134a.saturation_t(0.0).unwrap();

    // Pression de saturation ~2.93 bar
    assert!(
        (sat.pressure - 2.93).abs() < 0.1,
        "Psat(0 °C) expected ≈ 2.93 bar, got {:.4}",
        sat.pressure
    );
    // Température retournée = celle demandée
    assert!(
        (sat.temperature - 0.0).abs() < 0.1,
        "Temperature should be ≈ 0 °C, got {:.4}",
        sat.temperature
    );
    // Densité liquide > densité vapeur
    assert!(
        sat.density_liquid > sat.density_vapor,
        "D_liq ({:.2}) should be > D_vap ({:.2})",
        sat.density_liquid,
        sat.density_vapor
    );
}

#[test]
fn r134a_saturation_t_at_25c() {
    let r134a = Fluid::with_units("R134A", UnitSystem::engineering()).unwrap();
    let sat = r134a.saturation_t(25.0).unwrap();

    // R134A: Psat(25 °C) ≈ 6.65 bar
    assert!(
        (sat.pressure - 6.65).abs() < 0.2,
        "Psat(25 °C) expected ≈ 6.65 bar, got {:.4}",
        sat.pressure
    );
}

// ═══════════════════════════════════════════════════════════════════
//  Saturation par pression
// ═══════════════════════════════════════════════════════════════════

#[test]
fn r134a_saturation_p_at_1bar() {
    let r134a = Fluid::with_units("R134A", UnitSystem::engineering()).unwrap();
    let sat = r134a.saturation_p(1.0).unwrap();

    // R134A: Tsat(1 bar) ≈ -26.07 °C
    assert!(
        (sat.temperature - (-26.07)).abs() < 1.0,
        "Tsat(1 bar) expected ≈ -26 °C, got {:.4}",
        sat.temperature
    );
}

#[test]
fn r134a_saturation_p_at_5bar() {
    let r134a = Fluid::with_units("R134A", UnitSystem::engineering()).unwrap();
    let sat = r134a.saturation_p(5.0).unwrap();

    // R134A: Tsat(5 bar) ≈ 15.7 °C
    assert!(
        (sat.temperature - 15.7).abs() < 1.0,
        "Tsat(5 bar) expected ≈ 15.7 °C, got {:.4}",
        sat.temperature
    );
    assert!(sat.density_liquid > sat.density_vapor);
}

// ═══════════════════════════════════════════════════════════════════
//  Cohérence saturation_t ↔ saturation_p
// ═══════════════════════════════════════════════════════════════════

#[test]
fn saturation_t_p_round_trip() {
    let r134a = Fluid::with_units("R134A", UnitSystem::engineering()).unwrap();

    // Obtenir P à partir de T
    let sat_t = r134a.saturation_t(10.0).unwrap();
    // Puis retrouver T à partir de P
    let sat_p = r134a.saturation_p(sat_t.pressure).unwrap();

    assert!(
        (sat_p.temperature - 10.0).abs() < 0.5,
        "Round-trip T → P → T should return ≈ 10 °C, got {:.4}",
        sat_p.temperature
    );
}
