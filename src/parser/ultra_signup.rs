use {
    super::duration_deserializer,
    digital_duration_nom::duration::Duration,
    serde::Deserialize,
    std::{borrow::Cow, num::NonZeroU8},
};

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
    #[serde(deserialize_with = "duration_deserializer")]
    time: Duration,
    rank: f32,
}

#[derive(Debug, Deserialize)]
enum Status {
    Finishers = 1,
    DidNotFinish = 2,
    DidNotStart = 3,
    Disqualified = 5,
    UnofficialFinish = 6, // or perhaps that should be 4?
}

#[derive(Debug, Deserialize)]
struct StatusWithCount {
    status: Status,
    count: u16,
}

#[derive(Debug, Deserialize)]
pub struct StatusesWithPlacements(Vec<(StatusWithCount, Vec<Placement>)>);

impl StatusesWithPlacements {
    fn results(contents: &str) -> Option<Self> {
        serde_json::from_str(&contents).ok()
    }

    pub fn names_and_times(input: &str) -> Option<Vec<(Cow<str>, Duration)>> {
        Self::results(input).and_then(|swp| {
            swp.0
                .into_iter()
                .find(|(swc, _)| matches!(swc, StatusWithCount { status: Status::Finishers, .. }))
                .map(|(_, placements)| {
                    placements
                        .into_iter()
                        .map(|p| (Cow::from(format!("{} {}", p.first, p.last)), p.time))
                        .collect::<Vec<_>>()
                })
        })
    }
}
