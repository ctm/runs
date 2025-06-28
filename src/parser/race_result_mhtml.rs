use {
    super::helpers::scraper::{GetAndParse, fields_for_indexes},
    crate::prelude::*,
    digital_duration_nom::duration::Duration,
    scraper::{ElementRef, Html, Selector},
    std::{collections::HashMap, fmt::Debug, hash::Hash, num::NonZeroU16, str::FromStr},
};

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

impl FromStr for Field {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
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

    let place = *offset_for_field.get(&Place)?;
    let bib = *offset_for_field.get(&Bib)?;
    let name = *offset_for_field.get(&Name)?;
    let city_state = *offset_for_field.get(&CityState)?;
    let gender_rank = *offset_for_field.get(&GenderRank)?;
    let final_time = *offset_for_field.get(&FinalTime)?;
    let pace = *offset_for_field.get(&Pace)?;

    let td = Selector::parse("td").unwrap();
    let candidates: Vec<_> = table
        .select(&Selector::parse("tbody tr").unwrap())
        .filter_map(|elem| {
            let tds: Vec<_> = elem.select(&td).collect();
            Some(Placement {
                place: tds.get_and_parse(place)?,
                bib: tds.get_and_parse(bib)?,
                name: tds.get_str(name)?,
                city_state: tds.get_str(city_state),
                gender_rank: tds.get_str(gender_rank)?,
                final_time: tds.get_and_parse(final_time)?,
                pace: tds.get_and_parse(pace)?,
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
            .and_then(|table| placements(table, &fields_for_indexes(table)))
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
