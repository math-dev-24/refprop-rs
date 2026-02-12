use refprop::Fluid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // REFPROP_PATH is read from the .env file automatically.
    // You can also set it as an environment variable:
    //   set REFPROP_PATH=C:\Program Files (x86)\REFPROP

    println!("=== Loading R134A ===\n");
    let r134a = Fluid::new("R134a")?;

    // ── Fluid information ───────────────────────────────────────────
    let info = r134a.info()?;
    println!("Fluid info:\n{info}\n");

    // ── Critical point ──────────────────────────────────────────────
    let crit = r134a.critical_point()?;
    println!("Critical point:\n{crit}\n");

    // ── TP flash: T = 25 °C (298.15 K), P = 5 bar (500 kPa) ───────
    let props = r134a.props_tp(298.15, 500.0)?;
    println!("TP flash (T=298.15 K, P=500 kPa):\n{props}\n");

    // ── Transport properties at the same state point ────────────────
    let trn = r134a.transport(298.15, props.density)?;
    println!("Transport:\n{trn}\n");

    // ── Saturation at P = 5 bar ─────────────────────────────────────
    let sat = r134a.saturation_p(500.0)?;
    println!("Saturation at P=500 kPa:\n{sat}\n");

    // ── Saturation at T = 0 °C ──────────────────────────────────────
    let sat_t = r134a.saturation_t(273.15)?;
    println!("Saturation at T=273.15 K (0 °C):\n{sat_t}\n");

    // ── PH flash: P = 500 kPa, H from the TP flash above ───────────
    let ph = r134a.props_ph(500.0, props.enthalpy)?;
    println!("PH flash (P=500, H={:.2}):\n{ph}\n", props.enthalpy);

    // ── TQ flash: saturated vapor at 0 °C ─────────────────────────
    let tq = r134a.props_tq(273.15, 100.0)?;
    println!("TQ flash (T=273.15 K, Q=100%):\n{tq}\n");

    // ── Generic get() – CoolProp style ────────────────────────────
    let d = r134a.get("D", "T", 273.15, "Q", 100.0)?;
    println!("get(D, T=273.15, Q=100) = {d:.6} mol/L");

    let p_sat = r134a.get("P", "T", 273.15, "Q", 0.0)?;
    println!("get(P, T=273.15, Q=0) = {p_sat:.4} kPa");

    let p_sat = Fluid::new("R407C")?
    .get("P", "T", 273.15, "Q", 0.0)?;

    println!("get(P, T=273.15, Q=0) = {p_sat:.4} kPa");
    Ok(())
}
