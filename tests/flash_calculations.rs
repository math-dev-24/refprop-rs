use refprop::{Fluid, UnitSystem};

// ═══════════════════════════════════════════════════════════════════
//  Flash TP (Temperature-Pressure)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn r134a_tp_flash_subcooled() {
    let r134a = Fluid::with_units("R134A", UnitSystem::engineering()).unwrap();
    let props = r134a.props_tp(20.0, 10.0).unwrap();
    // À 20 °C et 10 bar, R134A est liquide sous-refroidi
    // quality < 0 ou > 100 signifie monophasique
    assert!(
        props.quality < 0.0 || props.quality > 100.0,
        "R134A at 20 °C, 10 bar should be subcooled liquid, Q = {:.2}",
        props.quality
    );
    // Densité liquide ~ 1200 kg/m³
    assert!(
        props.density > 1000.0,
        "Subcooled liquid density should be high, got {:.2} kg/m³",
        props.density
    );
}

#[test]
fn r134a_tp_flash_superheated() {
    let r134a = Fluid::with_units("R134A", UnitSystem::engineering()).unwrap();
    let props = r134a.props_tp(50.0, 2.0).unwrap();
    // À 50 °C et 2 bar, R134A est vapeur surchauffée
    assert!(
        props.quality < 0.0 || props.quality > 100.0,
        "R134A at 50 °C, 2 bar should be superheated vapor, Q = {:.2}",
        props.quality
    );
    // Densité vapeur assez faible
    assert!(
        props.density < 100.0,
        "Superheated vapor density should be low, got {:.2} kg/m³",
        props.density
    );
}

#[test]
fn tp_flash_cp_greater_than_cv() {
    // Cp ≥ Cv pour tout fluide
    let r134a = Fluid::with_units("R134A", UnitSystem::engineering()).unwrap();
    let props = r134a.props_tp(25.0, 5.0).unwrap();
    assert!(
        props.cp >= props.cv,
        "Cp ({:.4}) should be >= Cv ({:.4})",
        props.cp,
        props.cv
    );
}

#[test]
fn tp_flash_positive_sound_speed() {
    let r134a = Fluid::with_units("R134A", UnitSystem::engineering()).unwrap();
    let props = r134a.props_tp(25.0, 5.0).unwrap();
    assert!(
        props.sound_speed > 0.0,
        "Sound speed should be positive, got {:.4}",
        props.sound_speed
    );
}

// ═══════════════════════════════════════════════════════════════════
//  Flash TQ (Temperature-Quality)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn r134a_tq_flash_saturated_liquid() {
    let r134a = Fluid::with_units("R134A", UnitSystem::engineering()).unwrap();
    let props = r134a.props_tq(0.0, 0.0).unwrap();
    // Pression de saturation ~2.93 bar
    assert!(
        (props.pressure - 2.93).abs() < 0.1,
        "P_sat(0 °C) expected ≈ 2.93 bar, got {:.4}",
        props.pressure
    );
    // Qualité = 0%
    assert!(
        props.quality.abs() < 1.0,
        "Q should be ~0%, got {:.2}%",
        props.quality
    );
}

#[test]
fn r134a_tq_flash_saturated_vapor() {
    let r134a = Fluid::with_units("R134A", UnitSystem::engineering()).unwrap();
    let props = r134a.props_tq(0.0, 100.0).unwrap();
    // Qualité = 100%
    assert!(
        (props.quality - 100.0).abs() < 1.0,
        "Q should be ~100%, got {:.2}%",
        props.quality
    );
}

#[test]
fn r134a_tq_flash_two_phase() {
    let r134a = Fluid::with_units("R134A", UnitSystem::engineering()).unwrap();
    let liq = r134a.props_tq(0.0, 0.0).unwrap();
    let vap = r134a.props_tq(0.0, 100.0).unwrap();
    // Le liquide doit être plus dense que la vapeur
    assert!(
        liq.density > vap.density,
        "Liquid density ({:.2}) should be > vapor density ({:.2})",
        liq.density,
        vap.density
    );
    // L'enthalpie de la vapeur doit être supérieure à celle du liquide
    assert!(
        vap.enthalpy > liq.enthalpy,
        "Vapor enthalpy ({:.2}) should be > liquid enthalpy ({:.2})",
        vap.enthalpy,
        liq.enthalpy
    );
}

// ═══════════════════════════════════════════════════════════════════
//  Flash PQ (Pressure-Quality)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn r134a_pq_flash_at_3bar() {
    let r134a = Fluid::with_units("R134A", UnitSystem::engineering()).unwrap();
    let props = r134a.props_pq(3.0, 0.0).unwrap();
    // R134A Tsat(3 bar) ≈ 0.7 °C
    assert!(
        (props.temperature - 0.7).abs() < 1.5,
        "R134A Tsat(3 bar) expected ≈ 0.7 °C, got {:.4}",
        props.temperature
    );
}
