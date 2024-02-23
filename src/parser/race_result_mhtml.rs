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
    type Error = String;

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
            field => Err(format!("unknown field: {field}")),
        }
    }
}

fn getter<T: FromStr>(
    offset_for_field: &HashMap<Field, usize>,
    field: Field,
) -> Option<impl Fn(&[ElementRef]) -> Option<T>> {
    str_getter(offset_for_field, field)
        .map(|getter| move |tds: &[ElementRef]| -> Option<T> { getter(tds)?.parse().ok() })
}

fn str_getter(
    offset_for_field: &HashMap<Field, usize>,
    field: Field,
) -> Option<impl for<'doc> Fn(&[ElementRef<'doc>]) -> Option<&'doc str>> {
    let offset = *offset_for_field.get(&field)?;

    Some(
        for<'tds, 'doc> move |tds: &'tds [ElementRef<'doc>]| -> Option<&'doc str> {
            tds.get(offset)?.text().next()
        },
    )
}

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct Placement<'doc> {
    place: NonZeroU16,
    bib: NonZeroU16,
    name: &'doc str,
    city_state: Option<&'doc str>,
    // Country
    // AG Rank
    gender_rank: &'doc str,
    // 13.1M
    // 19.1M
    final_time: Duration,
    pace: Duration,
}

fn placements<'doc>(
    table: ElementRef<'doc>,
    offset_for_field: &HashMap<Field, usize>,
) -> Option<Vec<Placement<'doc>>> {
    use Field::*;

    let place = getter::<NonZeroU16>(offset_for_field, Place)?;
    let bib = getter::<NonZeroU16>(offset_for_field, Bib)?;
    let name = str_getter(offset_for_field, Name)?;
    let city_state = str_getter(offset_for_field, CityState)?;
    let gender_rank = str_getter(offset_for_field, GenderRank)?;
    let final_time = getter::<Duration>(offset_for_field, FinalTime)?;
    let pace = getter::<Duration>(offset_for_field, Pace)?;

    let td = Selector::parse("td").unwrap();
    let candidates: Vec<_> = table
        .select(&Selector::parse("tbody tr").unwrap())
        .filter_map(|elem| {
            let tds: Vec<_> = elem.select(&td).collect();
            Some(Placement {
                place: place(&tds)?,
                bib: bib(&tds)?,
                name: name(&tds)?,
                city_state: city_state(&tds),
                gender_rank: gender_rank(&tds)?,
                final_time: final_time(&tds)?,
                pace: pace(&tds)?,
            })
        })
        .collect();
    (!candidates.is_empty()).then_some(candidates)
}

impl<'doc> Placement<'doc> {
    fn gender_count(&self) -> Option<&'doc str> {
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

pub fn names_and_times(input: &str) -> OptionalResults {
    let document = Html::parse_document(input);
    Placement::results(&document).and_then(|placements| {
        let (male, female) = male_and_female_counts(&placements)?;
        Some(
            placements
                .into_iter()
                .map(|p| {
                    (
                        p.name.to_string().into(),
                        p.final_time,
                        p.morf(male, female),
                    )
                })
                .collect(),
        )
    })
}

fn male_and_female_counts<'doc>(placements: &[Placement<'doc>]) -> Option<(&'doc str, &'doc str)> {
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
