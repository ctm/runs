use {
    crate::{parser::take_until_and_consume, prelude::*},
    digital_duration_nom::duration::Duration,
    nom::{
        branch::alt,
        bytes::complete::{tag, take_until},
        character::complete::multispace0,
        combinator::{map, map_parser, map_res, opt, rest, value},
        error::Error,
        multi::many1,
        sequence::{preceded, terminated},
        IResult, Parser,
    },
    std::{fmt, str::FromStr},
};

#[allow(dead_code)]
#[derive(Debug)]
pub struct Placement<'a> {
    pub place: u16,
    pub bib: Option<Cow<'a, str>>,
    pub name: Cow<'a, str>,
    pub team: Option<Cow<'a, str>>,
    pub category: Option<Cow<'a, str>>,
    pub age: Option<u8>,
    pub gender: Option<Cow<'a, str>>,
    pub finish_time: Duration,
}

impl Placement<'_> {
    #[allow(dead_code)]
    pub fn body_from(uri: &str) -> Option<String> {
        if uri.starts_with("https://www.webscorer.com/race?raceid=") {
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
            results
                .into_iter()
                .map(|placement| {
                    let morf = placement.morf();
                    (placement.name, placement.finish_time, morf)
                })
                .collect()
        })
    }
}

impl fmt::Display for Placement<'_> {
    // NOTE: we're currently skipping category here
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let gender = match &self.gender {
            None => " ",
            Some(value) => value,
        };
        write!(
            f,
            "{:3} {:1} {:7.1} {:30}",
            self.place, gender, self.finish_time, &self.name
        )
    }
}

impl Gender for Placement<'_> {
    fn gender(&self) -> &str {
        self.gender.as_ref().map(|s| s.as_ref()).unwrap_or("")
    }
}

fn results(input: &str) -> IResult<&str, Vec<Placement>> {
    map(
        many1(preceded(
            take_until_and_consume("<tbody>"),
            many1(placement),
        )),
        |v| v.into_iter().flatten().collect::<Vec<_>>(),
    )
    .parse(input)
}

fn placement(input: &str) -> IResult<&str, Placement> {
    map(
        (
            preceded(tr_line, place),
            bib,
            name_and_team,
            category,
            age,
            gender,
            terminated(finish_time, take_until_and_consume("</tr>")),
        ),
        |(place, bib, name_and_team, category, age, gender, finish_time)| {
            let finish_time = Duration::from_str(finish_time).unwrap();
            let (name, team) = name_and_team;
            Placement {
                place,
                bib,
                name,
                team,
                category,
                age,
                gender,
                finish_time,
            }
        },
    )
    .parse(input)
}

fn tr_line(input: &str) -> IResult<&str, ()> {
    value(
        (),
        (
            multispace0,
            tag("<tr class=\""),
            take_until_and_consume("\""),
            tag(">\r\n"),
        ),
    )
    .parse(input)
}

fn place(input: &str) -> IResult<&str, u16> {
    map_res(inside_td("r-place"), |digits: &str| digits.parse()).parse(input)
}

#[allow(clippy::needless_lifetimes)]
fn inside_td<'a>(class: &'a str) -> impl Parser<&'a str, Error = Error<&'a str>, Output = &'a str> {
    preceded(
        (multispace0, tag("<td class='"), tag(class), tag("'>")),
        take_until_and_consume("</td>"),
    )
}

fn bib(input: &str) -> IResult<&str, Option<Cow<str>>> {
    map(optional_inside_td("r-bibnumber"), |bib| match bib {
        Some(ref string) if string == "<span class=\'no-diff-hyphen\'>-</span>" => None,
        _ => bib,
    })
    .parse(input)
}

#[allow(clippy::needless_lifetimes)]
fn optional_inside_td<'a>(
    class: &'a str,
) -> impl Parser<&'a str, Error = Error<&'a str>, Output = Option<Cow<'a, str>>> {
    map(inside_td(class), |value: &str| {
        let value = value.trim();

        match value {
            "" => None,
            _ => Some(html_decoded(value)),
        }
    })
}

fn name_and_team(input: &str) -> IResult<&str, (Cow<str>, Option<Cow<str>>)> {
    map_parser(inside_td("r-racername"), inner_name_and_team).parse(input)
}

fn inner_name_and_team(input: &str) -> IResult<&str, (Cow<str>, Option<Cow<str>>)> {
    map(
        (
            alt((take_until("<span class=\'team-name\'>"), rest)),
            opt(team),
        ),
        |(name, team)| {
            if let Some(team) = team {
                if team.trim().is_empty() {
                    (html_decoded(name), None)
                } else {
                    (html_decoded(name), Some(team))
                }
            } else {
                (html_decoded(name), team)
            }
        },
    )
    .parse(input)
}

fn team(input: &str) -> IResult<&str, Cow<str>> {
    map(
        preceded(
            tag("<span class='team-name'>"),
            take_until_and_consume("</span>"),
        ),
        html_decoded,
    )
    .parse(input)
}

fn html_decoded(string: &str) -> Cow<str> {
    if let Ok(decoded_string) = htmlescape::decode_html(string) {
        if *string != decoded_string {
            return decoded_string.into();
        }
    }
    string.into()
}

fn category(input: &str) -> IResult<&str, Option<Cow<str>>> {
    optional_inside_td("r-category").parse(input)
}

fn age(input: &str) -> IResult<&str, Option<u8>> {
    opt(map_res(inside_td("r-age"), |digits: &str| digits.parse())).parse(input)
}

fn gender(input: &str) -> IResult<&str, Option<Cow<str>>> {
    optional_inside_td("r-gender").parse(input)
}

fn finish_time(input: &str) -> IResult<&str, &str> {
    inside_td("r-finish-time").parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_place() {
        let line = "            <td class='r-place'>1</td><td class='r-bibnumber'><span class='no-diff-hyphen'>-</span></td><td class='r-racername'>deadhead<span class='team-name'></span></td><td class='r-category'></td><td class='r-gender'>M</td>\r\n";

        assert_eq!(1, place(line).unwrap().1);
    }

    #[test]
    fn test_placement() {
        let lines = "        <tr class=\"rowBg-mod0\">\r\n            <td class='r-place'>1</td><td class='r-bibnumber'><span class='no-diff-hyphen'>-</span></td><td class='r-racername'>deadhead<span class='team-name'></span></td><td class='r-category'></td><td class='r-gender'>M</td>\r\n            \r\n            <td class='r-finish-time'>18:58.1</td><td class='r-difference'><span class='sel_ddDiffCol sel-D tabHide'><span class='no-diff-hyphen'>-</span></span><span class='sel_ddDiffCol sel-P tabHide'><span class='no-percent-hyphen'>-</span></span><span class='sel_ddDiffCol sel-WP tabHide'>100%</span><span class='sel_ddDiffCol sel-AP tabHide'>21.86%</span><span class='sel_ddDiffCol sel-MP tabHide'>22.24%</span></td>\r\n        </tr>\r\n";

        let placement = placement(lines).unwrap().1;
        println!("placement = {:?}", placement);
    }
}
