use crate::parsers::NameAndTime;
use nom::{multispace, types::CompleteStr, IResult};
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
    pub fn results(contents: &'a str) -> Self {
        results(CompleteStr(contents)).unwrap().1
    }

    pub fn sort_by(mut self, key: Sort) -> Self {
        fn total_cmp(p1: &Placement, p2: &Placement) -> Ordering {
            p1.total.cmp(&p2.total)
        }

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
        write!(f, "") // TODO: better way to return success?
    }
}

named!(pub results<CompleteStr, Results>,
       do_parse!(
           call!(discard_through, "solo age groups") >>
               soloists: all_category_blocks >>
               call!(discard_through, "pairs age groups") >>
               pairs: all_category_blocks >>
               call!(discard_through, "team age groups") >>
               teams: all_category_blocks >>
               ( Results { soloists, pairs, teams } )
       )
);

named!(all_category_blocks<CompleteStr, Vec<Placement>>,
       do_parse!(
           blocks: many0!(preceded!(many0!(junk_line), category_block)) >>
           (blocks.into_iter().flatten().collect())
       )
);

named!(junk_line<CompleteStr, CompleteStr>,
       preceded!(not!(alt!(category_or_division_line | arrow_line)),
                 take_until_and_consume!("\r\n"))
);

// TODO: either refactor or better document.  To make arrow_line the
//       same tuype as category_or_division_line, we make it an Option<&str>
//       but in reality we really only want it to fail or succeed we don't
//       care about the successful return value, so we return None.
named!(arrow_line<CompleteStr, Option<&str>>,
       do_parse!(
           tag!("<h3><a name=\"") >>
               (None)
       )
);

// TODO: although the code for discard_through is tiny, I don't think
//       this is the best way to do it.  Most likely, there's already
//       a cleaner way to do this just using pre-existing macros.
//       However, if there's not, then writing my own macro would be
//       worthwhile.  That macro would support the generalization of
//       code that sets up a variable that's used in further macros.
//       There's a chance I could write such a thing right now, but
//       most likely I'd hit some stumbling blocks, first, so I'll
//       put it off until after I've done a bunch of other things.
fn discard_through<'a>(input: CompleteStr<'a>, name: &str) -> IResult<CompleteStr<'a>, ()> {
    let to_find = format!("<h3><a name=\"{}", name);

    do_parse!(
        input,
        take_until_and_consume!(&to_find[..]) >> take_until_and_consume!("\r\n") >> (())
    )
}

named!(category_block<CompleteStr, Vec<Placement>>,
       do_parse!(
           category: category_or_division_line >>
               junk_line >>
               junk_line >>
               placements: many0!(call!(placement, category)) >>
               (placements)
       )
);

named!(category_or_division_line<CompleteStr, Option<&str>>,
       preceded!(opt!(tag!("<pre>")), alt!(category_line | division_line))
);

named!(category_line<CompleteStr, Option<&str>>,
       do_parse!(
           tag!("CATEGORY: ") >>
               category: take_until_and_consume!("\r\n") >>
               (Some(&category))
       )
);

// If we have a division line, then the category will actually
// come from the placement line, which is in a slightly different
// format.  So, we return None to show that no *category* was
// found.  We throw away the division, because it's not useful.
named!(division_line<CompleteStr, Option<&str>>,
       do_parse!(
          tag!("DIVISION: ") >>
               take_until_and_consume!("\r\n") >>
               (None)
       )
);

named_args!(placement<'a>(header_category: Option<&'a str>)<CompleteStr<'a>, Placement<'a>>,
       do_parse!(
           category_place: right_justified_five_digit_number >> // 5
               tag!(" ") >>
               name: call!(name, header_category) >>
               tag!(" ") >>
               bike_up: optional_duration >>
               tag!(" ") >>
               run_up: optional_duration >>
               tag!(" ") >>
               ski_up: optional_duration >>
               tag!(" ") >>
               shoe_up: optional_duration >>
               tag!("  ") >>
               total_up: optional_duration >>
               tag!(" ") >>
               shoe_down: optional_duration >>
               tag!(" ") >>
               ski_down: optional_duration >>
               tag!(" ") >>
               run_down: optional_duration >>
               tag!(" ") >>
               bike_down: optional_duration >>
               tag!("  ") >>
               total_down: optional_duration >>
               tag!(" ") >>
               total: non_blank_duration >>
               tag!(" ") >>
               bib: right_justified_five_digit_number >>
               tag!(" ") >>
               category_column: cond!(header_category.is_none(), upto_fourteen_characters) >>
               opt!(tag!("</pre>")) >>
               crlf >>
               (
                   {
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
                   }
               )
       )
);

named!(crlf<CompleteStr, CompleteStr>,
       tag!("\r\n")
);

named!(right_justified_five_digit_number<CompleteStr, u16>,
       flat_map!(
           do_parse!(
               digits: take!(5) >>
                   (digits.trim_start())
           ),
           parse_to!(u16)
       )
);

named!(upto_fourteen_characters<CompleteStr, &str>,
       do_parse!(
           letters: take!(14) >>
               (letters.trim_end())
       )
);

named_args!(name<'a>(category: Option<&'a str>)<CompleteStr<'a>, &'a str>,
       do_parse!(
           letters: take!({
               match category {
                   None => 25,
                   _ => 21,
               }
           }) >>
               (letters.trim_end())
           )
);

named!(optional_duration<CompleteStr, Option<Duration>>,
       alt!(blank_duration | non_blank_duration)
);

named!(blank_duration<CompleteStr, Option<Duration>>,
       do_parse!(
           tag!("       ") >>
               (None)
       )
);

