use crate::parsers::NameAndTime;

use crate::ccr_timing::{
    take_until_and_consume,
    parse_to,
};
    
use nom::{
    IResult,
    branch::alt,
    bytes::complete::{
        tag,
        take_until,
    },
    character::complete::multispace0,
    combinator::{
        opt,
        rest,
    },
    multi::many0,
};
use sports_metrics::duration::Duration;
use std::borrow::Cow;
use std::fmt;
use std::str::FromStr;

#[derive(Debug)]
pub struct Placement<'a> {
    pub place: u16,
    pub bib: Option<Cow<'a, str>>,
    pub name: Cow<'a, str>,
    pub team: Option<Cow<'a, str>>,
    pub category: Option<Cow<'a, str>>,
    pub gender: Option<Cow<'a, str>>,
    pub finish_time: Duration,
}

impl<'a> Placement<'a> {
    pub fn body_from(uri: &str) -> Option<String> {
        if uri.starts_with("https://www.webscorer.com/race?raceid=") {
            super::body_from(&uri)
        } else {
            None
        }
    }

    pub fn results(contents: &'a str) -> Option<Vec<Self>> {
        match results(contents) {
            Ok((_, results)) => Some(results),
            Err(_) => None,
        }
    }

    pub fn names_and_times(results: &'a [Self]) -> Vec<&'a dyn NameAndTime> {
        results.iter().map(|r| r as &dyn NameAndTime).collect()
    }
}

impl<'a> NameAndTime for Placement<'a> {
    fn name(&self) -> &str {
        &self.name
    }

    fn time(&self) -> Duration {
        self.finish_time
    }
}

impl<'a> fmt::Display for Placement<'a> {
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

fn results(input: &str) -> IResult<&str, Vec<Placement>> {
    let (input, _) = take_until_and_consume("<tbody>")(input)?;
    let (input, results) = many0(placement)(input)?;
    Ok((input, results))
}

fn placement(input: &str) -> IResult<&str, Placement> {
    let (input, _) = tr_line(input)?;
    let (input, place) = place(input)?;
    let (input, bib) = bib(input)?;
    let (input, name_and_team) = name_and_team(input)?;
    let (input, category) = category(input)?;
    let (input, gender) = gender(input)?;
    let (input, finish_time) = finish_time(input)?;
    let (input, _) = take_until_and_consume("</tr>")(input)?;
    Ok((input, {
        let finish_time = Duration::from_str(finish_time).unwrap();
        let (name, team) = name_and_team;
        Placement { place, bib, name, team, category, gender, finish_time }
    }))
}

fn tr_line(input: &str) -> IResult<&str, ()> {
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("<tr class=\"")(input)?;
    let (input, _) = take_until_and_consume("\"")(input)?;
    let (input, _) = tag(">\r\n")(input)?;
    Ok((input, ()))
}

fn place(input: &str) -> IResult<&str, u16> {
    let (input, digits) = inside_td(input, "r-place")?;
    let (_, number) = parse_to(digits)?;
    Ok((input, number))
}

fn inside_td<'a>(input: &'a str, class: &'a str) -> IResult<&'a str, &'a str> {
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("<td class='")(input)?;
    let (input, _) = tag(class)(input)?;
    let (input, _) = tag("'>")(input)?;
    let (input, value) = take_until_and_consume("</td>")(input)?;
    Ok((input, &value))
}

fn bib(input: &str) -> IResult<&str, Option<Cow<str>>> {
    let (input, bib) = optional_inside_td(input, "r-bibnumber")?;
    Ok((input, {
                   // TODO: Although it doesn't matter here, I'm not happy
                   // tearing apart and reconstructing bib below.  Logically,
                   // I want to special-case the return value for bib when
                   // it's that ugly span and otherwise, return bib (which
                   // has already been created as an Option<Cow<str>>).
                   //
                   // Before I added the html decoding, I simply did the match
                   // against Some("<span ...").  I don't know how to do that
                   // when "<span ..." is wrapped in a Cow.  I don't want to
                   // stop development now to find out, but it does seem worth
                   // learning eventually.
                   match bib {
                       Some(string) => {
                           if string == "<span class=\'no-diff-hyphen\'>-</span>" {
                               None
                           } else {
                               Some(string)
                           }
                       },
                       _ => bib
                   }
    }))
}

fn optional_inside_td<'a>(input: &'a str, class: &'a str) -> IResult<& 'a str, Option<Cow<'a, str>>> {
    let (input, value) = inside_td(input, class)?;
    Ok((input, {
        let value = value.trim();

        match value {
            "" => None,
            _ => Some(html_decoded(value)),
        }
    }))
}

fn name_and_team(input: &str) -> IResult<&str, (Cow<str>, Option<Cow<str>>)> {
    let (input, name) = inside_td(input, "r-racername")?;
    Ok((input, inner_name_and_team(name).unwrap().1))
}

fn inner_name_and_team(input: &str) -> IResult<&str, (Cow<str>, Option<Cow<str>>)> {
    let (input, name) = alt((take_until("<span class=\'team-name\'>"), rest))(input)?;
    let (input, team) = opt(team)(input)?;
    Ok((input, {
        if let Some(team) = team {
            if team.trim().is_empty() {
                (html_decoded(&name), None)
            } else {
                (html_decoded(&name), Some(team))
            }
        } else {
            (html_decoded(&name), team)
        }
    }))
}

fn team(input: &str) -> IResult<&str, Cow<str>> {
    let (input, _) = tag("<span class='team-name'>")(input)?;
    let (input, team) = take_until_and_consume("</span>")(input)?;
    Ok((input,  html_decoded(&team)))
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
    optional_inside_td(input, "r-category")
}

fn gender(input: &str) -> IResult<&str, Option<Cow<str>>> {
    optional_inside_td(input, "r-gender")
}

fn finish_time(input: &str) -> IResult<&str, &str> {
    inside_td(input, "r-finish-time")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tr_line() {
        let line = "        <tr class=\"rowBg-mod0\">\r\n";

        assert_eq!((), tr_line(line).unwrap().1);
    }

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
