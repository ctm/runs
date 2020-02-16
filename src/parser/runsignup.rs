use {
    digital_duration_nom::duration::Duration,
    serde::{Deserialize, Deserializer},
    std::borrow::Cow,
};

#[derive(Deserialize, Debug)]
pub struct Placement {
    place: String,
    // bib: String,
    name: String,
    // city: String,
    #[serde(deserialize_with = "duration_deserializer")]
    clock_time: Duration,
    // ...
}

// TODO: merge with chrono_track and athlinks
impl Placement {
    pub fn results(contents: &str) -> Option<Vec<Self>> {
        contents
            .trim()
            .split('\n')
            .map(|s| serde_json::from_str::<Vec<_>>(s))
            .collect::<Result<Vec<_>, _>>()
            .ok()
            .map(|v| v.into_iter().flatten().collect())
    }

    pub fn names_and_times(input: &str) -> Option<Vec<(Cow<str>, Duration)>> {
        Self::results(input).map(|results| {
            results
                .into_iter()
                .map(|placement| (Cow::from(placement.name), placement.clock_time))
                .collect()
        })
    }
}

// TODO: this should be put somewhere else
fn duration_deserializer<'de, D: Deserializer<'de>>(d: D) -> Result<Duration, D::Error> {
    let s: String = String::deserialize(d)?;
    Ok(s.parse::<Duration>().unwrap()) // TODO: map_err instead Ok(...unwrap())
}
