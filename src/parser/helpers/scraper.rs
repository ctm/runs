use {
    scraper::{ElementRef, Selector},
    std::{collections::HashMap, hash::Hash, str::FromStr},
};

pub(crate) trait GetAndParse<'a> {
    fn get_str(&self, idx: usize) -> Option<&'a str>;
    fn get_and_parse<T: FromStr>(&self, idx: usize) -> Option<T>;
    fn get_strs(&self, idx: usize, selectors: &[Selector]) -> Option<Vec<&'a str>>;
}

impl<'a> GetAndParse<'a> for Vec<ElementRef<'a>> {
    fn get_str(&self, idx: usize) -> Option<&'a str> {
        self.get(idx)?.text().next()
    }

    fn get_and_parse<T: FromStr>(&self, idx: usize) -> Option<T> {
        self.get_str(idx)?.parse().ok()
    }

    fn get_strs(&self, idx: usize, selectors: &[Selector]) -> Option<Vec<&'a str>> {
        let e = self.get(idx)?;
        selectors
            .iter()
            .map(|s| e.select(s).next().and_then(|e| e.text().next()))
            .collect()
    }
}

pub(crate) fn fields_for_indexes<T: FromStr + Eq + Hash>(table: ElementRef) -> HashMap<T, usize> {
    HashMap::from_iter(
        table
            .select(&Selector::parse("thead th").unwrap())
            .enumerate()
            .filter_map(|(index, elem)| {
                elem.text()
                    .next()
                    .and_then(|t| t.parse().ok().map(|f| (f, index)))
            }),
    )
}
