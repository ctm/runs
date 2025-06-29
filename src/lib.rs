use parser::race_result_mhtml;

mod hashes;
mod names;
mod parser;

use {
    crate::parser::{
        ancient_ultra_signup, athlinks, ccr_timing, chrono_track, csv, its_your_race, race_roster,
        run_fit, runsignup, runsignup_20240506_mhtml, runsignup_mhtml, taos, ultra_signup,
        ultra_signup_mhtml, web_scorer,
    },
    anyhow::{Error, Result, bail},
    clap::Parser,
    digital_duration_nom::duration::Duration,
    itertools::Itertools,
    mail_parser::MessageParser,
    reqwest::Url,
    serde::Deserialize,
    std::{
        borrow::Cow,
        cmp::Reverse,
        collections::HashMap,
        fmt::{self, Display, Formatter},
        fs::{self, DirEntry, File},
        io::{self, Read},
        path::{Path, PathBuf},
        str::FromStr,
    },
};

pub fn summarize(config: &Config) -> Result<()> {
    if config.results.len() == 1 {
        if let Source::File(p) = &config.results[0] {
            if p.is_dir() {
                return summarize_scores(p);
            }
        }
    }
    summarize_total_times(config)
}

fn summarize_scores(p: &Path) -> Result<()> {
    let mut entries = fs::read_dir(p)?.peekable();

    match entries.peek() {
        None => Ok(()),
        Some(p) => {
            match p {
                Err(e) => {
                    // I don't know how a better way to create an error
                    // that I can return return here.  e is &io::Error
                    // and surprisingly, io::Error doesn't implement Clone.
                    bail!("trouble getting first path: {e:?}")
                }
                Ok(p) => {
                    if p.file_type()?.is_dir() {
                        summarize_directories(entries)
                    } else {
                        summarize_files(entries)
                    }
                }
            }
        }
    }
}

fn summarize_directories(entries: impl Iterator<Item = io::Result<DirEntry>>) -> Result<()> {
    let (paths, scores) = score_directories(entries)?;
    let mut scores = scores
        .into_iter()
        .map(|(name, scores)| {
            (
                name,
                scores
                    .iter()
                    .map(|ScoreInfo { points, count, .. }| (*points, *count))
                    .reduce(|(total_points, total_count), (points, count)| {
                        (total_points + points, total_count + count)
                    }),
                scores,
            )
        })
        .collect::<Vec<_>>();
    scores.sort_by_key(|(_, points, _)| Reverse(*points));
    let rank_width = scores.len().ilog10() as usize + 1;
    let mut old_rank = 1;
    let mut old_points = 0;
    let mut upcoming_rank = 1;
    let mut need_nl = false;
    for (name, points_and_counts, events) in scores {
        let (points, count) = points_and_counts.unwrap();
        if need_nl {
            println!();
        } else {
            need_nl = true;
        }
        let rank = if points == old_points {
            old_rank
        } else {
            old_points = points;
            old_rank = upcoming_rank;
            upcoming_rank
        };
        upcoming_rank += 1;
        println!("{rank:>rank_width$} {points:>4} {count:>3} {name}");
        for ScoreInfo {
            points,
            path_index,
            count: _,
        } in events
        {
            println!(
                "{:rank_width$}  {points:>3} {count:>3}   {}",
                "",
                paths_indexed(&paths, path_index)
            );
        }
    }
    Ok(())
}

fn paths_indexed(paths: &[PathBuf], index: u8) -> Cow<str> {
    paths[index as usize].file_stem().unwrap().to_string_lossy()
}

#[allow(clippy::type_complexity)]
fn score_directories(
    entries: impl Iterator<Item = io::Result<DirEntry>>,
) -> Result<(Vec<PathBuf>, HashMap<String, Vec<ScoreInfo>>)> {
    let mut paths = vec![];
    fold_paths(
        entries,
        |p| p.read_dir().map_err(|e| e.into()).and_then(score_files),
        |mut h: HashMap<_, Vec<ScoreInfo>>, (_i, (mut new_paths, scores))| {
            let offset = paths.len() as u8;
            paths.append(&mut new_paths);
            for (name, mut score_info) in scores.into_iter() {
                let ScoreInfo {
                    ref mut path_index, ..
                } = score_info;
                *path_index += offset;
                h.entry(name).or_default().push(score_info);
            }
            h
        },
    )
    .map(|(_, h)| (paths, h))
}

