use approx::assert_relative_eq;
use openastroviz_core::Sgp4Propagator;

const LINE1: &str = "1 00005U 58002B   00179.78495062  .00000023  00000-0  28098-4 0  4753";
const LINE2: &str = "2 00005  34.2682 348.7242 1859667 331.7664  19.3264 10.82419157413667";

fn assert_state_close(
    actual: &openastroviz_core::StateVector,
    expected_r: [f64; 3],
    expected_v: [f64; 3],
) {
    for i in 0..3 {
        assert_relative_eq!(actual.position_km[i], expected_r[i], epsilon = 2e-2);
        assert_relative_eq!(actual.velocity_km_s[i], expected_v[i], epsilon = 5e-5);
    }
}

#[test]
fn matches_vallado_reference_minutes() {
    let propagator = Sgp4Propagator::from_tle(Some("Vanguard 1".to_string()), LINE1, LINE2)
        .expect("failed to build propagator");

    let cases = [
        (
            0.0,
            [7022.46529266, -1400.08296755, 0.03995155],
            [1.893841015, 6.405893759, 4.53480725],
        ),
        (
            360.0,
            [-7154.03120202, -3783.17682504, -3536.19412294],
            [4.741887409, -4.151817765, -2.093935425],
        ),
        (
            1440.0,
            [-938.55923943, -6268.18748831, -4294.02924751],
            [7.536105209, -0.427127707, 0.98987808],
        ),
    ];

    for (minutes, expected_r, expected_v) in cases {
        let state = propagator
            .propagate_minutes(minutes)
            .expect("propagation should succeed");
        assert_state_close(&state, expected_r, expected_v);
    }
}

#[test]
fn matches_vallado_reference_datetime() {
    let propagator =
        Sgp4Propagator::from_tle(None, LINE1, LINE2).expect("failed to build propagator");
    let target = chrono::NaiveDateTime::parse_from_str(
        "2000-06-28T12:50:19.733571Z",
        "%Y-%m-%dT%H:%M:%S%.fZ",
    )
    .expect("invalid datetime");
    let state = propagator
        .propagate_datetime(target)
        .expect("propagation should succeed");

    let expected_r = [5568.53901181, 4492.06992591, 3863.87641983];
    let expected_v = [-4.209106476, 5.159719888, 2.74485298];
    assert_state_close(&state, expected_r, expected_v);
}
