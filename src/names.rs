// TODO: hardcoding these is poor form.  However, doing it this way gives
//       me a chance to play with phf and also allows me to write more
//       important functionality first.

use {phf::phf_map, std::borrow::Cow};

static ALIASES: phf::Map<&'static str, &'static str> = phf_map! {
    "Rodrigo Romeradelafuente" => "Rodrigo Romera",
    "Megan Devan" => "Rae Devan",
    "Tim Shultz" => "Timothy Shultz",
    "deadhead" => "Clifford Matthews",
    "Crystal Andersen" => "Crystal Anderson",
    "Guil Marez" => "Guill Marez",
    "Kim Brooks" => "Kimberly Brooks",
    "Matt Hickey" => "Matthew Hickey",
    "Ed Matteo" => "Edward Matteo",
    "Greg Huey" => "Gregory Huey",
    "Matthew Procter" => "Matt Procter",
    "Kenneth Oconnor" => "Kenneth O'Connor",
    "Michelle Bourret" => "Suzanne Bourret",
    "Matthew Swanson" => "Matt Swanson",
};

pub fn canonical<'a>(name_or_alias: Cow<str>) -> Cow<str> {
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
        assert_eq!(canonical("Megan Devan"), "Rae Devan");
    }
}
