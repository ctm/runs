use crate::parsers::NameAndTime;

use std::ops::{Range, RangeFrom, RangeTo};

use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_until},
    character::complete::multispace0,
    combinator::{cond, not, opt, rest},
    error::{make_error, ErrorKind, ParseError},
    multi::many0,
    sequence::preceded,
    Compare, Err, FindSubstring, IResult, InputLength, InputTake, ParseTo, Slice,
};

use sports_metrics::duration::Duration;
use std::{cmp::Ordering, fmt};

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

    // TODO: I believe we can do this generically
    pub fn names_and_times(results: &'a [Self]) -> Vec<&'a dyn NameAndTime> {
        results.iter().map(|r| r as &dyn NameAndTime).collect()
    }
}

impl<'a> NameAndTime for Placement<'a> {
    fn name(&self) -> &str {
        &self.name
    }

    fn time(&self) -> Duration {
        self.total
    }
}

impl<'a> fmt::Display for Placement<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        type Printable<'a> = &'a dyn sports_metrics::duration::Printable<Duration>;

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
    let (input, _) = discard_through(input, "solo age groups")?;
    let (input, soloists) = all_category_blocks(input)?;
    let (input, _) = discard_through(input, "pairs age groups")?;
    let (input, pairs) = all_category_blocks(input)?;
    let (input, _) = discard_through(input, "team age groups")?;
    let (input, teams) = all_category_blocks(input)?;
    Ok((
        input,
        Results {
            soloists,
            pairs,
            teams,
        },
    ))
}

fn all_category_blocks(input: &str) -> IResult<&str, Vec<Placement>> {
    let (input, blocks) = many0(preceded(many0(junk_line), category_block))(input)?;
    Ok((input, blocks.into_iter().flatten().collect()))
}

// TODO: move this elsewhere
pub fn take_until_and_consume<T, Input, Error: ParseError<Input>>(
    tag_to_match: T,
) -> impl Fn(Input) -> IResult<Input, Input, Error>
where
    Input: InputTake + FindSubstring<T> + Compare<T>,
    T: InputLength + Clone,
{
    move |input| {
        let cloned_tag_to_match = tag_to_match.clone();
        let (input, res) = take_until(cloned_tag_to_match)(input)?;
        let cloned_tag_to_match = tag_to_match.clone();
        let (input, _) = tag(cloned_tag_to_match)(input)?;
        Ok((input, res))
    }
}

fn junk_line(input: &str) -> IResult<&str, &str> {
    let (input, res) = preceded(
        not(alt((category_or_division_line, arrow_line))),
        take_until_and_consume("\r\n"),
    )(input)?;
    Ok((input, res))
}

//  To make arrow_line the same type as category_or_division_line, we
//  make it an Option<&str> but in reality we really only want it to
//  fail or succeed we don't care about the successful return value,
//  so we return None.
fn arrow_line(input: &str) -> IResult<&str, Option<&str>> {
    let (input, _) = tag("<h3><a name=\"")(input)?;
    Ok((input, None))
}

fn discard_through<'a>(input: &'a str, name: &str) -> IResult<&'a str, ()> {
    let to_find = format!("<h3><a name=\"{}\"", name);

    let (input, _) = take_until_and_consume(&to_find[..])(input)?;
    let (input, _) = take_until_and_consume("\r\n")(input)?;
    Ok((input, ()))
}

fn category_block(input: &str) -> IResult<&str, Vec<Placement>> {
    let (input, category) = category_or_division_line(input)?;
    let (input, _) = junk_line(input)?;
    let (input, _) = junk_line(input)?;
    let (input, placements) = many0(move |i| placement(i, category))(input)?;
    Ok((input, placements))
}

fn category_or_division_line(input: &str) -> IResult<&str, Option<&str>> {
    let (input, res) = preceded(opt(tag("<pre>")), alt((category_line, division_line)))(input)?;
    Ok((input, res))
}

