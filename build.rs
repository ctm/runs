// Hardcoding these is poor form.  However, doing it this way gives me
// a chance to play with phf and also allows me to write more
// important functionality first.  It also means that the mapping is
// under source control, which _might_ be handy.

use std::{
    env,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

fn main() {
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen.rs");
    let mut file = BufWriter::new(File::create(path).unwrap());

    writeln!(&mut file, "#[allow(clippy::unreadable_literal)]\nstatic ALIASES: phf::Map<&'static str, &'static str> = {};",
    phf_codegen::Map::new()
        .entry("Rodrigo Romeradelafuente", "\"Rodrigo Romera\"")
        .entry("Megan Devan", "\"Rae Devan\"")
        .entry("Tim Shultz", "\"Timothy Shultz\"")
        .entry("deadhead", "\"Clifford Matthews\"")
        .entry("Crystal Andersen", "\"Crystal Anderson\"")
        .entry("Guil Marez", "\"Guill Marez\"")
        .entry("Kim Brooks", "\"Kimberly Brooks\"")
        .entry("Matt Hickey", "\"Matthew Hickey\"")
        .entry("Ed Matteo", "\"Edward Matteo\"")
        .entry("Greg Huey", "\"Gregory Huey\"")
        .entry("Matthew Procter", "\"Matt Procter\"")
        .entry("Kenneth Oconnor", "\"Kenneth O'Connor\"")
        .entry("Michelle Bourret", "\"Suzanne Bourret\"")
        .entry("Matthew Swanson", "\"Matt Swanson\"")
        .entry("Jennifer Galasso", "\"Jenny Galasso\"")
        .build()).unwrap();
}
