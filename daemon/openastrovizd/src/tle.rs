use openastroviz_core::{Sgp4Propagator, StateVector};

const CELESTRAK_ACTIVE_TLE_URL: &str = "https://celestrak.org/NORAD/elements/gp.php?GROUP=active&FORMAT=tle";

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OrbitalRecord {
    pub name: String,
    pub line1: String,
    pub line2: String,
    pub propagator: Sgp4Propagator,
    pub epoch_state: StateVector,
}

/// Download the daily public TLE catalog from CelesTrak.
pub async fn fetch_tle_catalog(client: &reqwest::Client) -> Result<String, reqwest::Error> {
    client
        .get(CELESTRAK_ACTIVE_TLE_URL)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await
}

/// Parse a CelesTrak/Space-Track-style 3-line TLE catalog.
pub fn parse_tle_catalog(body: &str) -> Result<Vec<OrbitalRecord>, String> {
    let mut lines = body.lines().map(str::trim).filter(|line| !line.is_empty());
    let mut records = Vec::new();

    while let Some(name) = lines.next() {
        let line1 = lines
            .next()
            .ok_or_else(|| format!("missing line 1 for object `{name}`"))?;
        let line2 = lines
            .next()
            .ok_or_else(|| format!("missing line 2 for object `{name}`"))?;

        if !line1.starts_with('1') {
            return Err(format!("invalid TLE line 1 for `{name}`"));
        }
        if !line2.starts_with('2') {
            return Err(format!("invalid TLE line 2 for `{name}`"));
        }

        let propagator = Sgp4Propagator::from_tle(Some(name.to_owned()), line1, line2)
            .map_err(|e| format!("failed to parse TLE for `{name}`: {e}"))?;
        let epoch_state = propagator
            .propagate_minutes(0.0)
            .map_err(|e| format!("failed to derive epoch state for `{name}`: {e}"))?;

        records.push(OrbitalRecord {
            name: name.to_owned(),
            line1: line1.to_owned(),
            line2: line2.to_owned(),
            propagator,
            epoch_state,
        });
    }

    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_three_line_catalog() {
        let src = "ISS (ZARYA)\n1 25544U 98067A   20194.88612269 -.00002218  00000-0 -31515-4 0  9992\n2 25544  51.6461 221.2784 0001413  89.1723 280.4612 15.49507896236008\n";
        let records = parse_tle_catalog(src).expect("catalog should parse");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].name, "ISS (ZARYA)");
    }

    #[test]
    fn rejects_incomplete_triplet() {
        let src = "TEST\n1 00005U 58002B   00179.78495062  .00000023  00000-0  28098-4 0  4753\n";
        let err = parse_tle_catalog(src).expect_err("must fail");
        assert!(err.contains("missing line 2"));
    }
}
