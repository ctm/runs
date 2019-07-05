#![allow(dead_code)]

use {
    crate::ccr_timing::take_until_and_consume,
    nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::{digit1, multispace0},
        combinator::{map, map_parser, map_res, opt, value},
        multi::many1,
        sequence::{preceded, terminated, tuple},
        IResult,
    },
    sports_metrics::{duration::Duration, option_display::OptionDisplay},
    std::{
        borrow::Cow,
        fmt::{self, Display, Formatter},
        num::{NonZeroU16, NonZeroU8},
        ops::RangeInclusive,
    },
};

#[derive(Clone, Debug)]
pub enum MaleOrFemale {
    Male,
    Female,
}

impl Display for MaleOrFemale {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Male => "M",
                Self::Female => "F",
            }
        )
    }
}

// This is the both the age group placement and the age group (not counting
// gender), because that's a single column in the run fit results

#[derive(Clone, Debug)]
pub enum AgeGroup {
    Open,
    AgeGroup(RangeInclusive<NonZeroU8>),
}

impl Display for AgeGroup {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Open => write!(f, "Open"),
            Self::AgeGroup(range) => write!(f, "{}-{}", range.start(), range.end()),
        }
    }
}

#[derive(Debug)]
pub struct AgeGroupPlace {
    pub place: NonZeroU16,
    pub age_group: AgeGroup,
}

impl Display for AgeGroupPlace {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.place, self.age_group)
    }
}

#[derive(Debug)]
pub struct Placement<'a> {
    pub place: NonZeroU16,
    pub name: &'a str,
    pub city: Option<&'a str>,
    pub bib: &'a str,
    pub age: NonZeroU8,
    pub gender: MaleOrFemale,
    pub age_group_place: AgeGroupPlace,
    pub chip_time: Duration,
    pub gun_time: Duration,
    pub pace: Option<&'a str>,
}

impl<'a> Placement<'a> {
    #[allow(dead_code)]
    pub fn body_from(uri: &str) -> Option<String> {
        if uri.contains("://irunfit.org/results/") {
            super::body_from(&uri)
        } else {
            None
        }
    }

    pub fn results<'b>(contents: &'b str) -> Option<Vec<Placement<'b>>> {
        match results(contents) {
            Ok((_, results)) => Some(results),
            Err(_) => None,
        }
    }

    pub fn names_and_times(input: &str) -> Option<Vec<(Cow<str>, Duration)>> {
        Self::results(input).map(|results| {
            results
                .into_iter()
                .map(|placement| (Cow::from(placement.name), placement.chip_time.clone()))
                .collect()
        })
    }
}

impl<'a> Display for Placement<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        type Printable<'a, 'b> = &'a dyn OptionDisplay<&'b str>;

        write!(
            f,
            "{:3} {:30} {:20} {:3} {:3} {:1} {:9} {:7.1} {:7.1} {:7}",
            self.place,
            self.name,
            &self.city as Printable,
            self.bib,
            self.age,
            self.gender,
            self.age_group_place,
            self.chip_time,
            self.gun_time,
            &self.pace as Printable,
        )
    }
}

fn results(input: &str) -> IResult<&str, Vec<Placement>> {
    preceded(
        take_until_and_consume("<table border=0 cellpadding=0 cellspacing=0 class=\"racetable\">"),
        many1(placement),
    )(input)
}

fn table_heading(input: &str) -> IResult<&str, ()> {
    value(
        (),
        tuple((tr, td, close_tr, take_until_and_consume("</tr>"))),
    )(input)
}

fn tr(input: &str) -> IResult<&str, &str> {
    preceded(multispace0, tag("<tr>"))(input)
}

fn close_tr(input: &str) -> IResult<&str, &str> {
    preceded(multispace0, tag("</tr>"))(input)
}

fn td(input: &str) -> IResult<&str, &str> {
    map(
        preceded(
            tuple((multispace0, tag("<td"), take_until_and_consume(">"))),
            take_until_and_consume("</td>"),
        ),
        |inner: &str| inner.trim(),
    )(input)
}

fn placement(input: &str) -> IResult<&str, Placement> {
    map(
        tuple((
            preceded(
                tuple((opt(table_heading), tr)),
                map_res(td, |digits: &str| digits.parse()),
            ),
            td,
            map(td, |string| {
                if string.is_empty() {
                    None
                } else {
                    Some(string)
                }
            }),
            td,
            map_res(td, |digits: &str| digits.parse()),
            map_parser(td, gender),
            map_parser(td, age_group_place),
            map_res(td, |duration| duration.parse()),
            map_res(td, |duration| duration.parse()),
            terminated(opt(td), close_tr),
        )),
        |(place, name, city, bib, age, gender, age_group_place, chip_time, gun_time, pace)| {
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
    )(input)
}

fn place(input: &str) -> IResult<&str, NonZeroU16> {
    map_res(digit1, |digits: &str| digits.parse())(input)
}

fn age_group_place(input: &str) -> IResult<&str, AgeGroupPlace> {
    map(
        tuple((terminated(place, tag(":")), age_group)),
        |(place, age_group)| AgeGroupPlace { place, age_group },
    )(input)
}

fn age_group(input: &str) -> IResult<&str, AgeGroup> {
    alt((
        value(AgeGroup::Open, tag("Open")),
        map(tuple((terminated(age, tag("-")), age)), |(start, end)| {
            AgeGroup::AgeGroup(start..=end)
        }),
    ))(input)
}

fn age(input: &str) -> IResult<&str, NonZeroU8> {
    map_res(preceded(multispace0, digit1), |digits: &str| digits.parse())(input)
}

fn gender(input: &str) -> IResult<&str, MaleOrFemale> {
    alt((
        value(MaleOrFemale::Male, tag("M")),
        value(MaleOrFemale::Female, tag("F")),
    ))(input)
}
