// NOTE: Even though this is called RunFit, it works for the Ruidoso
//       Marathon, which isn't, AFAIK, associated with RunFit.  So, it
//       appears that they both use the same software, but I don't yet
//       know that software's name.

use {
    crate::{parser::take_until_and_consume, prelude::*},
    digital_duration_nom::{duration::Duration, option_display::OptionDisplay},
    nom::{
        IResult, Parser,
        branch::alt,
        bytes::complete::{tag, take_until},
        character::complete::multispace0,
        combinator::{map, map_parser, map_res, opt, peek, value},
        multi::{many0, many1},
        sequence::{preceded, terminated},
    },
    std::{
        fmt::{self, Display, Formatter},
        num::{NonZeroU8, NonZeroU16},
    },
};

#[derive(Debug)]
pub struct Placement<'a> {
    pub place: NonZeroU16,
    pub name: &'a str,
    pub city: Option<&'a str>,
    pub bib: &'a str,
    pub age: Option<NonZeroU8>,
    pub gender: Option<MaleOrFemale>,
    pub age_group_place: Option<&'a str>,
    pub chip_time: Duration,
    pub gun_time: Option<Duration>,
    pub pace: Option<&'a str>,
}

impl Placement<'_> {
    #[allow(dead_code)]
    pub fn body_from(uri: &str) -> Option<String> {
        if uri.contains("://irunfit.org/results/") {
            super::body_from(uri)
        } else {
            None
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
                        Cow::from(placement.name),
                        placement.chip_time,
                        placement.gender,
                    )
                })
                .collect();
            names_and_times.sort();
            names_and_times.dedup();
            names_and_times
        })
    }
}

impl Display for Placement<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{:3} {:30} {:20} {:3} {:3} {:1} {:9} {:7.1} {:7.1} {:7}",
            self.place,
            self.name,
            &self.city as &dyn OptionDisplay<&str>,
            self.bib,
            &self.age as &dyn OptionDisplay<NonZeroU8>,
            &self.gender as &dyn OptionDisplay<MaleOrFemale>,
            &self.age_group_place as &dyn OptionDisplay<&str>,
            self.chip_time,
            &self.gun_time as &dyn OptionDisplay<Duration>,
            &self.pace as &dyn OptionDisplay<&str>,
        )
    }
}

fn results(input: &str) -> IResult<&str, Vec<Placement>> {
    preceded(
        take_until_and_consume("<table border=0 cellpadding=0 cellspacing=0 class=\"racetable\">"),
        many1(placement),
    )
    .parse(input)
}

fn tr(input: &str) -> IResult<&str, &str> {
    preceded(multispace0, tag("<tr>")).parse(input)
}

fn close_tr(input: &str) -> IResult<&str, &str> {
    preceded(multispace0, tag("</tr>")).parse(input)
}

fn td(input: &str) -> IResult<&str, &str> {
    map(
        preceded(
            (multispace0, tag("<td"), take_until_and_consume(">")),
            take_until_and_consume("</td>"),
        ),
        |inner: &str| inner.trim(),
    )
    .parse(input)
}

fn td_duration(input: &str) -> IResult<&str, Duration> {
    map_res(td, |digits: &str| digits.parse()).parse(input)
}

fn placement(input: &str) -> IResult<&str, Placement> {
    map(
        (
            preceded(
                (many0(heading_tr), tr),
                map_res(td, |digits: &str| digits.parse()), // place
            ),
            td, // name
            map(td, |string| {
                // city
                if string.is_empty() {
                    None
                } else {
                    Some(string)
                }
            }),
            td, // bib
            map(td, |string| {
                // age
                if string.is_empty() {
                    None
                } else {
                    string.parse().ok()
                }
            }),
            opt(map_parser(td, gender)), // gender
            opt(td),                     // age group place
            chip_and_gun_time,
            terminated(opt(td), close_tr), // pace
        ),
        |(place, name, city, bib, age, gender, age_group_place, (chip_time, gun_time), pace)| {
            Placement {
                place,
                name,
                city,
                bib,
                age,
                gender,
                age_group_place,
                chip_time,
                gun_time,
                pace,
            }
        },
    )
    .parse(input)
}

fn gender(input: &str) -> IResult<&str, MaleOrFemale> {
    alt((
        value(MaleOrFemale::Male, tag("M")),
        value(MaleOrFemale::Female, tag("F")),
        value(MaleOrFemale::NonBinary, tag("X")),
    ))
    .parse(input)
}

// Returns a chip time and an optional gun time, but first tries to
// consume and discard an optional time back (which I first
// encountered in the Sandia Mountain Shadows Trail Run).
//
// We can't just consume all three times as an optional Duration,
// followed by a non-optional Duration followed by an optional
// Duration, because that would get us off track, so we explicitly try
// for an (optional td, Duration, Duration) tuple and if that fails,
// try for a (Duration, optional Duration) tuple.
//
// This sufficiently ad hoc that we need to run regression tests
// against all our Run Fit assets each time we change this code.
fn chip_and_gun_time(input: &str) -> IResult<&str, (Duration, Option<Duration>)> {
    alt((
        map(
            (opt(td), td_duration, td_duration),
            |(_time_back, chip, gun)| (chip, Some(gun)),
        ),
        (td_duration, opt(td_duration)),
    ))
    .parse(input)
}

// ========================================================================

fn heading_tr(input: &str) -> IResult<&str, ()> {
    value((), (multispace0, tr, many1(heading_td), close_tr)).parse(input)
}

fn heading_td(input: &str) -> IResult<&str, ()> {
    value(
        (),
        (
            multispace0,
            alt((tag("<td class=h"), navigation_open_td)),
            take_until_and_consume("</td>"),
        ),
    )
    .parse(input)
}

// Ugh! The Ruidoso Marathon has navigation tds that use a class that starts
// with a "d", which means I need to see if there's a colspan to figure out
// if this is one of those.
fn navigation_open_td(input: &str) -> IResult<&str, &str> {
    preceded(
        tag("<td"),
        peek(map_parser(take_until(">"), take_until("colspan=\""))),
    )
    .parse(input)
}
