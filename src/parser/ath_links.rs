use {
    super::duration_deserializer,
    digital_duration_nom::duration::Duration,
    serde::Deserialize,
    std::{
        borrow::Cow,
        num::{NonZeroU16, NonZeroU8},
    },
};

#[derive(Deserialize, Debug)]
pub struct Placement {
    name: String,
    sex: String,
    age: Option<NonZeroU8>,
    bib: String,
    hometown: String,
    rank: NonZeroU16,
    gender_rank: NonZeroU16,
    division_rank: NonZeroU16,
    #[serde(deserialize_with = "duration_deserializer")]
    pace: Duration,
    #[serde(deserialize_with = "duration_deserializer")]
    time: Duration,
}

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
