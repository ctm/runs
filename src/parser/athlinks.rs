use {
    crate::prelude::*,
    digital_duration_nom::duration::Duration,
    serde::Deserialize,
    std::num::{NonZeroU8, NonZeroU16},
};

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct Placement {
    name: String,
    sex: String,
    age: Option<NonZeroU8>,
    bib: String,
    hometown: String,
    rank: NonZeroU16,
    gender_rank: Option<NonZeroU16>,
    division_rank: NonZeroU16,
    pace: Duration,
    time: Duration,
}

impl Placement {
    pub fn results(contents: &str) -> Option<Vec<Self>> {
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
                .map(|placement| {
                    let morf = placement.morf();
                    (Cow::from(placement.name), placement.time, morf)
                })
                .collect()
        })
    }
}

impl Gender for Placement {
    fn gender(&self) -> &str {
        &self.sex[..]
    }
}
