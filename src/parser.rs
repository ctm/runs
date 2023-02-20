pub mod ancient_ultra_signup;
pub mod ath_links;
pub mod ccr_timing;
pub mod chrono_track;
pub mod run_fit;
pub mod runsignup;
pub mod taos;
pub mod ultra_signup;
pub mod web_scorer;

use {
    digital_duration_nom::duration::Duration,
    nom::{
        bytes::complete::{tag, take_until},
        error::ParseError,
        sequence::terminated,
        Compare, FindSubstring, IResult, InputLength, InputTake,
    },
    serde::{Deserialize, Deserializer},
};

#[allow(dead_code)]
fn body_from(uri: &str) -> Option<String> {
    match reqwest::blocking::get(uri) {
        Err(_) => None,
        Ok(response) => match response.text() {
            Err(_) => None,
            Ok(body) => Some(body),
        },
    }
}

pub fn take_until_and_consume<T, Input, Error: ParseError<Input>>(
    tag_to_match: T,
) -> impl FnMut(Input) -> IResult<Input, Input, Error>
where
    Input: InputTake + FindSubstring<T> + Compare<T>,
    T: InputLength + Clone,
{
    let cloned_tag_to_match = tag_to_match.clone();

    terminated(take_until(tag_to_match), tag(cloned_tag_to_match))
}

fn duration_deserializer<'de, D: Deserializer<'de>>(d: D) -> Result<Duration, D::Error> {
    let s: String = String::deserialize(d)?;
    s.parse::<Duration>().map_err(serde::de::Error::custom)
}
