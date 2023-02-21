use crate::hashes::ALIASES;
use std::borrow::Cow;

pub fn canonical(name_or_alias: Cow<str>) -> Cow<str> {
    if let Some(name) = ALIASES.get(name_or_alias.as_ref()) {
        Cow::from(*name)
    } else {
        name_or_alias
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical() {
        assert_eq!(canonical(Cow::from("Megan Devan")), "Rae Devan");
    }
}
