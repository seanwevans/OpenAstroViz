use openastroviz_core::Sgp4Propagator;

/// Demonstrate Vallado SGP4 propagation using the core crate.
///
/// Run with:
/// `cargo run -p openastroviz-core --example propagate`
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let line1 = "1 25544U 98067A   20194.88612269 -.00002218  00000-0 -31515-4 0  9992";
    let line2 = "2 25544  51.6461 221.2784 0001413  89.1723 280.4612 15.49507896236008";
    let propagator = Sgp4Propagator::from_tle(Some("ISS (ZARYA)".to_owned()), line1, line2)?;

    let offsets = [0.0f64, 10.0, 60.0, 1440.0];
    for minutes in offsets {
        let state = propagator.propagate_minutes(minutes)?;
        println!(
            "t+{:6.1} min | r = [{:9.3}, {:9.3}, {:9.3}] km | v = [{:7.4}, {:7.4}, {:7.4}] km/s",
            minutes,
            state.position_km[0],
            state.position_km[1],
            state.position_km[2],
            state.velocity_km_s[0],
            state.velocity_km_s[1],
            state.velocity_km_s[2]
        );
    }

    Ok(())
}
