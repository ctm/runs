use {
    crate::parser::take_until_and_consume,
    digital_duration_nom::duration::Duration,
    nom::{
        branch::alt,
        bytes::complete::{tag, take},
        character::complete::multispace0,
        combinator::{cond, flat_map, map, map_parser, map_res, not, opt, value},
        multi::{many0, many_m_n},
        sequence::{preceded, terminated, tuple},
        IResult,
    },
    std::{borrow::Cow, cmp::Ordering, fmt},
};

#[derive(Clone, Copy, Debug)]
pub struct Placement<'a> {
    pub category: &'a str,
    pub category_place: u16,
    pub name: &'a str,
    pub bike_up: Option<Duration>,
    pub run_up: Option<Duration>,
    pub ski_up: Option<Duration>,
    pub shoe_up: Option<Duration>,
    pub total_up: Option<Duration>,
    pub shoe_down: Option<Duration>,
    pub ski_down: Option<Duration>,
    pub run_down: Option<Duration>,
    pub bike_down: Option<Duration>,
    pub total_down: Option<Duration>,
    pub total: Duration,
    pub bib: u16,
}

impl<'a> Placement<'a> {
    #[allow(dead_code)]
    pub fn body_from(uri: &str) -> Option<String> {
        if uri.starts_with("http://ccrtiming.com/events-results/") {
            super::body_from(&uri)
        } else {
            None
        }
    }

    pub fn soloist_names_and_times(input: &str) -> Option<Vec<(Cow<str>, Duration)>> {
        Results::results(input).map(|results| {
            results
                .soloists
                .iter()
                .map(|soloist| (Cow::from(soloist.name), soloist.total))
                .collect()
        })
    }
}

impl<'a> fmt::Display for Placement<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        type Printable<'a> = &'a dyn digital_duration_nom::option_display::OptionDisplay<Duration>;

        write!(
            f,
            "{:5} {:25} {:7} {:7} {:7} {:7} {:7} {:7} {:7} {:7} {:7} {:7} {:7} {:5} {}",
            self.category_place,
            self.name,
            &self.bike_up as Printable,
            &self.run_up as Printable,
            &self.ski_up as Printable,
            &self.shoe_up as Printable,
            &self.total_up as Printable,
            &self.shoe_down as Printable,
            &self.ski_down as Printable,
            &self.run_down as Printable,
            &self.bike_down as Printable,
            &self.total_down as Printable,
            self.total,
            self.bib,
            self.category
        )
    }
}

pub struct Results<'a> {
    pub soloists: Vec<Placement<'a>>,
    pub pairs: Vec<Placement<'a>>,
    pub teams: Vec<Placement<'a>>,
}

impl<'a> Results<'a> {
    pub fn results(contents: &'a str) -> Option<Self> {
        match results(contents) {
            Ok((_, results)) => Some(results),
            Err(_) => None,
        }
    }

    #[allow(dead_code)]
    pub fn sort_by(mut self, key: Sort) -> Self {
        #[allow(dead_code)]
        fn total_cmp(p1: &Placement, p2: &Placement) -> Ordering {
            p1.total.cmp(&p2.total)
        }

        #[allow(dead_code)]
        fn opt_cmp<V>(p1: &Placement, p2: &Placement, key: fn(&Placement) -> Option<V>) -> Ordering
        where
            V: std::cmp::Ord,
        {
            match key(p1) {
                Some(v1) => match key(p2) {
                    Some(v2) => v1.cmp(&v2),
                    None => Ordering::Less,
                },
                None => match key(p2) {
                    Some(_) => Ordering::Greater,
                    None => Ordering::Equal,
                },
            }
        }

        #[allow(dead_code)]
        fn shoe_down_cmp(p1: &Placement, p2: &Placement) -> Ordering {
            opt_cmp(p1, p2, |p| p.shoe_down)
        }

        let f = match key {
            Sort::Total => total_cmp,
            Sort::ShoeDown => shoe_down_cmp,
        };

        self.soloists.sort_by(f);
        self.pairs.sort_by(f);
        self.teams.sort_by(f);
        self
    }
}

#[allow(dead_code)]
pub enum Sort {
    Total,
    ShoeDown,
}

impl<'a> fmt::Display for Results<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Soloists")?;
        for soloist in &self.soloists {
            writeln!(f, "{}", soloist)?;
        }

        writeln!(f, "Pairs")?;
        for pair in &self.pairs {
            writeln!(f, "{}", pair)?;
        }

        writeln!(f, "Teams")?;
        for team in &self.teams {
            writeln!(f, "{}", team)?;
        }
        Ok(())
    }
}

