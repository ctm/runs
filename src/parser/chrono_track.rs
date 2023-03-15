use {
    crate::prelude::*,
    digital_duration_nom::duration::Duration,
    serde::Deserialize,
    std::{
        borrow::Cow,
        num::{NonZeroU16, NonZeroU8},
    },
};

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct Placement {
    rank: NonZeroU16,
    name: String,
    bib: String,
    time: Duration,
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
