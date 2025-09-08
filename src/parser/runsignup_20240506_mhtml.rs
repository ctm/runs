use {
    super::helpers::scraper::{GetAndParse, fields_for_indexes},
    crate::prelude::*,
    digital_duration_nom::duration::Duration,
    scraper::{ElementRef, Html, Selector},
    std::{
        collections::HashMap,
        fmt::Debug,
        num::{NonZeroU8, NonZeroU16},
        str::FromStr,
    },
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum Field {
    Place,
    Bib,
    Name,
    Gender,
    GenderPlace,
    Age,
    City,
    State,
    ChipTime,
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
            "Gender" => Ok(Gender),
            "Gender Place" => Ok(GenderPlace),
            "Age" => Ok(Age),
            "City" => Ok(City),
            "State" => Ok(State),
            "Chip Time" => Ok(ChipTime),
            "Pace" => Ok(Pace),
            field => Err(format!("unknown field: {field}")),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Placement<'doc> {
    place: NonZeroU16,
    bib: NonZeroU16,
    name: Vec<&'doc str>,
    gender: MaleOrFemale,
    gender_place: NonZeroU16,
    age: NonZeroU8,
    city: Option<&'doc str>,
    state: Option<&'doc str>,
    chip_time: Duration,
    pace: Duration,
}

impl<'doc> Placement<'doc> {
    fn placements(
        table: ElementRef<'doc>,
        offset_for_field: &HashMap<Field, usize>,
    ) -> Option<Vec<Placement<'doc>>> {
        use Field::*;

        let place = *offset_for_field.get(&Place)?;
        let bib = *offset_for_field.get(&Bib)?;
        let name = *offset_for_field.get(&Name)?;
        let gender = *offset_for_field.get(&Gender)?;
        let gender_place = *offset_for_field.get(&GenderPlace)?;
        let age = *offset_for_field.get(&Age)?;
        let city = *offset_for_field.get(&City)?;
        let state = *offset_for_field.get(&State)?;
        let chip_time = *offset_for_field.get(&ChipTime)?;
        let pace = *offset_for_field.get(&Pace)?;

        let name_selectors = &[
            Selector::parse(".participantName__name__firstName").unwrap(),
            Selector::parse(".participantName__name__lastName").unwrap(),
        ];

        let td = Selector::parse("td").unwrap();
        let candidates: Vec<_> = table
            .select(&Selector::parse("tbody tr").unwrap())
            .filter_map(|elem| {
                let tds: Vec<_> = elem.select(&td).collect();
                Some(Placement {
                    place: tds.get_and_parse(place)?,
                    bib: tds.get_and_parse(bib)?,
                    name: tds.get_strs(name, name_selectors)?,
                    gender: tds.get_and_parse(gender)?,
                    gender_place: tds.get_and_parse(gender_place)?,
                    age: tds.get_and_parse(age)?,
                    city: tds.get_str(city),
                    state: tds.get_str(state),
                    chip_time: tds.get_and_parse(chip_time)?,
                    pace: tds.get_and_parse(pace)?,
                })
            })
            .collect();
        (!candidates.is_empty()).then_some(candidates)
    }

    fn results(document: &'doc Html) -> Option<Vec<Placement<'doc>>> {
        document
            .select(&Selector::parse("table#resultsTable").unwrap())
            .next()
            .and_then(|table| Self::placements(table, &fields_for_indexes(table)))
    }
    pub fn names_and_times(input: &str) -> OptionalResults<'_> {
        let document = Html::parse_document(input);
        Placement::results(&document).map(|placements| {
            placements
                .into_iter()
                .map(|p| {
                    (
                        format!("{} {}", p.name[0], p.name[1]).into(),
                        p.chip_time,
                        Some(p.gender),
                    )
                })
                .collect()
        })
    }
}
