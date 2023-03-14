// NOTE: sex is not currently implemented

use {
    super::duration_deserializer,
    crate::prelude::*,
    digital_duration_nom::duration::Duration,
    serde::Deserialize,
    std::{borrow::Cow, num::NonZeroU16},
};

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct Placement {
    rank: NonZeroU16,
    name: String,
    bib: String,
    #[serde(deserialize_with = "duration_deserializer")]
    time: Duration,
    #[serde(deserialize_with = "duration_deserializer")]
    pace: Duration,
}

impl Placement {
    fn results(contents: &str) -> Option<Vec<Self>> {
        contents
            .trim()
            .split('\n')
            .map(serde_json::from_str::<Vec<_>>)
            .collect::<Result<Vec<_>, _>>()
            .ok()
            .map(|v| v.into_iter().flatten().collect())
    }

    pub fn names_and_times(input: &str) -> OptionalResults {
        Self::results(input).map(|results| {
            results
                .into_iter()
                .map(|placement| (Cow::from(placement.name), placement.time, None))
                .collect()
        })
    }
}
