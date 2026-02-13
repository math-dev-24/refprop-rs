use refprop::{Fluid, UnitSystem};

// ── R407C (zéotrope) : bubble vs dew ────────────────────────────────

#[test]
fn r407c_bubble_pressure_at_20c() {
    let r407c = Fluid::with_units("R407C", UnitSystem::engineering()).unwrap();
    let p_bubble = r407c.get("P", "T", 20.0, "Q", 0.0).unwrap();
    assert!(
        (p_bubble - 10.38).abs() < 0.1,
        "P_bubble should be ≈ 10.38 bar, got {p_bubble:.4}"
    );
}

#[test]
fn r407c_dew_pressure_at_20c() {
    let r407c = Fluid::with_units("R407C", UnitSystem::engineering()).unwrap();
    let p_dew = r407c.get("P", "T", 20.0, "Q", 100.0).unwrap();
    assert!(
        (p_dew - 8.80).abs() < 0.1,
        "P_dew should be ≈ 8.80 bar, got {p_dew:.4}"
    );
}

#[test]
fn r407c_glide_positive() {
    // Pour un zéotrope, P_bubble > P_dew (glide)
    let r407c = Fluid::with_units("R407C", UnitSystem::engineering()).unwrap();
    let p_bubble = r407c.get("P", "T", 20.0, "Q", 0.0).unwrap();
    let p_dew = r407c.get("P", "T", 20.0, "Q", 100.0).unwrap();
    assert!(
        p_bubble > p_dew,
        "Zeotropic mixture should have P_bubble > P_dew"
    );
}

// ── R410A (quasi-azéotrope) ─────────────────────────────────────────

#[test]
fn r410a_saturation_pressure_at_0c() {
    // R410A: Psat(0 °C) ≈ 7.99 bar
    let r410a = Fluid::with_units("R410A", UnitSystem::engineering()).unwrap();
    let p = r410a.get("P", "T", 0.0, "Q", 0.0).unwrap();
    assert!(
        (p - 7.99).abs() < 0.15,
        "R410A Psat(0 °C) should be ≈ 7.99 bar, got {p:.4}"
    );
}

#[test]
fn r410a_small_glide() {
    // R410A est quasi-azéotrope : le glide doit être très faible
    let r410a = Fluid::with_units("R410A", UnitSystem::engineering()).unwrap();
    let p_bubble = r410a.get("P", "T", 0.0, "Q", 0.0).unwrap();
    let p_dew = r410a.get("P", "T", 0.0, "Q", 100.0).unwrap();
    let glide = (p_bubble - p_dew).abs();
    assert!(
        glide < 0.3,
        "R410A glide should be very small (< 0.3 bar), got {glide:.4}"
    );
}

// ── Custom mixture (R454C = R32/R1234YF) ────────────────────────────

#[test]
fn custom_mixture_r454c() {
    let r454c = Fluid::mixture_with_units(
        &[("R32", 0.215), ("R1234YF", 0.785)],
        UnitSystem::engineering(),
    )
    .unwrap();
    // Psat de R454C à 0 °C côté bulle, valeur attendue ~4.5 bar
    let p = r454c.get("P", "T", 0.0, "Q", 0.0).unwrap();
    assert!(p > 3.0 && p < 7.0, "R454C Psat(0 °C) should be reasonable, got {p:.4}");
}