#[derive(Debug)]
struct ScoreInfo {
    points: u16,
    path_index: u8,
    count: u8,
}

fn fold_paths<IN, OUT>(
    entries: impl Iterator<Item = io::Result<DirEntry>>,
    p_to_in: impl Fn(&Path) -> Result<IN>,
    f: impl FnMut(HashMap<String, OUT>, (usize, IN)) -> HashMap<String, OUT>,
) -> Result<(Vec<PathBuf>, HashMap<String, OUT>)> {
    let mut paths = vec![];
    let scores = entries
        .map(|entry| {
            entry.map_err(|e| e.into()).and_then(|entry| {
                paths.push(entry.path());
                p_to_in(paths.last().unwrap())
            })
        })
        .enumerate()
        .map(|(i, r)| r.map(|v| (i, v)))
        .fold_ok(HashMap::<String, OUT>::new(), f)?;
    Ok((paths, scores))
}

fn score_files(
    entries: impl Iterator<Item = io::Result<DirEntry>>,
) -> Result<(Vec<PathBuf>, HashMap<String, ScoreInfo>)> {
    fold_paths(entries, contents, |mut h, (i, contents)| {
        if let Some(mut names_and_times) = PARSERS.iter().find_map(|parser| parser(&contents)) {
            names_and_times.sort_by_key(|&(_, time, _)| time);
            let mut firsts = [None; 2];
            for (name, time, morf) in names_and_times {
                let time = (*time).as_secs();
                if let Some(morf) = morf {
                    let morf = morf as usize;
                    if morf < 2 {
                        let new_points = match &firsts[morf] {
                            None => {
                                firsts[morf] = Some(time);
                                100
                            }
                            Some(first) => (first * 100 / time) as u16,
                        };
                        if let Some(ScoreInfo {
                            points,
                            path_index,
                            count,
                        }) = h.get_mut(name.as_ref())
                        {
                            *count += 1;
                            if new_points > *points {
                                *points = new_points;
                                *path_index = i as u8;
                            }
                        } else {
                            h.insert(
                                names::canonical(name).into_owned(),
                                ScoreInfo {
                                    points: new_points,
                                    path_index: i as u8,
                                    count: 1,
                                },
                            );
                        }
                    }
                }
            }
        }
        h
    })
}

fn summarize_files(entries: impl Iterator<Item = io::Result<DirEntry>>) -> Result<()> {
    let (paths, scores) = score_files(entries)?;
    let mut scores = scores.into_iter().collect::<Vec<_>>();

    scores.sort_by_key(|&(_, ScoreInfo { points, .. })| Reverse(points));
    let width = scores.iter().map(|(name, ..)| name.len()).max().unwrap();
    for (
        name,
        ScoreInfo {
            points,
            path_index,
            count,
        },
    ) in scores
    {
        println!(
            "{points:>3}: {count:>3} {name:width$} {}",
            paths_indexed(&paths, path_index)
        );
    }
    Ok(())
}

static PARSERS: [fn(&str) -> OptionalResults; 16] = [
    ultra_signup::StatusesWithPlacements::names_and_times,
    ccr_timing::Placement::soloist_names_and_times,
    web_scorer::Placement::names_and_times,
    run_fit::Placement::names_and_times,
    runsignup_20240506_mhtml::Placement::names_and_times,
    runsignup::Placement::names_and_times,
    athlinks::Placement::names_and_times,
    chrono_track::Placement::names_and_times,
    taos::Placement::names_and_times,
    ancient_ultra_signup::Placement::names_and_times,
    ultra_signup_mhtml::StatusesWithPlacements::names_and_times,
    runsignup_mhtml::Placement::names_and_times,
    race_roster::Placement::names_and_times,
    its_your_race::Placement::names_and_times,
    csv::Placement::names_and_times,
    race_result_mhtml::names_and_times,
];

