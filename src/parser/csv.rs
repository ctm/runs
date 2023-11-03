// NOTE: Even though this is called CSV, it's currently a one-off for the CSV
//       that I got when I scraped the results out of the official PDF
//       of results of the Forever Young 6 Miler 2023.

use {
    crate::prelude::*, csv::ReaderBuilder, digital_duration_nom::duration::Duration,
    serde::Deserialize, std::num::NonZeroU8,
};

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct Placement {
    // Last name,First Name,G,Age,Event Bib #,,Time,Division,Place
    last_name: String,
    first_name: String,
    gender: String,
    age: NonZeroU8,
    _event: String, // "6" for 6 miler
    bib: NonZeroU8,
    time: Duration,
    // _division is a String, but it could be an Option<> of an enum whose
    // variants would be Master, Open, Senior, GrandMaster, Junior, and Legend
    _division: String,
    _place: String, // "Overall Male", "Overall Female", "1st", "2nd", etc.
}

impl Placement {
    pub fn results(contents: &str) -> Option<Vec<Self>> {
        ReaderBuilder::new()
            .has_headers(false)
            .from_reader(contents.as_bytes())
            .deserialize()
            .skip(2)
            .collect::<Result<Vec<_>, _>>()
            .ok()
    }

    pub fn names_and_times(input: &str) -> OptionalResults {
        Self::results(input).map(|results| {
            results
                .into_iter()
                .map(|placement| {
                    let morf = placement.morf();
                    (
                        Cow::from(format!("{} {}", placement.first_name, placement.last_name)),
                        placement.time,
                        morf,
                    )
                })
                .collect()
        })
    }
}

impl Gender for Placement {
    fn gender(&self) -> &str {
        &self.gender
    }
}
