pub mod ath_links;
pub mod ccr_timing;
pub mod chrono_track;
pub mod run_fit;
pub mod taos;
pub mod ultra_signup;
pub mod web_scorer;

use nom::{
    bytes::complete::{tag, take_until},
    error::ParseError,
    sequence::terminated,
    Compare, FindSubstring, IResult, InputLength, InputTake,
};

#[allow(dead_code)]
fn body_from(uri: &str) -> Option<String> {
    match reqwest::get(uri) {
        Err(_) => None,
        Ok(mut response) => match response.text() {
            Err(_) => None,
            Ok(body) => Some(body),
        },
    }
}

pub fn take_until_and_consume<T, Input, Error: ParseError<Input>>(
    tag_to_match: T,
) -> impl Fn(Input) -> IResult<Input, Input, Error>
where
    Input: InputTake + FindSubstring<T> + Compare<T>,
    T: InputLength + Clone,
{
    let cloned_tag_to_match = tag_to_match.clone();

    terminated(take_until(tag_to_match), tag(cloned_tag_to_match))
}
