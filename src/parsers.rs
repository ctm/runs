pub mod ccr_timing;
pub mod run_fit;
pub mod ultra_signup;
pub mod web_scorer;

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