fn contents(p: &Path) -> Result<String> {
    let mut file = File::open(p)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;

    Ok(MessageParser::default()
        .parse(&bytes)
        .and_then(|message| {
            if message.from().is_none() {
                None
            } else {
                message.body_html(0).map(|body| body.into_owned())
            }
        })
        .unwrap_or_else(|| {
            let candidate = String::from_utf8_lossy(&bytes);
            if candidate.contains("<br/>") {
                candidate.replace("<br/>", "\n") // for quad/2012.html
            } else {
                candidate.into_owned()
            }
        }))
}

fn summarize_total_times(config: &Config) -> Result<()> {
    let mut h: HashMap<String, Vec<Option<Duration>>> = HashMap::new();
    let n = config.results.len();

    for (i, source) in config.results.iter().enumerate() {
        match source {
            // For now, we just dump the body and don't actually use
            // it.  Of course if that body is saved into a file, we
            // can then use the file.  Without caching, I don't think
            // we want to support using urls directly.
            Source::Url(url) => {
                let url = url.to_string();
                eprintln!("url: {url}");
            }
            Source::File(pathbuf) => {
                let contents = contents(pathbuf)?;
                if let Some(names_and_times) = PARSERS.iter().find_map(|parser| parser(&contents)) {
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
    names_and_times: Vec<(Cow<str>, Duration, Option<MaleOrFemale>)>,
    i: usize,
    n: usize,
) {
    for (name, duration, _) in names_and_times {
        let name = names::canonical(name);
        match h.get_mut(name.as_ref()) {
            Some(durations) => {
                if let Some(old_duration) = durations[i] {
                    eprintln!("Previous time of {old_duration} for {name}, new time: {duration}");
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

    let (total_width, name_width, times_widths) = widths(&results);

    for (total, name, times) in results {
        print!("{name:>name_width$}");
        print!(" {total:>total_width$.1}");
        for (time, time_width) in times.iter().zip(times_widths.iter()) {
            print!(" {time:>time_width$.1}");
        }
        println!();
    }
}

fn widths(results: &[(Duration, &String, Vec<&Duration>)]) -> (usize, usize, Vec<usize>) {
    use std::cmp::max;

    let n_elements = results.first().map(|(_, _, v)| v.len()).unwrap_or(0);
    let time_widths = vec![0; n_elements];
    results
        .iter()
        .fold((0, 0, time_widths), |mut triple, (total, name, times)| {
            let (ref mut total_width, ref mut name_width, ref mut times_width) = triple;
            *total_width = max(*total_width, format!("{total:.1}").len());
            *name_width = max(*name_width, name.to_string().len());
            for (times_width, time) in times_width.iter_mut().zip(times) {
                *times_width = max(*times_width, format!("{time:.1}").len());
            }
            triple
        })
}

#[derive(Clone, Debug)]
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

#[derive(Debug, Parser)]
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
        Ok(Config::try_parse_from(std::env::args())?)
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

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
pub enum MaleOrFemale {
    Male = 0,
    Female = 1,
    NonBinary = 2,
}

impl Display for MaleOrFemale {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MaleOrFemale::Male => "M",
                MaleOrFemale::Female => "F",
                MaleOrFemale::NonBinary => "X",
            }
        )
    }
}

impl FromStr for MaleOrFemale {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        use MaleOrFemale::*;

        match value {
            "M" => Ok(Male),
            "F" => Ok(Female),
            "X" => Ok(NonBinary),
            other => Err(format!("Unkown sex: {other}")),
        }
    }
}

pub(crate) trait Gender {
    fn gender(&self) -> &str;
}

pub(crate) trait Morf: Gender {
    fn morf(&self) -> Option<MaleOrFemale> {
        use MaleOrFemale::*;

        match self.gender() {
            "M" | "Male" => Some(Male),
            "F" | "Female" => Some(Female),
            "X" => Some(NonBinary),
            "U" | "" => None,
            other => panic!("Unknown gender: {other}"),
        }
    }
}

impl<T: Gender> Morf for T {}

pub(crate) type OptionalResults<'a> = Option<Vec<(Cow<'a, str>, Duration, Option<MaleOrFemale>)>>;

pub(crate) mod prelude {
    pub(crate) use super::{Gender, MaleOrFemale, Morf, OptionalResults};
    pub(crate) use std::borrow::Cow;
}
