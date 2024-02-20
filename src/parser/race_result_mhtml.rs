use {
    crate::prelude::*,
    digital_duration_nom::duration::Duration,
    scraper::{ElementRef, Html, Selector},
    std::{collections::HashMap, fmt::Debug, num::NonZeroU16, str::FromStr},
};

// FWIW, considering we only use the 3-tuple of name, final time and
// male-or-female, I went out of my way to gather a few other fields and to
// keep them as &str rather than converting them to strings.  The gathering
// of the other fields is just because we might want to use them in some other
// project, so we may as well get the easy ones.
//
// I jumped through the lifetime hoops just to see if I could (i.e. to
// test myself and get practice doing it). When I started this project
// I didn't understand lifetimes as well as I do now. I still get
// spanked by the compiler quite a bit and I even had to use
// feature(closure_lifetime_binder) from nightly, but I was pretty
// tired while I worked on this projet (and often distracted by a
// toothache, whic is why I was tired).  So, I think it was a good use
// of my time.

// They don't explicitly list the gender, but at least for the El Paso
// Marathon in 2024, there are only two counts, 70 and 228.  So the
// fewest is presumably female and the most is male.  There's no guarantee
// that there will be exactly two counts in the future or even that the
// counts will be different, but this is good enough for now.

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct Placement<'a> {
    place: NonZeroU16,
    bib: NonZeroU16,
    name: &'a str,
    city_state: Option<&'a str>,
    // Country
    // AG Rank
    gender_rank: &'a str,
    // 13.1M
    // 19.1M
    final_time: Duration,
    pace: Duration,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum Field {
    Place,
    Bib,
    Name,
    CityState,
    GenderRank,
    FinalTime,
    Pace,
}

const N_FIELDS: usize = 7;

impl TryFrom<&str> for Field {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use Field::*;

        match value {
            "Place" => Ok(Place),
            "Bib" => Ok(Bib),
            "Name" => Ok(Name),
            "City, ST" => Ok(CityState),
            "Gender" => Ok(GenderRank),
            "Final Time" => Ok(FinalTime),
            "Pace" => Ok(Pace),
            _ => Err("unknown field"),
        }
    }
}

fn getter_constructor<T: FromStr>(
    offset_for_field: &HashMap<Field, usize>,
    field: Field,
) -> Option<impl Fn(&[ElementRef]) -> Option<T>> {
    str_getter_constructor(offset_for_field, field)
        .map(|getter| move |tds: &[ElementRef]| -> Option<T> { getter(tds)?.parse().ok() })
}

fn str_getter_constructor(
    offset_for_field: &HashMap<Field, usize>,
    field: Field,
) -> Option<impl for<'a> Fn(&[ElementRef<'a>]) -> Option<&'a str>> {
    let offset = *offset_for_field.get(&field)?;

    Some(
        for<'a, 'b> move |tds: &'a [ElementRef<'b>]| -> Option<&'b str> {
            tds.get(offset)?.text().next()
        },
    )
}

fn placements<'a>(
    table: ElementRef<'a>,
    offset_for_field: &HashMap<Field, usize>,
) -> Option<Vec<Placement<'a>>> {
    use Field::*;

    let place_getter = getter_constructor::<NonZeroU16>(offset_for_field, Place)?;
    let bib_getter = getter_constructor::<NonZeroU16>(offset_for_field, Bib)?;
    let name_getter = str_getter_constructor(offset_for_field, Name)?;
    let city_state_getter = str_getter_constructor(offset_for_field, CityState)?;
    let gender_rank_getter = str_getter_constructor(offset_for_field, GenderRank)?;
    let final_time_getter = getter_constructor::<Duration>(offset_for_field, FinalTime)?;
    let pace_getter = getter_constructor::<Duration>(offset_for_field, Pace)?;

    let td = Selector::parse("td").unwrap();
    let candidates = table
        .select(&Selector::parse("tbody tr").unwrap())
        .filter_map(|elem| {
            let tds = elem.select(&td).collect::<Vec<_>>();
            Some(Placement {
                place: place_getter(&tds)?,
                bib: bib_getter(&tds)?,
                name: name_getter(&tds)?,
                city_state: city_state_getter(&tds),
                gender_rank: gender_rank_getter(&tds)?,
                final_time: final_time_getter(&tds)?,
                pace: pace_getter(&tds)?,
            })
        })
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        None
    } else {
        Some(candidates)
    }
}

impl<'a> Placement<'a> {
    fn gender_count(&self) -> Option<&'a str> {
        self.gender_rank.split('/').nth(1)
    }

    fn results(document: &Html) -> Option<Vec<Placement>> {
        document
            .select(&Selector::parse("table.MainTable").unwrap())
            .next()
            .and_then(|table| {
                fields_for_indexes(table)
                    .and_then(|fields_for_indexes| placements(table, &fields_for_indexes))
            })
    }

    pub fn names_and_times(input: &str) -> OptionalResults {
        let document = Html::parse_document(input);
        Self::results(&document).and_then(|placements| {
            let (male, female) = Self::male_and_female_counts(&placements)?;
            Some(
                placements
                    .into_iter()
                    .map(|p| {
                        let morf = p.morf(male, female);
                        (p.name.to_string().into(), p.final_time, morf)
                    })
                    .collect(),
            )
        })
    }

    fn male_and_female_counts<'b>(placements: &[Placement<'b>]) -> Option<(&'b str, &'b str)> {
        let mut placements = placements.iter();
        let first = placements.next()?.gender_count()?;
        let mut second;
        while {
            second = placements.next()?.gender_count()?;
            second == first
        } {}
        let first_value: NonZeroU16 = first.parse().ok()?;
        let second_value: NonZeroU16 = second.parse().ok()?;
        Some(if first_value >= second_value {
            (first, second)
        } else {
            (second, first)
        })
    }

    fn morf(&self, male: &str, female: &str) -> Option<MaleOrFemale> {
        use MaleOrFemale::*;

        match self.gender_count() {
            None => panic!("No gender splitter"),
            Some(text) if text == male => Some(Male),
            Some(text) if text == female => Some(Female),
            Some(_text) => {
                eprintln!("Assuming Non-Binary: {self:?}");
                Some(NonBinary)
            }
        }
    }
}

fn fields_for_indexes(table: ElementRef) -> Option<HashMap<Field, usize>> {
    let result = HashMap::from_iter(
        table
            .select(&Selector::parse("thead th").unwrap())
            .enumerate()
            .filter_map(|(index, elem)| {
                elem.text()
                    .next()
                    .and_then(|t| t.try_into().ok().map(|f| (f, index)))
            }),
    );
    (result.len() == N_FIELDS).then_some(result)
}
