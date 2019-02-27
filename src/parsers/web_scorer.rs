use crate::parsers::NameAndTime;
use nom::multispace;
use nom::rest;
use nom::types::CompleteStr;
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
        match results(CompleteStr(contents)) {
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

named!(results<CompleteStr, Vec<Placement>>,
       do_parse!(
           take_until_and_consume!("<tbody>") >>
           results: many0!(placement) >>
           (results)
       )
);

named!(placement<CompleteStr, Placement>,
       do_parse!(
           tr_line >>
               place: place >>
               bib: bib >>
               name_and_team: name_and_team >>
               category: category >>
               gender: gender >>
               finish_time: finish_time >>
               take_until_and_consume!("</tr>") >>
               ({
                   let finish_time = Duration::from_str(finish_time).unwrap();
                   let (name, team) = name_and_team;
                   Placement { place, bib, name, team, category, gender, finish_time }
               })
       )
);

named!(tr_line<CompleteStr, ()>,
       do_parse!(
           opt!(multispace) >>
               tag!("<tr class=\"") >>
               take_until_and_consume!("\"") >>
               tag!(">\r\n") >>
               (())
       )
);

named!(place<CompleteStr, u16>,
       flat_map!(
           call!(inside_td, "r-place"), parse_to!(u16)
       )
);

named_args!(inside_td<'a>(class: &'a str)<CompleteStr<'a>, &'a str>,
            do_parse!(
                opt!(multispace) >>
                    tag!("<td class='") >>
                    tag!(class) >>
                    tag!("'>") >>
                    value: take_until_and_consume!("</td>") >>
                    (&value)
            )
);

named!(bib<CompleteStr, Option<Cow<str>>>,
       do_parse!(
           bib: call!(optional_inside_td, "r-bibnumber") >>
               ({
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
               })
       )
);

named_args!(optional_inside_td<'a>(class: &'a str)<CompleteStr<'a>, Option<Cow<'a, str>>>,
            do_parse!(
                value: call!(inside_td, class) >>
                    ({
                        let value = value.trim();

                        match value {
                            "" => None,
                            _ => Some(html_decoded(value)),
                        }
                    })
            )
);

named!(name_and_team<CompleteStr, (Cow<str>, Option<Cow<str>>)>,
       do_parse!(
           name: call!(inside_td, "r-racername") >>
               (inner_name_and_team(CompleteStr(name)).unwrap().1)
       )
);

named!(inner_name_and_team<CompleteStr, (Cow<str>, Option<Cow<str>>)>,
       do_parse!(
           name: alt!( take_until!("<span class=\'team-name\'>") | call!(rest)) >>
               team: opt!(team) >>
               ({
                   if let Some(team) = team {
                       if team.trim().is_empty() {
                           (html_decoded(&name), None)
                       } else {
                           (html_decoded(&name), Some(team))
                       }
                   } else {
                       (html_decoded(&name), team)
                   }
               })
       )
);

named!(team<CompleteStr, Cow<str>>,
       do_parse!(
           tag!("<span class='team-name'>") >>
               team: take_until_and_consume!("</span>") >>
               ({
                   html_decoded(&team)
               })
       )
);
       

fn html_decoded(string: &str) -> Cow<str> {
    if let Ok(decoded_string) = htmlescape::decode_html(string) {
        if *string != decoded_string {
            return decoded_string.into();
        }
    }
    string.into()
}

named!(category<CompleteStr, Option<Cow<str>>>,
       call!(optional_inside_td, "r-category")
);

named!(gender<CompleteStr, Option<Cow<str>>>,
       call!(optional_inside_td, "r-gender")
);

named!(finish_time<CompleteStr, &str>,
       call!(inside_td, "r-finish-time")
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tr_line() {
        let line = "        <tr class=\"rowBg-mod0\">\r\n";

        assert_eq!((), tr_line(CompleteStr(line)).unwrap().1);
    }

    #[test]
    fn test_place() {
        let line = "            <td class='r-place'>1</td><td class='r-bibnumber'><span class='no-diff-hyphen'>-</span></td><td class='r-racername'>deadhead<span class='team-name'></span></td><td class='r-category'></td><td class='r-gender'>M</td>\r\n";

        assert_eq!(1, place(CompleteStr(line)).unwrap().1);
    }

    #[test]
    fn test_placement() {
        let lines = "        <tr class=\"rowBg-mod0\">\r\n            <td class='r-place'>1</td><td class='r-bibnumber'><span class='no-diff-hyphen'>-</span></td><td class='r-racername'>deadhead<span class='team-name'></span></td><td class='r-category'></td><td class='r-gender'>M</td>\r\n            \r\n            <td class='r-finish-time'>18:58.1</td><td class='r-difference'><span class='sel_ddDiffCol sel-D tabHide'><span class='no-diff-hyphen'>-</span></span><span class='sel_ddDiffCol sel-P tabHide'><span class='no-percent-hyphen'>-</span></span><span class='sel_ddDiffCol sel-WP tabHide'>100%</span><span class='sel_ddDiffCol sel-AP tabHide'>21.86%</span><span class='sel_ddDiffCol sel-MP tabHide'>22.24%</span></td>\r\n        </tr>\r\n";

        let placement = placement(CompleteStr(lines)).unwrap().1;
        println!("placement = {:?}", placement);
    }
}