// In this particular case, we want exactly seven characters to
// represent a duration that may be left padded with spaces.  It's
// trivial to grab exactly seven characters, but I don't know of a way
// completely within Nom's macro system to then apply parsers to those
// seven characters.  However, I do know how the plumbing works, so I
// have created a helper that gets me where I want to be.
//
// TODO: There is probably a better way to do this and at some point I
// should ask for help.
named!(non_blank_duration<CompleteStr, Option<Duration>>,
       do_parse!(
           exactly_seven_chars: take!(7) >>
           duration: call!(lpd_helper, exactly_seven_chars) >>
               (Some(duration))
       )
);

// Ignore the "official" input and use the input that is passed as a separate
// parameter.  Ugh!
fn lpd_helper<'a>(
    remaining: CompleteStr<'a>,
    i: CompleteStr<'a>,
) -> IResult<CompleteStr<'a>, Duration> {
    match optionally_left_padded_duration(i) {
        Ok((_remaining, duration)) => Ok((remaining, duration)),
        Err(e) => Err(e),
    }
}

named!(optionally_left_padded_duration<CompleteStr, Duration>,
       do_parse!(
           opt!(multispace) >>
               duration: call!(sports_metrics::duration::duration_parser) >>
               (duration)
       )
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_results() {
        use std::{fs::File, io::Read};

        let mut file = File::open("assets/2019.html").unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let results = results(CompleteStr(&contents))
            .unwrap()
            .1
            .sort_by(Sort::ShoeDown);

        println!("results = {}", results);
        // TODO
    }

    #[test]
    fn test_junk_line() {
        let junk = CompleteStr("This is junk\r\n");
        let cat = CompleteStr("CATEGORY: MALE 01-19\r\n");

        println!("junk_line(junk) = {:?}", junk_line(junk));
        println!("junk_line(cat) = {:?}", junk_line(cat));
    }

    #[test]
    fn test_category_block() {
        // Would like to use raw string literals, but I need the \r in them...

        let category = "CATEGORY: MALE 50-59\r\nPlace Name                  Bikeup  Runup   Skiup   Shoeup  Total Up Shoedn  Skidn   Rundn   Bikedn  Total Dn Total    Bib#    \r\n===== ===================== ======= ======= ======= ======= ======== ======= ======= ======= ======= ======== ======= ===== \r\n    1 Eric Black              59:27   53:41   35:49   22:09  2:51:04    9:42   18:55   46:17   52:33  2:07:26 4:58:30    45 \r\n    2 Peter Tempest           55:17   59:51   41:57   30:44  3:07:47   14:19   24:29   48:55   47:57  2:15:38 5:23:24   134 \r\n    3 Kenneth Gordon        1:03:20 1:01:55   43:07   29:28  3:17:48   13:38   30:30   45:23   52:24  2:21:55 5:39:43    73 \r\n    4 Frank Novotny         1:00:21 1:00:14   42:09   28:36  3:11:18   12:48   43:17   49:03   48:26  2:33:33 5:44:50   108 \r\n    5 Kevin Williams        1:05:53   55:01   46:30   32:30  3:19:53   12:31   29:55   47:06 1:00:08  2:29:39 5:49:32   144 \r\n    6 Clifford Matthews     5:05:05           47:35   31:21  3:30:01   10:33   30:42   53:50   50:18  2:25:22 5:55:22    99 \r\n\n    ";

        let division = "DIVISION: MIXED 30-39\r\n    1 Stranger Things             42:45   54:21   49:38   21:28  2:48:11           47:27   39:04   29:26  1:35:49 4:24:00   417 Team:MF30-39  \r\n    2 Team Drago                1:03:53   53:07   43:02   24:13  3:04:14   12:49   21:07   43:20   46:46  2:04:02 5:08:15   418 Team:MF30-39  \r\n    3 JHM Gym                   1:23:46   52:04   38:26   18:56  3:13:10    8:03   20:41   35:51   52:11  1:56:44 5:09:54   414 Team:MF30-39  \r\n    4 All Swedish No Finish     1:00:22   58:00   39:34   28:02  3:05:57   13:02   18:08   44:21   49:31  2:05:00 5:10:57   413 Team:MF30-39  \r\n    5 SaDaJoCla                 1:07:22   58:04   52:28   29:51  3:27:43   16:55   37:11   41:42   48:39  2:24:26 5:52:09   415 Team:MF30-39  \r\n    6 Three Legged Foxes        1:03:07   58:16   49:23   37:30  3:28:13   18:18   44:36   45:27   50:05  2:38:25 6:06:38   419 Team:MF30-39  \r\n    7 Self Inflicted Fitness    1:18:34 1:10:54   45:32   34:08  3:49:08   15:42                 5:26:12  2:55:39 6:44:46   416 Team:MF30-39  \r\n";

        let placements = category_block(CompleteStr(category)).unwrap().1;
        println!("placements = {:?}", placements);

        let placements = category_block(CompleteStr(division)).unwrap().1;
        println!("placements = {:?}", placements);
    }

    #[test]
    fn test_placement() {
        let line = "    6 Clifford Matthews     5:05:05           47:35   31:21  3:30:01   10:33   30:42   53:50   50:18  2:25:22 5:55:22    99 \r\n";
        let placement = placement(CompleteStr(line), Some("MALE 50-59")).unwrap().1;
        // TODO: compare to known good parse
        println!("placement = {:?}", placement);
    }

    #[test]
    fn test_optional_duration() {
        assert_eq!(None, optional_duration(CompleteStr("       ")).unwrap().1);

        assert_eq!(
            Some(Duration::new_hour_min_sec(5, 6, 7)),
            optional_duration(CompleteStr("5:06:07")).unwrap().1
        );
    }
}
