// This one could benefit from using an HTML parser, but as
// proof-of-concept I just used a hack where I looked for the class
// name, but not the starting tag.

use {
    crate::{parser::take_until_and_consume, prelude::*},
    digital_duration_nom::duration::Duration,
    nom::{
        bytes::complete::tag,
        combinator::{map, map_parser, map_res},
        multi::many1,
        sequence::{preceded, terminated},
        IResult, Parser,
    },
};

#[allow(dead_code)]
#[derive(Debug)]
pub struct Placement<'a> {
    place: Cow<'a, str>, // Really? Not a u16? We don't use it anyway.
    // bib: String,
    name: Cow<'a, str>,
    gender: Cow<'a, str>,
    // city: String,
    time: Duration,
    // ...
}

impl Placement<'_> {
    fn results(contents: &str) -> Option<Vec<Placement>> {
        results(contents).ok().map(|(_, results)| results)
    }

    pub fn names_and_times(input: &str) -> OptionalResults {
        Self::results(input).map(|results| {
            results
                .into_iter()
                .map(|placement| {
                    let morf = placement.morf();
                    (placement.name, placement.time, morf)
                })
                .collect()
        })
    }
}
impl Gender for Placement<'_> {
    fn gender(&self) -> &str {
        self.gender.as_ref()
    }
}

fn results(input: &str) -> IResult<&str, Vec<Placement>> {
    preceded(take_until_and_consume("<tbody>"), many1(placement)).parse(input)
}

fn placement(input: &str) -> IResult<&str, Placement> {
    map(
        (
            preceded(tr_line, place),
            name,
            gender,
            terminated(time, take_until_and_consume("</tr>")),
        ),
        |(place, name, gender, time)| Placement {
            place,
            name,
            gender,
            time,
        },
    )
    .parse(input)
}

fn tr_line(input: &str) -> IResult<&str, (&str, &str)> {
    (take_until_and_consume("<tr"), take_until_and_consume(">")).parse(input)
}

fn place(input: &str) -> IResult<&str, Cow<str>> {
    inside_td("place").parse(input)
}

fn name(input: &str) -> IResult<&str, Cow<str>> {
    map(
        map_parser(
            inside_td("participantName"),
            (
                inside_div::<&str>("participantName__name__firstName"),
                inside_div::<&str>("participantName__name__lastName"),
            ),
        ),
        |(first, last)| format!("{first} {last}").into(),
    )
    .parse(input)
}

fn gender(input: &str) -> IResult<&str, Cow<str>> {
    preceded(
        tag("<td>"),
        map(take_until_and_consume("</td>"), |s: &str| s.into()),
    )
    .parse(input)
}

fn time(input: &str) -> IResult<&str, Duration> {
    // map_res(inside_td("time"), |digits: &str| digits.parse()).parse(input)
    map_res(inside_td("time"), |digits: &str| digits.parse()).parse(input)
}

fn inside_td<'a, T: From<&'a str>>(
    class: &'a str,
) -> impl FnMut(&'a str) -> IResult<&'a str, T> + 'a {
    inside_tag("td", class)
}

fn inside_div<'a, T: From<&'a str>>(
    class: &'a str,
) -> impl FnMut(&'a str) -> IResult<&'a str, T> + 'a {
    inside_tag("div", class)
}

// NOTE: inside_tag will throw away characters until it gets the td
// that has the class that it wants.  This allows us to discard entire
// <td>..</td> sequences that we don't care about.
fn inside_tag<'a, T: From<&'a str>>(
    tag: &'a str,
    class: &'a str,
) -> impl FnMut(&'a str) -> IResult<&'a str, T> + 'a {
    let initial_tag = format!("class=\"{class}\"");
    let closing_tag = format!("</{tag}>");
    move |input| {
        preceded(
            (
                take_until_and_consume(&initial_tag[..]),
                take_until_and_consume(">"),
            ),
            map(take_until_and_consume(&closing_tag[..]), |s: &str| s.into()),
        )
        .parse(input)
    }
}
