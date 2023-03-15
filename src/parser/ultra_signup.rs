use {
    crate::prelude::*,
    digital_duration_nom::duration::Duration,
    serde::Deserialize,
    std::{borrow::Cow, num::NonZeroU8},
};

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Placement {
    place: u16,
    first: String,
    last: String,
    city: Option<String>,
    state: Option<String>,
    age: NonZeroU8,
    gender: String,
    gp: u16,
    time: Duration,
    rank: f32,
}

impl Gender for Placement {
    fn gender(&self) -> &str {
        &self.gender[..]
    }
}

#[derive(Debug, Deserialize)]
enum Status {
    Finishers = 1,
    DidNotFinish = 2,
    DidNotStart = 3,
    Disqualified = 5,
    UnofficialFinish = 6, // or perhaps that should be 4?
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct StatusWithCount {
    status: Status,
    count: u16,
}

#[derive(Debug, Deserialize)]
pub struct StatusesWithPlacements(Vec<(StatusWithCount, Vec<Placement>)>);

impl StatusesWithPlacements {
    fn results(contents: &str) -> Option<Self> {
        serde_json::from_str(contents).ok()
    }

    pub fn names_and_times(input: &str) -> OptionalResults {
        Self::results(input).and_then(|swp| {
            swp.0
                .into_iter()
                .find(|(swc, _)| {
                    matches!(
                        swc,
                        StatusWithCount {
                            status: Status::Finishers,
                            ..
                        }
                    )
                })
                .map(|(_, placements)| {
                    placements
                        .into_iter()
                        .map(|p| {
                            (
                                Cow::from(format!("{} {}", p.first, p.last)),
                                p.time,
                                p.morf(),
                            )
                        })
                        .collect::<Vec<_>>()
                })
        })
    }
}
