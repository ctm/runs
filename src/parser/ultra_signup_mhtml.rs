// NOTE: this started out as the web_scorer HTML scraper, because that
// one handles tr elements nicely.  However, to make this more like
// the other UltraSignup scrapers, I'll need to take the various
// statuses into account, which is more work.

use {
    crate::parser::take_until_and_consume,
    digital_duration_nom::duration::Duration,
    nom::{
        combinator::{map, map_res, value},
        multi::many1,
        sequence::{preceded, terminated, tuple},
        IResult,
    },
    std::{
        borrow::Cow::{self, Borrowed},
        num::NonZeroU8,
    },
};

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct Placement<'a> {
    place: u16,
    first: Cow<'a, str>,
    last: Cow<'a, str>,
    city: Option<Cow<'a, str>>,
    state: Option<Cow<'a, str>>,
    age: NonZeroU8,
    gender: Cow<'a, str>,
    gp: u16,
    time: Duration,
    rank: f32,
}

impl<'a> Placement<'a> {
    fn results(contents: &str) -> Option<Vec<Placement>> {
        results(contents).ok().map(|(_, results)| results)
    }

    pub fn names_and_times(input: &str) -> Option<Vec<(Cow<str>, Duration)>> {
        Self::results(input).map(|results| {
            results
                .into_iter()
                .map(|placement| {
                    (
                        format!("{} {}", placement.first, placement.last).into(),
                        placement.time,
                    )
                })
                .collect()
        })
    }
}

fn results(input: &str) -> IResult<&str, Vec<Placement>> {
    preceded(take_until_and_consume("><tbody>"), many1(placement))(input)
}

fn placement(input: &str) -> IResult<&str, Placement> {
    map(
        tuple((
            preceded(tr_line, place),
            first,
            last,
            city,
            state,
            age,
            gender,
            gp,
            time,
            terminated(rank, take_until_and_consume("</tr>")),
        )),
        |(place, first, last, city, state, age, gender, gp, time, rank)| Placement {
            place,
            first: Borrowed(first),
            last: Borrowed(last),
            city,
            state,
            age,
            gender: Borrowed(gender),
            gp,
            time,
            rank,
        },
    )(input)
}

fn tr_line(input: &str) -> IResult<&str, ()> {
    value(
        (),
        tuple((
            take_until_and_consume("<tr role=\"row\""),
            take_until_and_consume(">"),
        )),
    )(input)
}

fn place(input: &str) -> IResult<&str, u16> {
    map_res(inside_td("list_place"), |digits: &str| digits.parse())(input)
}

fn first(input: &str) -> IResult<&str, &str> {
    inside_td("list_firstname")(input)
}

fn last(input: &str) -> IResult<&str, &str> {
    inside_td("list_lastname")(input)
}

fn city(input: &str) -> IResult<&str, Option<Cow<str>>> {
    optional_inside_td("list_city")(input)
}

fn state(input: &str) -> IResult<&str, Option<Cow<str>>> {
    optional_inside_td("list_state")(input)
}

fn age(input: &str) -> IResult<&str, NonZeroU8> {
    map_res(inside_td("list_age"), |digits: &str| digits.parse())(input)
}

fn gender(input: &str) -> IResult<&str, &str> {
    inside_td("list_gender")(input)
}

fn gp(input: &str) -> IResult<&str, u16> {
    map_res(inside_td("list_gender_place"), |digits: &str| {
        digits.parse()
    })(input)
}

fn time(input: &str) -> IResult<&str, Duration> {
    map_res(inside_td("list_formattime"), |digits: &str| digits.parse())(input)
}

fn rank(input: &str) -> IResult<&str, f32> {
    map_res(inside_td("list_runner_rank"), |digits: &str| digits.parse())(input)
}

// NOTE: inside_td will throw away characters until it gets the td
// that has the aria-describedby that it wants.  This allows us to
// discard entire <td>..</td> sequences that we don't care about.
fn inside_td<'a>(aria: &'a str) -> impl FnMut(&'a str) -> IResult<&str, &str> {
    let initial_tag = format!("aria-describedby=\"{aria}\">");
    move |input| {
        preceded(
            take_until_and_consume(&initial_tag[..]),
            take_until_and_consume("</td>"),
        )(input)
    }
}

#[allow(clippy::needless_lifetimes)]
fn optional_inside_td<'a>(aria: &'a str) -> impl FnMut(&'a str) -> IResult<&str, Option<Cow<str>>> {
    map(inside_td(aria), |value: &str| {
        let value = value.trim();
        if value.is_empty() {
            None
        } else {
            Some(Borrowed(value))
        }
    })
}
