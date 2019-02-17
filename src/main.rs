use runs::parsers::ccr_timing;
use runs::parsers::ultra_signup::Placement;
use sports_metrics::duration::Duration;
use std::{fs::File, io::Read};

// NOTE: this is hack and slash code.  Very little of this should be in main.
// TODO: refactor!

fn main() {
    let mut file = File::open("assets/2019.html").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let quad_results = ccr_timing::Results::results(&contents);
    let quad_names_and_times = ccr_timing::Placement::names_and_times(&quad_results.soloists);

    file = File::open("assets/mt_taylor_50k_2018.json").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let fiftyk_results = Placement::results(&contents);
    let fiftyk_names_and_times = Placement::names_and_times(&fiftyk_results);

    let v = vec![fiftyk_names_and_times, quad_names_and_times];
    let all_results = runs::merged_results(&v);
    let mut doublers: Vec<_> = all_results
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
    doublers.sort();

    for (total, name, times) in doublers {
        print!("{:>8} ", total);
        for time in times {
            print!("{:>8} ", time);
        }
        println!(" {}", name);
    }
}