pub fn results(input: &str) -> IResult<&str, Results> {
    map(
        tuple((
            discard_through("solo age groups"),
            all_category_blocks,
            discard_through("pairs age groups"),
            all_category_blocks,
            discard_through("team age groups"),
            all_category_blocks,
        )),
        |(_, soloists, _, pairs, _, teams)| Results {
            soloists,
            pairs,
            teams,
        },
    )(input)
}

fn all_category_blocks(input: &str) -> IResult<&str, Vec<Placement>> {
    map(
        many0(preceded(many0(junk_line), category_block)),
        |blocks| blocks.into_iter().flatten().collect(),
    )(input)
}

fn junk_line(input: &str) -> IResult<&str, &str> {
    preceded(
        not(alt((category_or_division_line, arrow_line))),
        take_until_and_consume("\r\n"),
    )(input)
}

//  To make arrow_line the same type as category_or_division_line, we
//  make it an Option<&str> but in reality we really only want it to
//  fail or succeed we don't care about the successful return value,
//  so we return None.
fn arrow_line(input: &str) -> IResult<&str, Option<&str>> {
    map(tag("<h3><a name=\""), |_| None)(input)
}

fn discard_through(name: &str) -> impl FnMut(&str) -> IResult<&str, ()> {
    let to_find = format!("<h3><a name=\"{}\"", name);

    move |input| {
        value(
            (),
            tuple((
                take_until_and_consume(&to_find[..]),
                take_until_and_consume("\r\n"),
            )),
        )(input)
    }
}

fn category_block(input: &str) -> IResult<&str, Vec<Placement>> {
    flat_map(
        terminated(category_or_division_line, many_m_n(2, 2, junk_line)),
        |category| many0(placement(category)),
    )(input)
}

fn category_or_division_line(input: &str) -> IResult<&str, Option<&str>> {
    preceded(opt(tag("<pre>")), alt((category_line, division_line)))(input)
}

fn category_line(input: &str) -> IResult<&str, Option<&str>> {
    map(
        preceded(tag("CATEGORY: "), take_until_and_consume("\r\n")),
        Some,
    )(input)
}

// If we have a division line, then the category will actually
// come from the placement line, which is in a slightly different
// format.  So, we return None to show that no *category* was
// found.  We throw away the division, because it's not useful.
fn division_line(input: &str) -> IResult<&str, Option<&str>> {
    value(
        None,
        tuple((tag("DIVISION: "), take_until_and_consume("\r\n"))),
    )(input)
}

fn placement<'a>(
    header_category: Option<&'a str>,
) -> impl FnMut(&'a str) -> IResult<&'a str, Placement<'a>> + 'a {
    map(
        tuple((
            terminated(right_justified_five_digit_number, tag(" ")),
            terminated(name(header_category), tag(" ")),
            terminated(optional_duration, tag(" ")),
            terminated(optional_duration, tag(" ")),
            terminated(optional_duration, tag(" ")),
            terminated(optional_duration, tag("  ")),
            terminated(optional_duration, tag(" ")),
            terminated(optional_duration, tag(" ")),
            terminated(optional_duration, tag(" ")),
            terminated(optional_duration, tag(" ")),
            terminated(optional_duration, tag("  ")),
            terminated(optional_duration, tag(" ")),
            terminated(non_blank_duration, tag(" ")),
            terminated(right_justified_five_digit_number, tag(" ")),
            terminated(
                cond(header_category.is_none(), upto_fourteen_characters),
                tuple((opt(tag("</pre>")), crlf)),
            ),
        )),
        move |(
            category_place,
            name,
            bike_up,
            run_up,
            ski_up,
            shoe_up,
            total_up,
            shoe_down,
            ski_down,
            run_down,
            bike_down,
            total_down,
            total,
            bib,
            category_column,
        )| {
            let total = total.unwrap();
            let category;
            match header_category {
                Some(value) => category = value,
                _ => category = &category_column.expect("no category anywhere"),
            }
            Placement {
                category,
                category_place,
                name,
                bike_up,
                run_up,
                ski_up,
                shoe_up,
                total_up,
                shoe_down,
                ski_down,
                run_down,
                bike_down,
                total_down,
                total,
                bib,
            }
        },
    )
}

fn crlf(input: &str) -> IResult<&str, &str> {
    tag("\r\n")(input)
}

fn right_justified_five_digit_number(input: &str) -> IResult<&str, u16> {
    map_res(take(5usize), |digits: &str| digits.trim_start().parse())(input)
}

fn upto_fourteen_characters(input: &str) -> IResult<&str, &str> {
    map(take(14usize), |letters: &str| letters.trim_end())(input)
}

fn name(category: Option<&str>) -> impl FnMut(&str) -> IResult<&str, &str> + '_ {
    move |input| {
        let n: usize = match category {
            None => 25,
            _ => 21,
        };
        map(take(n), |letters: &str| letters.trim())(input)
    }
}

