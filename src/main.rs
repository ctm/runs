use {anyhow::Error, runs::Config};

fn runs() -> Result<(), Error> {
    let config = Config::new()?;

    runs::summarize(&config)
}

fn main() {
    std::process::exit(match runs() {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("{}", err);
            1
        }
    });
}
