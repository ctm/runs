use crate::parsers::NameAndTime;
use nom::{
    bytes::complete::tag, character::complete::digit1, combinator::map, sequence::preceded, IResult,
};
use serde::Deserialize;
use sports_metrics::duration::Duration;
use std::str::FromStr;

// This is what we're given.  It's close to what we want, but it has some
// fields we don't understand and also has inconsistent use of snake-case
// and time is a string.
#[derive(Deserialize, Debug)]
struct PlacementJson {
    age: u8,             // 30
    age_rank: u16,       // 0
    agegroup: String,    // "30-39"
    bib: String,         // "1"
    city: String,        // "Tucson"
    drilldown: u8,       // 0
    firstname: String,   // "Craig"
    formattime: String,  // "4:33:25"
    gender: String,      // "M"
    gender_place: u16,   // 1
    lastname: String,    // "Curley
    participant_id: u32, // 809059
    photo_count: u16,    // 0
    place: u16,          // 1
    prior_count: u16,    // 0
    race_count: u16,     // 0
    runner_rank: f64,    // 97.44
    state: String,       // "AZ"
    status: u8,          // 1 appears to be finished, 2 DNF, 3 DNS?
    time: String,        // "16405000" (that's the time in milliseconds as a string)
}

pub struct Placement {
    pub age: u8,             // 30
    pub age_rank: u16,       // 0
    pub age_group: String,   // "30-39"
    pub bib: String,         // "1"
    pub city: String,        // "Tucson"
    pub first_name: String,  // "Craig"
    pub gender: String,      // "M"
    pub gender_place: u16,   // 1
    pub last_name: String,   // "Curley
    pub participant_id: u32, // 809059
    pub place: u16,          // 1
    pub runner_rank: f64,    // 97.44
    pub state: String,       // "AZ"
    pub time: Duration,
    pub name: String, // This one we create ourselves
}

impl Placement {
    #[allow(dead_code)]
    pub fn body_from(uri: &str) -> Option<String> {
        match jsonable_uri(uri) {
            Err(_) => None,
            Ok((_, uri)) => super::body_from(&uri),
        }
    }

    pub fn results(contents: &str) -> Option<Vec<Self>> {
        let fiftyk_json: Result<Vec<PlacementJson>, serde_json::error::Error> =
            serde_json::from_str(&contents);
        match fiftyk_json {
            Ok(fiftyk_json) => Some(fiftyk_json.into_iter().map(Self::from).collect()),
            Err(_) => None,
        }
    }

    pub fn names_and_times<'a>(results: &'a [Self]) -> Vec<&'a dyn NameAndTime> {
        results.iter().map(|r| r as &dyn NameAndTime).collect()
    }
}

impl From<PlacementJson> for Placement {
    fn from(json: PlacementJson) -> Self {
        let age = json.age;
        let age_rank = json.age_rank;
        let age_group = json.agegroup;
        let bib = json.bib;
        let city = json.city;
        let first_name = json.firstname;
        let gender = json.gender;
        let gender_place = json.gender_place;
        let last_name = json.lastname;
        let participant_id = json.participant_id;
        let place = json.place;
        let runner_rank = json.runner_rank;
        let state = json.state;

        let milliseconds = u64::from_str(&json.time).unwrap();
        let secs = milliseconds / 1000;
        let nanos = (milliseconds % 1000) as u32 * 1_000_000;
        let time = Duration::new(secs, nanos);

        let name = format!("{} {}", first_name, last_name);

        Self {
            age,
            age_rank,
            age_group,
            bib,
            city,
            first_name,
            gender,
            gender_place,
            last_name,
            participant_id,
            place,
            runner_rank,
            state,
            time,
            name,
        }
    }
}

impl NameAndTime for Placement {
    fn name(&self) -> &str {
        &self.name
    }

    fn time(&self) -> Duration {
        self.time
    }
}

#[allow(dead_code)]
fn jsonable_uri(input: &str) -> IResult<&str, String> {
    map(
        preceded(
            tag("https://ultrasignup.com/results_event.aspx?did="),
            digit1,
        ),
        |did| {
            format!("https://ultrasignup.com/service/events.svc/results/{}/json?_search=false&rows=1500&page=1&sidx=status+asc%2C+&sord=asc", did)
        },
    )(input)
}
