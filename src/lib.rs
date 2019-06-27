#![feature(proc_macro_hygiene)]

mod error;
mod names;
mod parsers;

pub use error::Error;

use {
    crate::parsers::{ccr_timing, ultra_signup, web_scorer, NameAndTime},
    reqwest::Url,
    sports_metrics::duration::Duration,
    std::{
        collections::HashMap,
        fs::File,
        io::Read,
        path::{Path, PathBuf},
        str::FromStr,
    },
    structopt::StructOpt,
};

type Result<T> = std::result::Result<T, Error>;

pub fn summarize(config: &Config) -> Result<()> {
    let mut h: HashMap<String, Vec<Option<Duration>>> = HashMap::new();
    let n = config.results.len();

    for (i, source) in config.results.iter().enumerate() {
        match source {
            Source::Url(url) => println!("TODO: support urls ({})", url),
            Source::File(pathbuf) => {
                let mut file = File::open(pathbuf)?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;

                if let Some(results) = ccr_timing::Results::results(&contents) {
                    let names_and_times = ccr_timing::Placement::names_and_times(&results.soloists);
                    merge(&mut h, names_and_times, i, n);
                } else if let Some(results) = ultra_signup::Placement::results(&contents) {
                    let names_and_times = ultra_signup::Placement::names_and_times(&results);
                    merge(&mut h, names_and_times, i, n);
                } else if let Some(results) = web_scorer::Placement::results(&contents) {
                    let names_and_times = web_scorer::Placement::names_and_times(&results);
                    merge(&mut h, names_and_times, i, n);
                }
            }
        }
    }
    print(h);
    Ok(())
}

fn merge(
    h: &mut HashMap<String, Vec<Option<Duration>>>,
    names_and_times: Vec<&dyn NameAndTime>,
    i: usize,
    n: usize,
) {
    for result in names_and_times {
        let name = names::canonical(&result.name());
        // TODO: get rid of the unconditional to_string below
        let durations = h.entry(name.to_string()).or_insert_with(|| {
            let mut v = Vec::with_capacity(n);
            for _ in 0..n {
                v.push(None);
            }
            v
        });
        durations[i] = Some(result.time());
    }
}

fn print(all_results: HashMap<String, Vec<Option<Duration>>>) {
    let mut results: Vec<_> = all_results
        .iter()
        .filter_map(|(name, times)| {
            if times.iter().all(Option::is_some) {
                let times: Vec<_> = times.iter().flatten().collect();
                let total: Duration = times.iter().cloned().sum();

                Some((total, name, times))
            } else {
                None
            }
        })
        .collect();
    results.sort();

    for (total, name, times) in results {
        print!("{:>8} ", total);
        for time in times {
            print!("{:>8} ", time);
        }
        println!(" {}", name);
    }
}

#[derive(Debug)]
enum Source {
    Url(Url),
    File(PathBuf),
}

impl FromStr for Source {
    type Err = Error;

    fn from_str(arg: &str) -> Result<Self> {
        match Url::parse(arg) {
            Ok(url) => Ok(Source::Url(url)),
            _ => Ok(Source::File(Path::new(arg).to_path_buf())),
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt()]
/// Runs merges results from races, keeping track of who has completed
/// all races.  The output is a set of lines, one per person, sorted
/// by sum of that person's races.
///
/// This tool was initially written to calculate results for
/// "Mt. Taylor Doublers" (people who have run both the Mount Taylor
/// 50k and completed the Mount Taylor Winter Quadrathlon), so it
/// knows how to parse the results from those two races.  However, it
/// also knows how to parse at least some webscorer results.
///
/// This app has many short-comings, the biggest being that if you supply
/// a url instead of a filename for the results, runs will not cache the
/// results and will hit the website that is providing that result.
pub struct Config {
    /// filename or url
    results: Vec<Source>,
}

impl Config {
    pub fn new() -> Result<Self> {
        Ok(Config::from_iter_safe(std::env::args())?)
    }
}
