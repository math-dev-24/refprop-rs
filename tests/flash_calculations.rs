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
//  Flash TH (Temperature-Enthalpy)
// ═══════════════════════════════════════════════════════════════════

/// TH flash in superheated region: verify round-trip consistency
/// by first computing H via TP, then recovering P via TH.
#[test]
fn r134a_th_flash_superheated() {
    let r134a = Fluid::with_units("R134A", UnitSystem::engineering()).unwrap();
    // Get reference properties at 50 °C, 5 bar (superheated vapor)
    let ref_props = r134a.props_tp(50.0, 5.0).unwrap();
    // Now recover state from (T, H)
    let props = r134a.props_th(50.0, ref_props.enthalpy).unwrap();
    assert!(
        (props.pressure - 5.0).abs() < 0.05,
        "TH flash should recover P ≈ 5 bar, got {:.4}",
        props.pressure
    );
    assert!(
        (props.density - ref_props.density).abs() < 0.5,
        "TH flash density mismatch: expected {:.2}, got {:.2}",
        ref_props.density,
        props.density
    );
}

/// TH flash with a different superheated state to verify consistency.
#[test]
fn r134a_th_flash_superheated_high_pressure() {
    let r134a = Fluid::with_units("R134A", UnitSystem::engineering()).unwrap();
    // 80 °C, 15 bar — well into superheated vapor
    let ref_props = r134a.props_tp(80.0, 15.0).unwrap();
    let props = r134a.props_th(80.0, ref_props.enthalpy).unwrap();
    assert!(
        (props.pressure - 15.0).abs() < 0.2,
        "TH flash should recover P ≈ 15 bar, got {:.4}",
        props.pressure
    );
}

/// TH flash via get() — order-independent.
#[test]
fn r134a_th_get_pressure() {
    let r134a = Fluid::with_units("R134A", UnitSystem::engineering()).unwrap();
    let ref_props = r134a.props_tp(50.0, 5.0).unwrap();
    let p = r134a.get("P", "T", 50.0, "H", ref_props.enthalpy).unwrap();
    assert!(
        (p - 5.0).abs() < 0.05,
        "get(P, T, H) should return ≈ 5 bar, got {:.4}",
        p
    );
    // Reverse key order
    let p2 = r134a.get("P", "H", ref_props.enthalpy, "T", 50.0).unwrap();
    assert!(
        (p2 - 5.0).abs() < 0.05,
        "get(P, H, T) reverse order should also work, got {:.4}",
        p2
    );
}

// ═══════════════════════════════════════════════════════════════════
//  Flash TS (Temperature-Entropy)
// ═══════════════════════════════════════════════════════════════════

/// TS flash in superheated region: verify round-trip consistency
/// by first computing S via TP, then recovering P via TS.
#[test]
fn r134a_ts_flash_superheated() {
    let r134a = Fluid::with_units("R134A", UnitSystem::engineering()).unwrap();
    let ref_props = r134a.props_tp(50.0, 5.0).unwrap();
    let props = r134a.props_ts(50.0, ref_props.entropy).unwrap();
    assert!(
        (props.pressure - 5.0).abs() < 0.05,
        "TS flash should recover P ≈ 5 bar, got {:.4}",
        props.pressure
    );
}

/// TS flash via get() — order-independent.
#[test]
fn r134a_ts_get_pressure() {
    let r134a = Fluid::with_units("R134A", UnitSystem::engineering()).unwrap();
    let ref_props = r134a.props_tp(25.0, 8.0).unwrap();
    let p = r134a.get("P", "T", 25.0, "S", ref_props.entropy).unwrap();
    assert!(
        (p - 8.0).abs() < 0.1,
        "get(P, T, S) should return ≈ 8 bar, got {:.4}",
        p
    );
    // Reverse key order
    let p2 = r134a.get("P", "S", ref_props.entropy, "T", 25.0).unwrap();
    assert!(
        (p2 - 8.0).abs() < 0.1,
        "get(P, S, T) reverse order should also work, got {:.4}",
        p2
    );
}

