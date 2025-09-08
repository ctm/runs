// NOTE: sex is not currently implemented

use {
    crate::prelude::*, digital_duration_nom::duration::Duration, serde::Deserialize,
    std::num::NonZeroU16,
};

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct Placement {
    rank: NonZeroU16,
    name: String,
    bib: String,
    time: Duration,
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

    pub fn names_and_times(input: &str) -> OptionalResults<'_> {
        Self::results(input).map(|results| {
            results
                .into_iter()
                .map(|placement| (Cow::from(placement.name), placement.time, None))
                .collect()
        })
    }
}
