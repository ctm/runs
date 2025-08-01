use {
    crate::{parser::take_until_and_consume, prelude::*},
    digital_duration_nom::duration::Duration,
    nom::{
        IResult, Parser,
        combinator::{map, map_res},
        multi::many1,
        sequence::preceded,
    },
    std::{
        num::{NonZeroU8, NonZeroU16},
        str::FromStr,
    },
};

#[allow(dead_code)]
#[derive(Debug)]
pub struct Placement<'a> {
    pub rank: NonZeroU16,
    pub bib: Cow<'a, str>,
    pub time: Duration,
    pub first_name: Cow<'a, str>,
    pub last_name: Cow<'a, str>,
    pub age_group: Cow<'a, str>,
    pub city: Cow<'a, str>,
    pub state: Cow<'a, str>,
    pub gender: Cow<'a, str>,
    pub age: NonZeroU8,
}

impl<'a> Placement<'a> {
    #[allow(clippy::type_complexity)]
    pub fn new(
        (rank, bib, time, first_name, last_name, age_group, city, state, gender, age): (
            NonZeroU16,
            Cow<'a, str>,
            Duration,
            Cow<'a, str>,
            Cow<'a, str>,
            Cow<'a, str>,
            Cow<'a, str>,
            Cow<'a, str>,
            Cow<'a, str>,
            NonZeroU8,
        ),
    ) -> Self {
        Self {
            rank,
            bib,
            time,
            first_name,
            last_name,
            age_group,
            city,
            state,
            gender,
            age,
        }
    }

    pub fn results(contents: &str) -> Option<Vec<Placement>> {
        match results(contents) {
            Ok((_, results)) => Some(results),
            Err(_) => None,
        }
    }

    pub fn names_and_times(input: &str) -> OptionalResults {
        Self::results(input).map(|results| {
            let mut names_and_times: Vec<_> = results
                .into_iter()
                .map(|placement| {
                    (
                        Cow::from(placement.name()),
                        placement.time,
                        placement.morf(),
                    )
                })
                .collect();
            names_and_times.sort();
            names_and_times.dedup();
            names_and_times
        })
    }

    fn name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}

impl Gender for Placement<'_> {
    fn gender(&self) -> &str {
        self.gender.as_ref()
    }
}

fn results(input: &str) -> IResult<&str, Vec<Placement>> {
    preceded(take_until_and_consume("Age<br>"), many1(placement)).parse(input)
}

fn placement(input: &str) -> IResult<&str, Placement> {
    map(
        (
            parsed_tab,   // rank
            unparsed_tab, // bib
            parsed_tab,   // time
            unparsed_tab, // first_name
            unparsed_tab, // last_name
            unparsed_tab, // age_group
            unparsed_tab, // city
            unparsed_tab, // state
            unparsed_tab, // gender,
            map_res(
                take_until_and_consume("<br>"), // age
                |s: &str| s.parse(),
            ),
        ),
        Placement::new,
    )
    .parse(input)
}

fn parsed_tab<T: FromStr>(input: &str) -> IResult<&str, T> {
    map_res(unparsed_tab, |s| s.parse()).parse(input)
}

fn unparsed_tab(input: &str) -> IResult<&str, Cow<str>> {
    map(take_until_and_consume("\t"), |s: &str| {
        if s.contains("&nbsp;") {
            let mut s = s.to_string().replace("&nbsp;", " ");
            let trimmed = s.trim();
            if s != trimmed {
                s = trimmed.to_string();
            }
            From::from(s)
        } else {
            From::from(s)
        }
    })
    .parse(input)
}
