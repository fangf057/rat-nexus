use snafu::prelude::*;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    #[snafu(display("Failed to lock mutex: poisoned"))]
    LockPoisoned,

    #[snafu(display("Terminal error: {source}"))]
    TerminalError { source: std::io::Error },

    #[snafu(display("IO error: {source}"))]
    IoError { source: std::io::Error },

    #[snafu(display("Task execution error"))]
    TaskError,
}

pub type Result<T> = std::result::Result<T, Error>;