// ═══════════════════════════════════════════════════════════════════
//  Flash TD (Temperature-Density)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn r134a_td_flash_round_trip() {
    let r134a = Fluid::with_units("R134A", UnitSystem::engineering()).unwrap();
    let ref_props = r134a.props_tp(50.0, 5.0).unwrap();
    let props = r134a.props_td(50.0, ref_props.density).unwrap();
    assert!(
        (props.pressure - 5.0).abs() < 0.05,
        "TD flash should recover P ≈ 5 bar, got {:.4}",
        props.pressure
    );
}

#[test]
fn r134a_td_get_enthalpy() {
    let r134a = Fluid::with_units("R134A", UnitSystem::engineering()).unwrap();
    let ref_props = r134a.props_tp(25.0, 8.0).unwrap();
    let h = r134a.get("H", "T", 25.0, "D", ref_props.density).unwrap();
    assert!(
        (h - ref_props.enthalpy).abs() < 0.5,
        "get(H, T, D) expected {:.2}, got {:.2}",
        ref_props.enthalpy,
        h
    );
}

// ═══════════════════════════════════════════════════════════════════
//  Flash PD (Pressure-Density)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn r134a_pd_flash_round_trip() {
    let r134a = Fluid::with_units("R134A", UnitSystem::engineering()).unwrap();
    let ref_props = r134a.props_tp(50.0, 5.0).unwrap();
    let props = r134a.props_pd(5.0, ref_props.density).unwrap();
    assert!(
        (props.temperature - 50.0).abs() < 0.1,
        "PD flash should recover T ≈ 50 °C, got {:.4}",
        props.temperature
    );
}

// ═══════════════════════════════════════════════════════════════════
//  Flash DH (Density-Enthalpy)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn r134a_dh_flash_round_trip() {
    let r134a = Fluid::with_units("R134A", UnitSystem::engineering()).unwrap();
    let ref_props = r134a.props_tp(50.0, 5.0).unwrap();
    let props = r134a
        .props_dh(ref_props.density, ref_props.enthalpy)
        .unwrap();
    assert!(
        (props.temperature - 50.0).abs() < 0.5,
        "DH flash should recover T ≈ 50 °C, got {:.4}",
        props.temperature
    );
    assert!(
        (props.pressure - 5.0).abs() < 0.1,
        "DH flash should recover P ≈ 5 bar, got {:.4}",
        props.pressure
    );
}

// ═══════════════════════════════════════════════════════════════════
//  Flash DS (Density-Entropy)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn r134a_ds_flash_round_trip() {
    let r134a = Fluid::with_units("R134A", UnitSystem::engineering()).unwrap();
    let ref_props = r134a.props_tp(50.0, 5.0).unwrap();
    let props = r134a
        .props_ds(ref_props.density, ref_props.entropy)
        .unwrap();
    assert!(
        (props.temperature - 50.0).abs() < 0.5,
        "DS flash should recover T ≈ 50 °C, got {:.4}",
        props.temperature
    );
}

// ═══════════════════════════════════════════════════════════════════
//  Flash HS (Enthalpy-Entropy)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn r134a_hs_flash_round_trip() {
    let r134a = Fluid::with_units("R134A", UnitSystem::engineering()).unwrap();
    let ref_props = r134a.props_tp(50.0, 5.0).unwrap();
    let props = r134a
        .props_hs(ref_props.enthalpy, ref_props.entropy)
        .unwrap();
    assert!(
        (props.temperature - 50.0).abs() < 0.5,
        "HS flash should recover T ≈ 50 °C, got {:.4}",
        props.temperature
    );
    assert!(
        (props.pressure - 5.0).abs() < 0.1,
        "HS flash should recover P ≈ 5 bar, got {:.4}",
        props.pressure
    );
}

#[test]
fn r134a_hs_get_temperature() {
    let r134a = Fluid::with_units("R134A", UnitSystem::engineering()).unwrap();
    let ref_props = r134a.props_tp(30.0, 6.0).unwrap();
    let t = r134a
        .get("T", "H", ref_props.enthalpy, "S", ref_props.entropy)
        .unwrap();
    assert!(
        (t - 30.0).abs() < 0.5,
        "get(T, H, S) expected ≈ 30 °C, got {:.4}",
        t
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