fn optional_duration(input: &str) -> IResult<&str, Option<Duration>> {
    alt((blank_duration, non_blank_duration))(input)
}

fn blank_duration(input: &str) -> IResult<&str, Option<Duration>> {
    value(None, tag("       "))(input)
}

fn non_blank_duration(input: &str) -> IResult<&str, Option<Duration>> {
    map_parser(take(7usize), map(optionally_left_padded_duration, Some))(input)
}

fn optionally_left_padded_duration(input: &str) -> IResult<&str, Duration> {
    preceded(multispace0, digital_duration_nom::duration::duration_parser)(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_junk_line() {
        let junk = "This is junk\r\n";
        let cat = "CATEGORY: MALE 01-19\r\n";

        println!("junk_line(junk) = {:?}", junk_line(junk));
        println!("junk_line(cat) = {:?}", junk_line(cat));
    }

    #[test]
    fn test_category_block() {
        // Would like to use raw string literals, but I need the \r in them...

        let category = "CATEGORY: MALE 50-59\r\nPlace Name                  Bikeup  Runup   Skiup   Shoeup  Total Up Shoedn  Skidn   Rundn   Bikedn  Total Dn Total    Bib#    \r\n===== ===================== ======= ======= ======= ======= ======== ======= ======= ======= ======= ======== ======= ===== \r\n    1 Eric Black              59:27   53:41   35:49   22:09  2:51:04    9:42   18:55   46:17   52:33  2:07:26 4:58:30    45 \r\n    2 Peter Tempest           55:17   59:51   41:57   30:44  3:07:47   14:19   24:29   48:55   47:57  2:15:38 5:23:24   134 \r\n    3 Kenneth Gordon        1:03:20 1:01:55   43:07   29:28  3:17:48   13:38   30:30   45:23   52:24  2:21:55 5:39:43    73 \r\n    4 Frank Novotny         1:00:21 1:00:14   42:09   28:36  3:11:18   12:48   43:17   49:03   48:26  2:33:33 5:44:50   108 \r\n    5 Kevin Williams        1:05:53   55:01   46:30   32:30  3:19:53   12:31   29:55   47:06 1:00:08  2:29:39 5:49:32   144 \r\n    6 Clifford Matthews     5:05:05           47:35   31:21  3:30:01   10:33   30:42   53:50   50:18  2:25:22 5:55:22    99 \r\n\n    ";

        let division = "DIVISION: MIXED 30-39\r\n    1 Stranger Things             42:45   54:21   49:38   21:28  2:48:11           47:27   39:04   29:26  1:35:49 4:24:00   417 Team:MF30-39  \r\n    2 Team Drago                1:03:53   53:07   43:02   24:13  3:04:14   12:49   21:07   43:20   46:46  2:04:02 5:08:15   418 Team:MF30-39  \r\n    3 JHM Gym                   1:23:46   52:04   38:26   18:56  3:13:10    8:03   20:41   35:51   52:11  1:56:44 5:09:54   414 Team:MF30-39  \r\n    4 All Swedish No Finish     1:00:22   58:00   39:34   28:02  3:05:57   13:02   18:08   44:21   49:31  2:05:00 5:10:57   413 Team:MF30-39  \r\n    5 SaDaJoCla                 1:07:22   58:04   52:28   29:51  3:27:43   16:55   37:11   41:42   48:39  2:24:26 5:52:09   415 Team:MF30-39  \r\n    6 Three Legged Foxes        1:03:07   58:16   49:23   37:30  3:28:13   18:18   44:36   45:27   50:05  2:38:25 6:06:38   419 Team:MF30-39  \r\n    7 Self Inflicted Fitness    1:18:34 1:10:54   45:32   34:08  3:49:08   15:42                 5:26:12  2:55:39 6:44:46   416 Team:MF30-39  \r\n";

        let placements = category_block(category).unwrap().1;
        println!("placements = {:?}", placements);

        let placements = category_block(division).unwrap().1;
        println!("placements = {:?}", placements);
    }

    #[test]
    fn test_placement() {
        let line = "    6 Clifford Matthews     5:05:05           47:35   31:21  3:30:01   10:33   30:42   53:50   50:18  2:25:22 5:55:22    99 \r\n";
        let placement = placement(Some("MALE 50-59"))(line).unwrap().1;
        // TODO: compare to known good parse
        println!("placement = {:?}", placement);
    }

    #[test]
    fn test_optional_duration() {
        assert_eq!(None, optional_duration("       ").unwrap().1);

        assert_eq!(
            Some(Duration::new_hour_min_sec(5, 6, 7)),
            optional_duration("5:06:07").unwrap().1
        );
    }
}
