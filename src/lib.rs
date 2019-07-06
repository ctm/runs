mod error;
mod names;
mod parser;

pub use error::Error;

use {
    crate::parser::{ccr_timing, run_fit, ultra_signup, web_scorer},
    reqwest::Url,
    sports_metrics::duration::Duration,
    std::{
        borrow::Cow,
        collections::HashMap,
        fs::File,
        io::Read,
        path::{Path, PathBuf},
        str::FromStr,
        string::String,
    },
    structopt::StructOpt,
};

type Result<T> = std::result::Result<T, Error>;

type Parser = &'static dyn Fn(&str) -> Option<Vec<(Cow<str>, Duration)>>;

pub fn summarize(config: &Config) -> Result<()> {
    let mut h: HashMap<String, Vec<Option<Duration>>> = HashMap::new();
    let n = config.results.len();

    let parsers = vec![
        &ccr_timing::Placement::soloist_names_and_times as Parser,
        &ultra_signup::Placement::names_and_times as Parser,
        &web_scorer::Placement::names_and_times as Parser,
        &run_fit::Placement::names_and_times as Parser,
    ];

    for (i, source) in config.results.iter().enumerate() {
        match source {
            Source::Url(_url) => println!("URLs are deliberately unsupported"),
            Source::File(pathbuf) => {
                let mut file = File::open(pathbuf)?;
                let mut bytes = Vec::new();
                file.read_to_end(&mut bytes)?;

                let contents = String::from_utf8_lossy(&bytes);

                if let Some(names_and_times) = parsers.iter().find_map(|parser| parser(&contents)) {
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
    names_and_times: Vec<(Cow<str>, Duration)>,
    i: usize,
    n: usize,
) {
    for (name, duration) in names_and_times {
        let name = names::canonical(name);
        match h.get_mut(name.as_ref()) {
            Some(durations) => durations[i] = Some(duration),
            None => {
                let mut v = Vec::with_capacity(n);
                for index in 0..n {
                    if index == i {
                        v.push(Some(duration))
                    } else {
                        v.push(None);
                    }
                }
                h.insert(name.to_string(), v);
            }
        }
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
        print!("{:>9} ", total);
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
