// Using ARIA_FIELDS via phf is a lose.  I should just have used a
// match statement, because then the compiler is free to construct a
// perfect hash or use a linear search or whatever it thinks is
// fastest.

use {
    crate::{hashes::ARIA_FIELDS, prelude::*},
    digital_duration_nom::duration::Duration,
    scraper::{CaseSensitivity::AsciiCaseInsensitive, ElementRef, Html, Selector},
    std::{collections::HashMap, fmt::Debug, mem, num::NonZeroU8, str::FromStr},
};

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct Placement {
    place: u16,
    first: String,
    last: String,
    city: Option<String>,
    state: Option<String>,
    age: NonZeroU8,
    gender: String,
    gp: u16,
    time: Duration,
    rank: f32,
}

impl Gender for Placement {
    fn gender(&self) -> &str {
        &self.gender[..]
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum Field {
    Place,
    First,
    Last,
    City,
    State,
    Age,
    Gender,
    Gp,
    Time,
    Rank,
}

fn get_and_parse<T: FromStr>(
    values: &mut HashMap<Field, String>,
    field: Field,
    label: &str,
) -> Option<T>
where
    <T as FromStr>::Err: Debug,
{
    values
        .get(&field)
        .or_else(|| {
            eprintln!("no {label} in {values:?}");
            None
        })
        .and_then(|p| {
            p.parse()
                .map_err(|e| {
                    eprintln!("Can't convert {label}: {e:?}, values: {values:?}");
                    e
                })
                .ok()
        })
}

fn remove(values: &mut HashMap<Field, String>, field: Field, label: &str) -> Option<String> {
    values.remove(&field).or_else(|| {
        eprintln!("no {label} in {values:?}");
        None
    })
}

impl Placement {
    fn from_list_result<'a>(tds: impl Iterator<Item = ElementRef<'a>>) -> Option<Self> {
        use Field::*;

        let mut values = HashMap::new();
        for td in tds {
            if let Some(key) = td.value().attr("aria-describedby") {
                if let Some(key) = key.strip_prefix("list_") {
                    if let Some(field) = ARIA_FIELDS.get(key) {
                        let mut value = String::new();
                        value.extend(td.text());
                        values.insert(*field, value);
                    }
                }
            }
        }

        let place = get_and_parse(&mut values, Place, "place")?;
        let age = get_and_parse(&mut values, Age, "age")?;
        let gp = get_and_parse(&mut values, Gp, "gender place")?;
        let rank = get_and_parse(&mut values, Rank, "rank")?;

        // DNF and DNS may have blank times.  Elsewhere, they have 0
        // times.  In theory, DNF and DNS have 0 for both place and
        // gp, so if we see that, we give them a really large finish
        // time, rather than zero so that if the results somehow
        // sneaks through a filter and is summed into a valid number,
        // the result shows up better.

        let time = if place == 0 && gp == 0 {
            Duration::new(9_999_999, 0)
        } else {
            get_and_parse(&mut values, Time, "time")?
        };

        // We remove from least specific to most specific, since our
        // error messages dump values.  The error messages are just
        // there to help if we're parsing a file and getting
        // surprising results, so the fact that we're removing some
        // values before printing the error message is not that big of
        // a deal.

        let state = values.remove(&State);
        let city = values.remove(&City);
        let gender = remove(&mut values, Gender, "gender")?;
        let first = remove(&mut values, First, "first name")?;
        let last = remove(&mut values, Last, "last name")?;

        Some(Self {
            place,
            first,
            last,
            city,
            state,
            age,
            gender,
            gp,
            time,
            rank,
        })
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct StatusWithCount {
    status: String,
    count: u16,
}

#[derive(Debug)]
pub struct StatusesWithPlacements(Vec<(StatusWithCount, Vec<Placement>)>);

impl StatusesWithPlacements {
    fn results(contents: &str) -> Option<Self> {
        let mut results = None;
        let mut placements = None;

        let tbody = Selector::parse("tbody").unwrap();
        let tr = Selector::parse("tr").unwrap();
        let td = Selector::parse("td").unwrap();
        let document = Html::parse_document(contents);

        for body in document.select(&tbody) {
            for tr in body.select(&tr) {
                if tr.value().has_class("listghead_0", AsciiCaseInsensitive) {
                    let mut s = String::new();
                    s.extend(tr.text());
                    let pieces = s.split(" - ").collect::<Vec<_>>();
                    if pieces.len() == 2 {
                        if let Ok(count) = pieces[1].parse() {
                            let swc = StatusWithCount {
                                status: pieces[0].to_string(),
                                count,
                            };
                            match results.as_mut() {
                                None => {
                                    placements = Some(vec![]);
                                    results = Some(vec![(swc, vec![])])
                                }
                                Some(results) => {
                                    if let Some(mut placements) = placements.replace(vec![]) {
                                        if let Some(current) = results.last_mut() {
                                            mem::swap(&mut placements, &mut current.1);
                                            results.push((swc, placements));
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if let Some(placements) = placements.as_mut() {
                    let mut tds = tr.select(&td);
                    if let Some(td) = tds.next() {
                        if td.value().attr("aria-describedby") == Some("list_results") {
                            if let Some(placement) = Placement::from_list_result(tds) {
                                placements.push(placement);
                            }
                        }
                    }
                }
            }
        }
        if let Some(results) = results.as_mut() {
            if let Some(result) = results.last_mut() {
                if let Some(placements) = placements {
                    result.1 = placements;
                }
            }
        }
        results.map(Self)
    }

    pub fn names_and_times(input: &str) -> OptionalResults {
        Self::results(input).and_then(|swp| {
            swp.0
                .into_iter()
                .find(|(StatusWithCount { status, .. }, _)| status == "Finishers")
                .map(|(_, placements)| {
                    placements
                        .into_iter()
                        .map(|p| {
                            (
                                Cow::from(format!("{} {}", p.first, p.last)),
                                p.time,
                                p.morf(),
                            )
                        })
                        .collect()
                })
        })
    }
}
