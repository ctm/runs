use {
    std::{
        error::Error as StdErrorError,
        fmt::{Display, Formatter, Result},
    },
    structopt::clap,
};

#[derive(Debug)]
pub enum Error {
    Internal(String),
    Io(std::io::Error),
    Clap(clap::Error),
    // FromUtf(std::string::FromUtf8Error),
    // WalkDir(walkdir::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let message = match self {
            Error::Internal(message) => message,
            Error::Io(err) => err.description(),
            Error::Clap(err) => err.description(),
        };
        write!(f, "{}", message)
    }
}

impl std::error::Error for Error {}

impl From<&str> for Error {
    fn from(error: &str) -> Self {
        Error::Internal(error.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<clap::Error> for Error {
    fn from(error: clap::Error) -> Self {
        Error::Clap(error)
    }
}