fn category_line(input: &str) -> IResult<&str, Option<&str>> {
    let (input, _) = tag("CATEGORY: ")(input)?;
    let (input, category) = take_until_and_consume("\r\n")(input)?;
    Ok((input, Some(&category)))
}

// If we have a division line, then the category will actually
// come from the placement line, which is in a slightly different
// format.  So, we return None to show that no *category* was
// found.  We throw away the division, because it's not useful.
fn division_line(input: &str) -> IResult<&str, Option<&str>> {
    let (input, _) = tag("DIVISION: ")(input)?;
    let (input, _) = take_until_and_consume("\r\n")(input)?;
    Ok((input, None))
}

fn placement<'a>(
    input: &'a str,
    header_category: Option<&'a str>,
) -> IResult<&'a str, Placement<'a>> {
    let (input, category_place) = right_justified_five_digit_number(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, name) = name(input, header_category)?;
    let (input, _) = tag(" ")(input)?;
    let (input, bike_up) = optional_duration(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, run_up) = optional_duration(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, ski_up) = optional_duration(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, shoe_up) = optional_duration(input)?;
    let (input, _) = tag("  ")(input)?;
    let (input, total_up) = optional_duration(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, shoe_down) = optional_duration(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, ski_down) = optional_duration(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, run_down) = optional_duration(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, bike_down) = optional_duration(input)?;
    let (input, _) = tag("  ")(input)?;
    let (input, total_down) = optional_duration(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, total) = non_blank_duration(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, bib) = right_justified_five_digit_number(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, category_column) =
        cond(header_category.is_none(), upto_fourteen_characters)(input)?;
    let (input, _) = opt(tag("</pre>"))(input)?;
    let (input, _) = crlf(input)?;
    Ok((input, {
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
    }))
}

fn crlf(input: &str) -> IResult<&str, &str> {
    let (input, res) = tag("\r\n")(input)?;
    Ok((input, res))
}

// TODO: move this elsewhere
pub fn parse_to<T, R>(input: T) -> IResult<T, R>
where
    T: ParseTo<R>
        + InputLength
        + Slice<Range<usize>>
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
{
    let (input, source) = rest(input)?;
    match source.parse_to() {
        Some(number) => Ok((input, number)),
        None => Err(Err::Error(make_error(input, ErrorKind::ParseTo))), // TODO: a better error
    }
}

fn right_justified_five_digit_number(input: &str) -> IResult<&str, u16> {
    let (input, digits) = take(5usize)(input)?;
    let (_, number) = parse_to(digits.trim_start())?;
    Ok((input, number))
}

fn upto_fourteen_characters(input: &str) -> IResult<&str, &str> {
    let (input, letters) = take(14usize)(input)?;
    Ok((input, letters.trim_end()))
}

fn name<'a>(input: &'a str, category: Option<&'a str>) -> IResult<&'a str, &'a str> {
    let (input, letters) = take({
        match category {
            None => 25usize,
            _ => 21usize,
        }
    })(input)?;
    Ok((input, letters.trim()))
}

fn optional_duration(input: &str) -> IResult<&str, Option<Duration>> {
    let (input, res) = alt((blank_duration, non_blank_duration))(input)?;
    Ok((input, res))
}

fn blank_duration(input: &str) -> IResult<&str, Option<Duration>> {
    let (input, _) = tag("       ")(input)?;
    Ok((input, None))
}

// NOTE: if this parser errors out, the error in the input that will be
//       listed will be the seven characters.  That's OK for the way we
//       use this parser, but it is probably not a good practice.
fn non_blank_duration(input: &str) -> IResult<&str, Option<Duration>> {
    let (input, exactly_seven_chars) = take(7usize)(input)?;
    let (_, duration) = optionally_left_padded_duration(exactly_seven_chars)?;
    Ok((input, Some(duration)))
}

fn optionally_left_padded_duration(input: &str) -> IResult<&str, Duration> {
    let (input, _) = multispace0(input)?;
    let (input, duration) = sports_metrics::duration::duration_parser(input)?;
    Ok((input, duration))
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
        let placement = placement(line, Some("MALE 50-59")).unwrap().1;
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
