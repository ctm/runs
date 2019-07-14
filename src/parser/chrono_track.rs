use {
    digital_duration_nom::duration::Duration,
    serde::{Deserialize, Deserializer},
    std::{
        borrow::Cow,
        num::{NonZeroU16, NonZeroU8},
    },
};

#[derive(Deserialize, Debug)]
pub struct Placement {
    rank: NonZeroU16,
    name: String,
    bib: String,
    #[serde(deserialize_with = "duration_deserializer")]
    time: Duration,
    #[serde(deserialize_with = "duration_deserializer")]
    pace: Duration,
    hometown: String,
    age: Option<NonZeroU8>,
    sex: String,
    division: String,
    division_rank: NonZeroU16,
}

// TODO: results and names_and_times are *exactly* the same as from ath_links,
//       I believe.  Obviously, they should be merged.
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
                .map(|placement| (Cow::from(placement.name), placement.time))
                .collect()
        })
    }
}

// TODO: this should be put somewhere else
fn duration_deserializer<'de, D: Deserializer<'de>>(d: D) -> Result<Duration, D::Error> {
    let s: String = String::deserialize(d)?;
    Ok(s.parse::<Duration>().unwrap()) // TODO: map_err instead Ok(...unwrap())
}
