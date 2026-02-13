use refprop::Fluid;

// ═══════════════════════════════════════════════════════════════════
//  FluidInfo — constantes physiques
// ═══════════════════════════════════════════════════════════════════

#[test]
fn r134a_molar_mass() {
    // R134A: M ≈ 102.032 g/mol
    let r134a = Fluid::new("R134A").unwrap();
    let info = r134a.info().unwrap();
    assert!(
        (info.molar_mass - 102.032).abs() < 0.1,
        "R134A M expected ≈ 102.03 g/mol, got {:.4}",
        info.molar_mass
    );
}

#[test]
fn co2_molar_mass() {
    // CO2: M ≈ 44.01 g/mol
    let co2 = Fluid::new("CO2").unwrap();
    let info = co2.info().unwrap();
    assert!(
        (info.molar_mass - 44.01).abs() < 0.1,
        "CO2 M expected ≈ 44.01 g/mol, got {:.4}",
        info.molar_mass
    );
}

#[test]
fn water_molar_mass() {
    // H2O: M ≈ 18.015 g/mol
    let water = Fluid::new("WATER").unwrap();
    let info = water.info().unwrap();
    assert!(
        (info.molar_mass - 18.015).abs() < 0.1,
        "Water M expected ≈ 18.015 g/mol, got {:.4}",
        info.molar_mass
    );
}

#[test]
fn r134a_triple_point() {
    // R134A: T_triple ≈ 169.85 K (-103.3 °C)
    let r134a = Fluid::new("R134A").unwrap();
    let info = r134a.info().unwrap();
    assert!(
        (info.triple_point_temp - 169.85).abs() < 1.0,
        "R134A T_triple expected ≈ 169.85 K, got {:.4}",
        info.triple_point_temp
    );
}

#[test]
fn r134a_normal_boiling_point() {
    // R134A: T_nbp ≈ 247.08 K (-26.07 °C)
    let r134a = Fluid::new("R134A").unwrap();
    let info = r134a.info().unwrap();
    assert!(
        (info.normal_boiling_point - 247.08).abs() < 1.0,
        "R134A T_nbp expected ≈ 247.08 K, got {:.4}",
        info.normal_boiling_point
    );
}

#[test]
fn fluid_info_gas_constant() {
    // La constante R est universelle ≈ 8.314 J/(mol·K)
    let r134a = Fluid::new("R134A").unwrap();
    let info = r134a.info().unwrap();
    assert!(
        (info.gas_constant - 8.314).abs() < 0.01,
        "R expected ≈ 8.314 J/(mol·K), got {:.6}",
        info.gas_constant
    );
}
