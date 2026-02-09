use refprop::Fluid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fluid_name = "CO2";
    let fluid = Fluid::new(fluid_name)?;

    let critical_point = fluid.critical_point()?;

    let temp_crit = critical_point.temperature - 273.15;
    let press_crit = critical_point.pressure / 100.0;
    
    println!("Critical point: {temp_crit} °C, {press_crit} bar");


    Ok(())
}