mod names;
mod parser;

use {
    crate::parser::{
        ancient_ultra_signup, ath_links, ccr_timing, chrono_track, run_fit, runsignup, taos,
        ultra_signup, web_scorer,
    },
    anyhow::{Error, Result},
    digital_duration_nom::duration::Duration,
    reqwest::Url,
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

type Parser = &'static dyn Fn(&str) -> Option<Vec<(Cow<str>, Duration)>>;

pub fn summarize(config: &Config) -> Result<()> {
    let mut h: HashMap<String, Vec<Option<Duration>>> = HashMap::new();
    let n = config.results.len();

    let parsers = vec![
        &ultra_signup::StatusesWithPlacements::names_and_times as Parser,
        &ccr_timing::Placement::soloist_names_and_times as Parser,
        &web_scorer::Placement::names_and_times as Parser,
        &run_fit::Placement::names_and_times as Parser,
        &runsignup::Placement::names_and_times as Parser,
        &ath_links::Placement::names_and_times as Parser,
        &chrono_track::Placement::names_and_times as Parser,
        &taos::Placement::names_and_times as Parser,
        &ancient_ultra_signup::Placement::names_and_times as Parser,
    ];

    for (i, source) in config.results.iter().enumerate() {
        match source {
            // For now, we just dump the body and don't actually use
            // it.  Of course if that body is saved into a file, we
            // can then use the file.  Without caching, I don't think
            // we want to support using urls directly.
            Source::Url(url) => {
                let url = url.to_string();
                eprintln!("url: {}", url);
            }
            Source::File(pathbuf) => {
                let mut file = File::open(pathbuf)?;
                let mut bytes = Vec::new();
                file.read_to_end(&mut bytes)?;

                let contents = String::from_utf8_lossy(&bytes);

                if let Some(names_and_times) = parsers.iter().find_map(|parser| parser(&contents)) {
                    // dump_ian_scores(&names_and_times);
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
            Some(durations) => {
                if let Some(old_duration) = durations[i] {
                    eprintln!(
                        "Previous time of {} for {}, new time: {}",
                        old_duration, name, duration
                    );
                }
                durations[i] = Some(duration)
            }
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
        print!("{:>9.1} ", total);
        for time in times {
            print!("{:>8.1} ", time);
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

// This code never should have gone into master, but I wound up doing
// a bunch of work in my for-ian branch that really should have been
// done elsewhere.  However, I had a soft deadline to get for-ian
// done, so I didn't care.  Then covid hit and that soft deadline
// disappeared, so rather than finish for-ian, I simply did some
// minimal squashing and tossed it in.
#[allow(dead_code)]
fn dump_ian_scores(names_and_times: &[(Cow<str>, Duration)]) {
    let mut names_and_times = names_and_times.to_vec();

    names_and_times.sort_by_key(|&(_, time)| time);
    let n = names_and_times.len();
    let half_n = n / 2;
    let median = if n % 2 == 1 {
        names_and_times[half_n].1
    } else {
        (names_and_times[half_n - 1].1 + names_and_times[half_n].1) / 2
    };
    let median: f64 = median.into();
    for (name, time) in names_and_times {
        let time: f64 = time.as_secs() as f64;
        println!("{:>3}: {}", (median / time * 100.0).floor(), name);
    }
}
