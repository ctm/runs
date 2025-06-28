use {
    crate::prelude::*, digital_duration_nom::duration::Duration, serde::Deserialize,
    std::str::FromStr,
};

pub enum Status {
    Finished = 1,
    DidNotFinish = 2,
    DidNotStart = 3,
    Disqualified = 5,
}

// This is what we're given.  It's close to what we want, but it has some
// fields we don't understand and also has inconsistent use of snake-case
// and time is a string.
#[allow(dead_code)]
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
    status: u8,          // 1 Finished, 2 DNF, 3 DNS
    time: String,        // "16405000" (that's the time in milliseconds as a string)
}

#[allow(dead_code)]
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
    pub status: Status,
    pub time: Duration,
    pub name: String, // This one we create ourselves
}

impl Placement {
    pub fn results(contents: &str) -> Option<Vec<Self>> {
        let json: Result<Vec<PlacementJson>, serde_json::error::Error> =
            serde_json::from_str(contents);
        match json {
            Ok(json) => Some(json.into_iter().map(Self::from).collect()),
            Err(_) => None,
        }
    }

    pub fn names_and_times(input: &str) -> OptionalResults {
        use Status::*;

        Self::results(input).map(|results| {
            results
                .into_iter()
                .filter_map(|placement| match placement.status {
                    Finished => {
                        let morf = placement.morf();
                        Some((Cow::from(placement.name), placement.time, morf))
                    }
                    _ => None,
                })
                .collect()
        })
    }
}

impl Gender for Placement {
    fn gender(&self) -> &str {
        &self.gender[..]
    }
}

impl From<PlacementJson> for Placement {
    fn from(json: PlacementJson) -> Self {
        use Status::*;

        const FINISHED: u8 = Finished as u8;
        const DID_NOT_FINISH: u8 = DidNotFinish as u8;
        const DID_NOT_START: u8 = DidNotStart as u8;
        const DISQUALIFIED: u8 = Disqualified as u8;

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
        let status = match json.status {
            FINISHED => Finished,
            DID_NOT_FINISH => DidNotFinish,
            DID_NOT_START => DidNotStart,
            DISQUALIFIED => Disqualified,
            other => panic!("Unknown status {other}"),
        };

        let milliseconds = u64::from_str(&json.time).unwrap();
        let secs = milliseconds / 1000;
        let nanos = (milliseconds % 1000) as u32 * 1_000_000;
        let time = Duration::new(secs, nanos);

        let name = format!("{first_name} {last_name}");

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
            status,
            time,
            name,
        }
    }
}
