#![feature(proc_macro_hygiene)]

#[macro_use]
extern crate nom;

pub mod names;
pub mod parsers;

// Perhaps the code here should be somewhere else
use crate::parsers::NameAndTime;
use sports_metrics::duration::Duration;
use std::collections::HashMap;

pub fn merged_results<'a>(
    results: &'a [Vec<&dyn NameAndTime>],
) -> HashMap<&'a str, Vec<Option<Duration>>> {
    let mut h: HashMap<&str, Vec<Option<Duration>>> = HashMap::new();
    let n = results.len();

    for (i, results) in results.iter().enumerate() {
        for result in results {
            let name = names::canonical(&result.name());
            let durations = h.entry(&name).or_insert_with(|| {
                let mut v = Vec::with_capacity(n);
                for _ in 0..n {
                    v.push(None);
                }
                v
            });
            durations[i] = Some(result.time());
        }
    }
    h
}
