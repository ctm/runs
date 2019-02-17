// TODO: hardcoding these is poor form.  However, doing it this way gives
//       me a chance to play with phf and also allows me to write more
//       important functionality first.

use phf::phf_map;

static ALIASES: phf::Map<&'static str, &'static str> = phf_map! {
    "Rodrigo Romeradelafuente" => "Rodrigo Romera",
    "Megan Devan" => "Rae Devan",
    "Tim Shultz" => "Timothy Shultz",
    "deadhead" => "Clifford Matthews",
};

pub fn canonical(name_or_alias: &str) -> &str {
    if let Some(name) = ALIASES.get(name_or_alias) {
        name
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
