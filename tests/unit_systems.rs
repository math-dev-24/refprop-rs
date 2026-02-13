use refprop::{Fluid, UnitSystem};

// ═══════════════════════════════════════════════════════════════════
//  Cohérence entre systèmes d'unités
// ═══════════════════════════════════════════════════════════════════

#[test]
fn engineering_vs_refprop_temperature() {
    // Même fluide, même point : T en K vs T en °C
    let r134a_eng = Fluid::with_units("R134A", UnitSystem::engineering()).unwrap();
    let r134a_rp = Fluid::with_units("R134A", UnitSystem::refprop()).unwrap();

    let t_eng = r134a_eng.get("T", "P", 3.0, "Q", 0.0).unwrap(); // °C
    let t_rp = r134a_rp.get("T", "P", 300.0, "Q", 0.0).unwrap(); // K (3 bar = 300 kPa)

    let diff = (t_rp - 273.15) - t_eng;
    assert!(
        diff.abs() < 0.1,
        "T(eng) = {t_eng:.4} °C, T(rp) = {t_rp:.4} K → diff = {diff:.4}"
    );
}

#[test]
fn engineering_vs_refprop_pressure() {
    // Psat(0 °C) en bar vs kPa
    let r134a_eng = Fluid::with_units("R134A", UnitSystem::engineering()).unwrap();
    let r134a_rp = Fluid::with_units("R134A", UnitSystem::refprop()).unwrap();

    let p_eng = r134a_eng.get("P", "T", 0.0, "Q", 0.0).unwrap(); // bar
    let p_rp = r134a_rp.get("P", "T", 273.15, "Q", 0.0).unwrap(); // kPa

    let diff = (p_rp / 100.0) - p_eng;
    assert!(
        diff.abs() < 0.01,
        "P(eng) = {p_eng:.4} bar, P(rp) = {p_rp:.4} kPa → diff = {diff:.6}"
    );
}

#[test]
fn engineering_vs_si_density() {
    // La densité en kg/m³ doit être la même en engineering et SI
    let r134a_eng = Fluid::with_units("R134A", UnitSystem::engineering()).unwrap();
    let r134a_si = Fluid::with_units("R134A", UnitSystem::si()).unwrap();

    let d_eng = r134a_eng.get("D", "T", 0.0, "Q", 100.0).unwrap(); // kg/m³
    let d_si = r134a_si.get("D", "T", 273.15, "Q", 100.0).unwrap(); // kg/m³

    let diff = (d_eng - d_si).abs();
    assert!(
        diff < 0.1,
        "D(eng) = {d_eng:.4}, D(si) = {d_si:.4}, diff = {diff:.6}"
    );
}

#[test]
fn si_pressure_in_pascal() {
    // En SI strict, la pression est en Pa
    let r134a_si = Fluid::with_units("R134A", UnitSystem::si()).unwrap();
    let r134a_eng = Fluid::with_units("R134A", UnitSystem::engineering()).unwrap();

    let p_si = r134a_si.get("P", "T", 273.15, "Q", 0.0).unwrap(); // Pa
    let p_eng = r134a_eng.get("P", "T", 0.0, "Q", 0.0).unwrap(); // bar

    let diff = (p_si / 100_000.0) - p_eng;
    assert!(
        diff.abs() < 0.01,
        "P(si) = {p_si:.0} Pa, P(eng) = {p_eng:.4} bar → diff = {diff:.6}"
    );
}
