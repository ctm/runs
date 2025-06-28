pub mod ancient_ultra_signup;
pub mod athlinks;
pub mod ccr_timing;
pub mod chrono_track;
pub mod csv;
mod helpers;
pub mod its_your_race;
pub mod race_result_mhtml;
pub mod race_roster;
pub mod run_fit;
pub mod runsignup;
pub mod runsignup_20240506_mhtml;
pub mod runsignup_mhtml;
pub mod taos;
pub mod ultra_signup;
pub mod ultra_signup_mhtml;
pub mod web_scorer;

use nom::{
    Compare, FindSubstring, Input, Parser,
    bytes::complete::{tag, take_until},
    error::ParseError,
    sequence::terminated,
};

#[allow(dead_code)]
fn body_from(uri: &str) -> Option<String> {
    match reqwest::blocking::get(uri) {
        Err(_) => None,
        Ok(response) => response.text().ok(),
    }
}

pub fn take_until_and_consume<T, I, E: ParseError<I>>(
    tag_to_match: T,
) -> impl Parser<I, Output = I, Error = E>
where
    I: Input + FindSubstring<T> + Compare<T>,
    T: Input + Copy,
{
    terminated(take_until(tag_to_match), tag(tag_to_match))
}
