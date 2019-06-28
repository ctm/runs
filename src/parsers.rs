pub mod ccr_timing;
pub mod ultra_signup;
pub mod web_scorer;

use sports_metrics::duration::Duration;

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

pub trait NameAndTime {
    fn name(&self) -> &str;
    fn time(&self) -> Duration;
}
