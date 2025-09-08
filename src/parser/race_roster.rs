use {
    crate::prelude::*,
    digital_duration_nom::duration::Duration,
    scraper::{ElementRef, Html, Selector},
    std::{collections::HashMap, fmt::Debug, num::NonZeroU8, str::FromStr},
};

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct Placement<'a> {
    bib: &'a str,
    name: &'a str,
    age: NonZeroU8,
    gender: &'a str,
    age_group: &'a str,
    city: Option<&'a str>,
    state: Option<&'a str>,
    chip_time: Duration,
    place: u16,
    // Gender Pace, e.g., "1 / 21" ignored
    // Age Rank, e.g. "1st MARATHON Male" ignored
    // apparently optional Team Name ignored
}

const BIB: &str = "Bib";
const NAME: &str = "Name";
const AGE: &str = "Age";
const GENDER: &str = "Gender";
const AGE_GROUP: &str = "Age Group";
const CITY: &str = "City";
const STATE: &str = "State";
const CHIP_TIME: &str = "Chip Time";
const PLACE: &str = "Overall Place";

impl Placement<'_> {
    pub fn names_and_times(input: &str) -> OptionalResults<'_> {
        let document = Html::parse_document(input);
        Self::results(&document).map(|results| {
            results
                .into_iter()
                .map(|p| (p.name.to_string().into(), p.chip_time, p.morf()))
                .collect()
        })
    }

    fn results(document: &Html) -> Option<Vec<Placement<'_>>> {
        let tbody = Selector::parse("tbody").unwrap();
        let tr = Selector::parse("tr").unwrap();
        let td = Selector::parse("td").unwrap();

        let tbody = document.select(&tbody).next()?;

        let mapper = ColumnMapper::from_document(document)?;

        tbody
            .select(&tr)
            .map(|e| {
                let values = ColumnValues::from_element(&mapper, &td, e);

                let bib = values.value(BIB)?;
                let name = values.value(NAME)?;
                let age = values.parsed_value(AGE)?;
                let gender = values.value(GENDER)?;
                let age_group = values.value(AGE_GROUP)?;
                let city = values.value(CITY);
                let state = values.value(STATE);
                let chip_time = values.parsed_value(CHIP_TIME)?;
                let place = values.parsed_value(PLACE)?;
                Some(Placement {
                    bib,
                    name,
                    age,
                    gender,
                    age_group,
                    city,
                    state,
                    chip_time,
                    place,
                })
            })
            .collect()
    }
}

impl Gender for Placement<'_> {
    fn gender(&self) -> &str {
        self.gender
    }
}

struct ColumnMapper<'a> {
    header_to_index: HashMap<&'a str, usize>,
}

impl ColumnMapper<'_> {
    fn from_document(document: &Html) -> Option<ColumnMapper<'_>> {
        let thead = Selector::parse("thead tr").unwrap();
        let th_span = Selector::parse("th>div>span").unwrap();
        let thead = document.select(&thead).next()?;

        let header_to_index = thead
            .select(&th_span)
            .enumerate()
            .map(|(i, e)| e.text().next().map(|t| (t, i)))
            .collect::<Option<Vec<_>>>()?
            .into_iter()
            .collect::<HashMap<_, _>>();
        Some(ColumnMapper { header_to_index })
    }
}

struct ColumnValues<'doc, 'a> {
    mapper: &'a ColumnMapper<'doc>,
    values: Vec<&'doc str>,
}

impl<'doc> ColumnValues<'doc, '_> {
    fn from_element<'d, 'b>(
        mapper: &'b ColumnMapper<'d>,
        td: &Selector,
        e: ElementRef<'d>,
    ) -> ColumnValues<'d, 'b> {
        let values = e
            .select(td)
            .map(|e| e.text().next().unwrap_or(""))
            .collect::<Vec<_>>();

        ColumnValues { mapper, values }
    }

    fn value(&self, key: &str) -> Option<&'doc str> {
        self.mapper
            .header_to_index
            .get(key)
            .and_then(|i| self.values.get(*i).copied())
    }

    fn parsed_value<T: FromStr>(&self, key: &str) -> Option<T> {
        self.value(key)?.parse().ok()
    }
}
