use openastroviz_core::{PropagationError, Sgp4Propagator};

const LINE1: &str = "1 00005U 58002B   00179.78495062  .00000023  00000-0  28098-4 0  4753";
const LINE2: &str = "2 00005  34.2682 348.7242 1859667 331.7664  19.3264 10.82419157413667";

#[test]
fn rejects_invalid_tle() {
    let err = Sgp4Propagator::from_tle(Some("bad".to_string()), "invalid", LINE2)
        .expect_err("invalid TLE should fail");
    assert!(matches!(err, PropagationError::Tle(_)));
}

#[test]
fn rejects_non_finite_minutes() {
    let propagator =
        Sgp4Propagator::from_tle(Some("Vanguard 1".to_string()), LINE1, LINE2).unwrap();

    let err = propagator
        .propagate_minutes(f64::NAN)
        .expect_err("non-finite propagation input should fail");
    assert!(matches!(err, PropagationError::Propagation(_)));
}

#[test]
fn exposes_epoch_from_tle() {
    let propagator =
        Sgp4Propagator::from_tle(Some("Vanguard 1".to_string()), LINE1, LINE2).unwrap();
    let epoch = propagator.epoch();

    let expected =
        chrono::NaiveDateTime::parse_from_str("2000-06-27T18:50:19.733571", "%Y-%m-%dT%H:%M:%S%.f")
            .unwrap();
    let delta_ms = (epoch - expected).num_microseconds().unwrap().abs();
    assert!(delta_ms <= 10);
}
